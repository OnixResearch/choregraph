//! Pinned PRISM 4.9 execution wrapper, fail-closed output parser, BLAKE3
//! receipts, and retained-subject replay.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use choregraph_choreography::endpoint::LocalProtocolSetBody;
use serde::Serialize;

use crate::model::{
    AnalysisError, AnalysisManifest, AnalysisReceipt, ProbabilitySidecar, PropertyResult,
    PropertySpec, RetainedSubject, ToolConfiguration,
};

static WORKDIR_COUNTER: AtomicU64 = AtomicU64::new(0);

const RECEIPT_CONTEXT: &str = "choregraph quantitative role pilot receipt v1";

fn domain_hash<T: Serialize>(domain: &str, value: &T) -> Result<String, AnalysisError> {
    let bytes = serde_json::to_vec(value).map_err(|err| {
        AnalysisError::MalformedCheckerOutput(format!("serialize {domain}: {err}"))
    })?;
    Ok(domain_hash_bytes(domain, &bytes))
}

fn domain_hash_bytes(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(RECEIPT_CONTEXT);
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

/// Computes a domain-separated BLAKE3 receipt binding source/endpoints,
/// annotations, abstraction policy, properties, scheduler assumptions, tool
/// configuration, and exported PRISM model bytes.
pub fn build_receipt(
    source_bytes: &[u8],
    local_set: &LocalProtocolSetBody,
    sidecar: &ProbabilitySidecar,
    manifest: &AnalysisManifest,
    exported_model: &str,
) -> Result<AnalysisReceipt, AnalysisError> {
    let local_bytes = serde_json::to_vec(local_set)
        .map_err(|err| AnalysisError::MalformedCheckerOutput(format!("serialize locals: {err}")))?;
    let mut src_and_ep = Vec::with_capacity(source_bytes.len().saturating_add(local_bytes.len()));
    src_and_ep.extend_from_slice(source_bytes);
    src_and_ep.push(0);
    src_and_ep.extend_from_slice(&local_bytes);

    let source_and_endpoints_blake3 = domain_hash_bytes("source-and-endpoints", &src_and_ep);
    let annotations_blake3 = domain_hash("annotations", sidecar)?;
    let abstraction_blake3 = domain_hash(
        "abstraction",
        &(manifest.ownership_abstraction, manifest.semantics.as_str()),
    )?;
    let properties_blake3 = domain_hash("properties", &manifest.properties)?;
    let scheduler_blake3 = domain_hash("scheduler", &manifest.scheduler_assumption)?;
    let tool_config_blake3 = domain_hash("tool-config", &manifest.tool)?;
    let exported_model_blake3 = domain_hash_bytes("exported-model", exported_model.as_bytes());

    let receipt_preimage = format!(
        "{}\n{source_and_endpoints_blake3}\n{annotations_blake3}\n{abstraction_blake3}\n{properties_blake3}\n{scheduler_blake3}\n{tool_config_blake3}\n{exported_model_blake3}",
        manifest.schema
    );
    let receipt_blake3 = domain_hash_bytes("analysis-receipt", receipt_preimage.as_bytes());

    Ok(AnalysisReceipt {
        schema: manifest.schema.clone(),
        source_and_endpoints_blake3,
        annotations_blake3,
        abstraction_blake3,
        properties_blake3,
        scheduler_blake3,
        tool_config_blake3,
        exported_model_blake3,
        receipt_blake3,
    })
}

/// Verifies that a retained receipt matches recomputed digests for every bound
/// input; any drift invalidates the receipt.
pub fn verify_receipt(
    receipt: &AnalysisReceipt,
    source_bytes: &[u8],
    local_set: &LocalProtocolSetBody,
    sidecar: &ProbabilitySidecar,
    manifest: &AnalysisManifest,
    exported_model: &str,
) -> Result<(), AnalysisError> {
    let expected = build_receipt(source_bytes, local_set, sidecar, manifest, exported_model)?;
    if *receipt != expected {
        return Err(AnalysisError::ReceiptMismatch(String::from(
            "retained quantitative analysis receipt does not match current inputs or tool config",
        )));
    }
    Ok(())
}

/// Raw PRISM output captured from one invocation.
#[derive(Clone, Debug)]
pub struct PrismRunOutput {
    pub stdout: String,
    pub stderr: String,
    pub states_txt: String,
    pub trans_txt: String,
    pub labels_txt: String,
    pub probabilities: Vec<f64>,
}

struct TempWorkdir {
    path: PathBuf,
}

impl TempWorkdir {
    fn create() -> Result<Self, AnalysisError> {
        let pid = std::process::id();
        let seq = WORKDIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("choregraph-prism-{pid}-{seq}"));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)
            .map_err(|err| AnalysisError::ToolUnavailable(format!("mkdir temp workdir: {err}")))?;
        Ok(Self { path })
    }
}

