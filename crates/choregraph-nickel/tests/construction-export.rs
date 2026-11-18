//! Actual Nickel-exported bytes are read only by this integration boundary, never by src.

const REGISTRY: &str = include_str!("../../../fixtures/nickel/construction/registry.json");
const LIBRARY: &[u8] = include_bytes!("../../../library/nickel/graph-construction-v1.ncl");
const CONTRACTS: &[u8] =
    include_bytes!("../../../contracts/nickel/graph-construction-contracts-v1.ncl");

trait Required<T> {
    fn required(self, context: &str) -> Result<T, String>;
}
impl<T, E: std::fmt::Debug> Required<T> for Result<T, E> {
    fn required(self, context: &str) -> Result<T, String> {
        self.map_err(|error| format!("{context}: {error:?}"))
    }
}

/// Inputs shared by every exported fixture; the test selects the explicit limits.
struct Exports<'a> {
    directory: &'a std::path::Path,
    registry: &'a choregraph_nickel::construction::model::host::TargetRegistry,
    limits: choregraph_nickel::construction::model::profile::Limits,
}

fn captured_modules<'a>(
    entry: &'a str,
    source: &'a [u8],
) -> [choregraph_nickel::construction::association::CapturedModule<'a>; 3] {
    [
        choregraph_nickel::construction::association::CapturedModule {
            logical_id: entry,
            bytes: source,
        },
        choregraph_nickel::construction::association::CapturedModule {
            logical_id: choregraph_nickel::construction::model::profile::LIBRARY_MODULE,
            bytes: LIBRARY,
        },
        choregraph_nickel::construction::association::CapturedModule {
            logical_id: choregraph_nickel::construction::model::profile::CONTRACTS_MODULE,
            bytes: CONTRACTS,
        },
    ]
}

