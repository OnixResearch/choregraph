//! Construction admission controls use exact captured materials, never ambient Nickel state.
use super::*;

// r[verify choregraph.construction.captured]
// r[verify choregraph.construction.composition]
// r[verify choregraph.construction.registry]
// r[verify choregraph.construction.bounds]
// r[verify choregraph.construction.binding]
// r[verify choregraph.construction.laws]
// r[verify choregraph.construction.validation]

const REGISTRY_JSON: &str = include_str!("../../../../fixtures/nickel/construction/registry.json");
const LIBRARY: &[u8] = include_bytes!("../../../../library/nickel/graph-construction-v1.ncl");
const CONTRACTS: &[u8] =
    include_bytes!("../../../../contracts/nickel/graph-construction-contracts-v1.ncl");

trait Required<T> {
    fn required(self, context: &str) -> T;
}
impl<T, E: std::fmt::Debug> Required<T> for Result<T, E> {
    fn required(self, context: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{context}: {error:?}"),
        }
    }
}
impl<T> Required<T> for Option<T> {
    fn required(self, context: &str) -> T {
        match self {
            Some(value) => value,
            None => panic!("{context}: missing fixture value"),
        }
    }
}

// This is synthetic host-supplied observation for isolated native tests, not a
// claim that Nickel ran or that a captured worker observed these imports.
fn synthetic_receipt(
    candidate: &[u8],
    modules: &[crate::construction::association::CapturedModule<'_>],
    entry: &str,
) -> Vec<u8> {
    let source = modules
        .iter()
        .find(|module| module.logical_id == entry)
        .required("fixture entry");
    serde_json::to_vec(&serde_json::json!({
        "version": "nickel-eval.captured-binding.v1",
        "profile": crate::construction::model::profile::CAPTURED_EVALUATOR_PROFILE,
        "policy": "nickel-eval.logical-relative-nickel-json.v1",
        "entry": {"name": entry, "source_bytes": source.bytes.len(), "source_blake3": blake3::hash(source.bytes).to_hex().to_string()},
        "submitted_map_blake3": crate::construction::association::captured_map_blake3(modules),
        "observed": [{"name": entry, "source_bytes": source.bytes.len(), "source_blake3": blake3::hash(source.bytes).to_hex().to_string()}],
        "edges": [], "cohort": {}, "limits": {},
        "output_bytes": candidate.len(), "output_blake3": blake3::hash(candidate).to_hex().to_string()
    })).required("synthetic receipt JSON")
}

fn fixture_registry() -> crate::construction::model::host::TargetRegistry {
    serde_json::from_str(REGISTRY_JSON).required("fixture registry")
}

fn compile(
    value: &serde_json::Value,
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<crate::construction::Output, crate::construction::Failure> {
    compile_modules(value, registry, &fixture_modules(), limits)
}

fn compile_modules(
    value: &serde_json::Value,
    registry: &crate::construction::model::host::TargetRegistry,
    modules: &[crate::construction::association::CapturedModule<'_>],
    limits: crate::construction::model::profile::Limits,
) -> Result<crate::construction::Output, crate::construction::Failure> {
    let candidate = serde_json::to_vec(value).required("candidate JSON");
    let digest = blake3::hash(&candidate).to_hex().to_string();
    let observation = synthetic_receipt(&candidate, modules, "fixture.ncl");
    let materials = fixture_materials(modules, &observation, &digest);
    crate::construction::compile_json(&candidate, registry, &materials, limits)
}

fn admitted(
    value: &serde_json::Value,
    registry: &crate::construction::model::host::TargetRegistry,
) -> crate::construction::Output {
    compile(
        value,
        registry,
        crate::construction::model::profile::Limits::default(),
    )
    .required("construction must admit")
}

fn rejected(
    value: &serde_json::Value,
    registry: &crate::construction::model::host::TargetRegistry,
    failure: crate::construction::Failure,
) {
    assert_eq!(
        compile(
            value,
            registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(failure)
    );
}

fn make_candidate(body: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema": crate::construction::model::profile::CONSTRUCTION_SCHEMA, "root": "workflow", "target": "lattice-test",
        "registry": "fixture-registry", "parameters": {}, "templates": [], "body": body
    })
}

fn operation((key, name): (&str, &str)) -> serde_json::Value {
    let configuration = if name == "pass" {
        serde_json::json!({"label": {"tag": "literal", "value": "example"}})
    } else {
        serde_json::json!({})
    };
    serde_json::json!({"key": key, "kind": {"tag": "operation", "operation": name, "configuration": configuration, "effects": []}})
}

fn handle((instance, direction, boundary): (&str, &str, &str)) -> serde_json::Value {
    serde_json::json!({"root": "workflow", "instance": instance, "direction": direction, "boundary": boundary})
}

fn connection(from: (&str, &str), to: (&str, &str)) -> serde_json::Value {
    serde_json::json!({"source": handle((from.0, "output", from.1)), "target": handle((to.0, "input", to.1))})
}

fn wire(parts: Vec<serde_json::Value>, connections: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({"key": "root", "kind": {"tag": "wire", "parts": parts, "connections": connections, "inputs": [], "outputs": []}})
}

fn diamond() -> serde_json::Value {
    make_candidate(wire(
        vec![
            operation(("fork", "fork")),
            operation(("a", "pass")),
            operation(("b", "pass")),
            operation(("join", "join")),
        ],
        vec![
            connection(("fork", "left"), ("a", "in")),
            connection(("fork", "right"), ("b", "in")),
            connection(("a", "out"), ("join", "left")),
            connection(("b", "out"), ("join", "right")),
        ],
    ))
}

fn nested() -> serde_json::Value {
    let mut graph = diamond();
    graph["templates"] = serde_json::json!([
        {"name": "cell", "parameters": ["label"], "body": {
            "key": "cell-body", "kind": {"tag": "operation", "operation": "pass",
            "configuration": {"label": {"tag": "parameter", "name": "label"}}, "effects": []}}},
        {"name": "wrapper", "parameters": ["label"], "body": {
            "key": "nested", "kind": {"tag": "instantiate", "template": "cell",
            "arguments": {"label": {"tag": "parameter", "name": "label"}}}}}
    ]);
    for (index, label) in [(1, "a"), (2, "b")] {
        graph["body"]["kind"]["parts"][index] = serde_json::json!({
            "key": label, "kind": {"tag": "instantiate", "template": "wrapper",
            "arguments": {"label": {"tag": "literal", "value": label}}}
        });
    }
    graph
}
fn fixture_modules() -> [crate::construction::association::CapturedModule<'static>; 3] {
    [
        crate::construction::association::CapturedModule {
            logical_id: "fixture.ncl",
            bytes: b"candidate source",
        },
        crate::construction::association::CapturedModule {
            logical_id: crate::construction::model::profile::LIBRARY_MODULE,
            bytes: LIBRARY,
        },
        crate::construction::association::CapturedModule {
            logical_id: crate::construction::model::profile::CONTRACTS_MODULE,
            bytes: CONTRACTS,
        },
    ]
}

fn fixture_materials<'a>(
    modules: &'a [crate::construction::association::CapturedModule<'a>],
    observation: &'a [u8],
    digest: &'a str,
) -> crate::construction::association::CompilationMaterials<'a> {
    crate::construction::association::CompilationMaterials {
        modules,
        entry: "fixture.ncl",
        evaluator_identity: "captured-evaluator-build",
        evaluator_profile: crate::construction::model::profile::CAPTURED_EVALUATOR_PROFILE,
        evaluator_binding: observation,
        evaluated_output_blake3: digest,
        library: LIBRARY,
        contracts: CONTRACTS,
        lowerer_identity: "projection-v3-lowerer",
        toolchain_identity: "test-toolchain",
    }
}

mod algebra;
mod capture;
mod composition;
mod denial;
mod execution;
mod readmission;
mod references;
mod traversal;

fn rule(resource: &str) -> crate::construction::model::host::EffectRule {
    crate::construction::model::host::EffectRule {
        from_state: "ready".into(),
        action: "start".into(),
        resource: resource.into(),
        ability: "run".into(),
    }
}