impl Drop for TempWorkdir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Executes the pinned PRISM binary on `(prism_model, props_file)`, capturing
/// both quantitative property results and PRISM's explicit state/transition/label
/// exports for independent conformance checking.
pub fn run_prism(
    tool: &ToolConfiguration,
    prism_model: &str,
    props_file: &str,
    expected_property_count: usize,
    timeout_ms: u64,
) -> Result<PrismRunOutput, AnalysisError> {
    let work = TempWorkdir::create()?;
    let model_path = work.path.join("model.prism");
    let props_path = work.path.join("check.props");
    let states_path = work.path.join("states.txt");
    let trans_path = work.path.join("trans.txt");
    let labels_path = work.path.join("labels.txt");

    std::fs::write(&model_path, prism_model)
        .map_err(|err| AnalysisError::ToolUnavailable(format!("write model: {err}")))?;
    std::fs::write(&props_path, props_file)
        .map_err(|err| AnalysisError::ToolUnavailable(format!("write props: {err}")))?;

    let mut command = Command::new(&tool.prism_binary);
    command
        .arg(&model_path)
        .arg(&props_path)
        .args(&tool.extra_args)
        .arg("-exportstates")
        .arg(&states_path)
        .arg("-exporttrans")
        .arg(&trans_path)
        .arg("-exportlabels")
        .arg(&labels_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|err| {
        AnalysisError::ToolUnavailable(format!("failed to spawn {}: {err}", tool.prism_binary))
    })?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => break,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(AnalysisError::ToolTimeout { timeout_ms });
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(err) => {
                let _ = child.kill();
                return Err(AnalysisError::ToolUnavailable(format!(
                    "wait for prism failed: {err}"
                )));
            }
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|err| AnalysisError::ToolUnavailable(format!("read prism output: {err}")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return Err(AnalysisError::ToolFailed {
            exit_code: output.status.code(),
            stderr: format!("{stderr}\n{stdout}"),
        });
    }

    let probabilities = parse_prism_results(&stdout, expected_property_count)?;
    let states_txt = std::fs::read_to_string(&states_path).map_err(|err| {
        AnalysisError::MalformedCheckerOutput(format!("missing PRISM states export: {err}"))
    })?;
    let trans_txt = std::fs::read_to_string(&trans_path).map_err(|err| {
        AnalysisError::MalformedCheckerOutput(format!("missing PRISM trans export: {err}"))
    })?;
    let labels_txt = std::fs::read_to_string(&labels_path).map_err(|err| {
        AnalysisError::MalformedCheckerOutput(format!("missing PRISM labels export: {err}"))
    })?;

    Ok(PrismRunOutput {
        stdout,
        stderr,
        states_txt,
        trans_txt,
        labels_txt,
        probabilities,
    })
}

/// Fail-closed parser for PRISM `Result: <number>` output lines.
pub fn parse_prism_results(stdout: &str, expected_count: usize) -> Result<Vec<f64>, AnalysisError> {
    let mut results = Vec::with_capacity(expected_count);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Result:") {
            let token = rest.split_whitespace().next().ok_or_else(|| {
                AnalysisError::MalformedCheckerOutput(format!("empty Result line: {trimmed}"))
            })?;
            let value: f64 = token.parse().map_err(|_| {
                AnalysisError::MalformedCheckerOutput(format!(
                    "non-numeric PRISM Result token: {token}"
                ))
            })?;
            if !value.is_finite() || !(0.0..=1.0 + 1e-9).contains(&value) {
                return Err(AnalysisError::MalformedCheckerOutput(format!(
                    "out-of-range PRISM probability: {value}"
                )));
            }
            results.push(value);
        }
    }
    if results.len() != expected_count {
        return Err(AnalysisError::MalformedCheckerOutput(format!(
            "expected {expected_count} PRISM Result lines, parsed {}",
            results.len()
        )));
    }
    Ok(results)
}

/// Replays a retained `.prism` model and `.props` file through PRISM, verifying
/// that the retained receipt's model digest matches and returning fresh results.
pub fn replay_retained_subject(
    retained: &RetainedSubject,
    properties: &[PropertySpec],
    manifest: &AnalysisManifest,
) -> Result<Vec<PropertyResult>, AnalysisError> {
    let model_hash = domain_hash_bytes("exported-model", retained.prism_model.as_bytes());
    if model_hash != retained.receipt.exported_model_blake3 {
        return Err(AnalysisError::ReceiptMismatch(String::from(
            "retained PRISM model bytes do not match receipt exported_model_blake3",
        )));
    }
    let props_hash = domain_hash("properties", &properties)?;
    if props_hash != retained.receipt.properties_blake3 {
        return Err(AnalysisError::ReceiptMismatch(String::from(
            "replay properties do not match receipt properties_blake3",
        )));
    }
    let run = run_prism(
        &manifest.tool,
        &retained.prism_model,
        &retained.props_file,
        properties.len(),
        manifest.bounds.timeout_ms,
    )?;
    let mut out = Vec::with_capacity(properties.len());
    for (prop, prob) in properties.iter().zip(run.probabilities) {
        out.push(PropertyResult {
            property: prop.clone(),
            formula: prop.pctl_formula(),
            is_bounded: prop.is_bounded(),
            scheduler_assumption: manifest.scheduler_assumption,
            probability: prob,
        });
    }
    Ok(out)
}