/// A structurally valid synthetic host receipt lets the native compiler exercise the
/// exact Nickel-exported bytes. It does not claim captured-worker observation.
fn synthetic_observation(
    modules: &[choregraph_nickel::construction::association::CapturedModule<'_>],
    entry: &str,
    source: &[u8],
    candidate: &[u8],
) -> Result<Vec<u8>, String> {
    let source_blake3 = blake3::hash(source).to_hex().to_string();
    serde_json::to_vec(&serde_json::json!({
        "version": "nickel-eval.captured-binding.v1",
        "profile": choregraph_nickel::construction::model::profile::CAPTURED_EVALUATOR_PROFILE,
        "policy": "nickel-eval.logical-relative-nickel-json.v1",
        "entry": {"name": entry, "source_bytes": source.len(), "source_blake3": source_blake3},
        "submitted_map_blake3": choregraph_nickel::construction::association::captured_map_blake3(modules),
        "observed": [{"name": entry, "source_bytes": source.len(), "source_blake3": source_blake3}],
        "edges": [], "cohort": {}, "limits": {},
        "output_bytes": candidate.len(), "output_blake3": blake3::hash(candidate).to_hex().to_string()
    }))
    .required("synthetic test receipt")
}

/// `Err` is a harness failure; the inner result is the native compiler's verdict.
fn compile_export(
    exports: &Exports<'_>,
    name: &str,
    source: &[u8],
    negative: bool,
) -> Result<
    Result<choregraph_nickel::construction::Output, choregraph_nickel::construction::Failure>,
    String,
> {
    let file = if negative {
        exports
            .directory
            .join("negative")
            .join(format!("{name}.json"))
    } else {
        exports.directory.join(format!("{name}.json"))
    };
    let candidate =
        std::fs::read(&file).required("the Nix rail must evaluate the exact Nickel fixture")?;
    // Rejection controls must reach native admission rather than fail as malformed exports.
    let exported: serde_json::Value =
        serde_json::from_slice(&candidate).required("the Nix rail must export JSON")?;
    assert!(
        exported.is_object(),
        "{name}: the Nix rail must export one graph record"
    );
    let entry = format!("{name}.ncl");
    let modules = captured_modules(&entry, source);
    let observation = synthetic_observation(&modules, &entry, source, &candidate)?;
    let digest = blake3::hash(&candidate).to_hex().to_string();
    let materials = choregraph_nickel::construction::association::CompilationMaterials {
        modules: &modules,
        entry: &entry,
        evaluator_identity: "nix-actual-nickel-export",
        evaluator_profile:
            choregraph_nickel::construction::model::profile::CAPTURED_EVALUATOR_PROFILE,
        evaluator_binding: &observation,
        evaluated_output_blake3: &digest,
        library: LIBRARY,
        contracts: CONTRACTS,
        lowerer_identity: "construction-test-lowerer",
        toolchain_identity: "nix-test-toolchain",
    };
    let output = choregraph_nickel::construction::compile_json(
        &candidate,
        exports.registry,
        &materials,
        exports.limits,
    );
    if let Ok(admitted) = &output {
        let rebuild = |bytes: &[u8]| {
            choregraph_nickel::construction::association::validate_compilation_binding(
                admitted,
                bytes,
                exports.registry,
                &materials,
                exports.limits,
            )
        };
        rebuild(&candidate).required("exact exported candidate remains bound to its materials")?;
        let mut altered = candidate.clone();
        altered.push(b'\n');
        assert!(
            rebuild(&altered).is_err(),
            "{name}: the binding must reject different exported bytes"
        );
    }
    Ok(output)
}

#[test]
#[ignore = "requires exact Nickel exports supplied by the graph-construction Nix rail"]
fn actual_nickel_exports_compile_and_native_controls_reject() -> Result<(), String> {
    // Workspace cargo tests have no evaluator output; the separate graph-construction
    // Nix check provides this directory and thus exercises the actual evaluator path.
    let directory = std::env::var("CHOREGRAPH_CONSTRUCTION_EXPORTED_DIR")
        .required("actual evaluator export directory")?;
    let registry: choregraph_nickel::construction::model::host::TargetRegistry =
        serde_json::from_str(REGISTRY).required("registry fixture")?;
    let exports = Exports {
        directory: std::path::Path::new(&directory),
        registry: &registry,
        limits: choregraph_nickel::construction::model::profile::Limits::default(),
    };
    let cases: [(&str, &[u8], usize, usize, usize); 5] = [
        (
            "nested-twice",
            include_bytes!("../../../fixtures/nickel/construction/positive/nested-twice.ncl"),
            4,
            4,
            0,
        ),
        (
            "generated-fork-join",
            include_bytes!(
                "../../../fixtures/nickel/construction/positive/generated-fork-join.ncl"
            ),
            4,
            4,
            0,
        ),
        (
            "unit-sequence",
            include_bytes!("../../../fixtures/nickel/construction/positive/unit-sequence.ncl"),
            3,
            0,
            2,
        ),
        (
            "effectful-sequence",
            include_bytes!("../../../fixtures/nickel/construction/positive/effectful-sequence.ncl"),
            2,
            0,
            1,
        ),
        (
            "invocation-sequence",
            include_bytes!(
                "../../../fixtures/nickel/construction/positive/invocation-sequence.ncl"
            ),
            2,
            0,
            1,
        ),
    ];
    for (name, source, nodes, edges, controls) in cases {
        let output = compile_export(&exports, name, source, false)?
            .map_err(|error| format!("actual Nickel export {name} rejected: {error:?}"))?;
        assert_eq!(output.projection.nodes.len(), nodes, "{name}");
        assert_eq!(output.projection.edges.len(), edges, "{name}");
        assert_eq!(output.projection.control_edges.len(), controls, "{name}");
        if name == "effectful-sequence" {
            assert_eq!(output.projection.effects.len(), 2);
            assert_ne!(
                output.projection.effects[0].declaration_key,
                output.projection.effects[1].declaration_key
            );
            assert_ne!(
                output.projection.effects[0].payload_binding,
                output.projection.effects[1].payload_binding
            );
            assert_eq!(
                output.projection.effects[0].request_binding,
                output.projection.effects[1].request_binding
            );
        }
    }
    for (name, source) in [
        (
            "linear-copy",
            &include_bytes!(
                "../../../fixtures/nickel/construction/negative/native/linear-copy.ncl"
            )[..],
        ),
        (
            "linear-drop",
            &include_bytes!(
                "../../../fixtures/nickel/construction/negative/native/linear-drop.ncl"
            )[..],
        ),
        (
            "wrong-type",
            &include_bytes!("../../../fixtures/nickel/construction/negative/native/wrong-type.ncl")
                [..],
        ),
        (
            "cycle",
            &include_bytes!("../../../fixtures/nickel/construction/negative/native/cycle.ncl")[..],
        ),
    ] {
        assert!(
            compile_export(&exports, name, source, true)?.is_err(),
            "Nickel-exported native rejection control {name} unexpectedly admitted"
        );
    }
    Ok(())
}
