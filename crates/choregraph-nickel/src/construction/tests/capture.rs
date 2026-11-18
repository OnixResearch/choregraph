use super::*;

#[test]
fn bounds_and_stale_compilation_materials_fail_closed() {
    let graph = nested();
    let registry = fixture_registry();
    let limits = crate::construction::model::profile::Limits::default();
    let admitted = admitted(&graph, &registry);
    assert_eq!(admitted.projection.nodes.len(), 4);
    for restricted in [
        crate::construction::model::profile::Limits {
            max_candidate_bytes: 10,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_ports: 1,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_edges: 1,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_instances: 3,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_depth: 1,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_origin_bytes: 1,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_parameter_bytes: 1,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_boundaries: 3,
            ..limits
        },
        crate::construction::model::profile::Limits {
            max_output_bytes: 1024,
            ..limits
        },
    ] {
        assert_eq!(
            compile(&graph, &registry, restricted).err(),
            Some(crate::construction::Failure::LimitExceeded)
        );
    }
}

#[test]
fn node_and_identifier_ceilings_fail_closed() {
    let graph = nested();
    let registry = fixture_registry();
    let limits = crate::construction::model::profile::Limits::default();
    let mut small_registry = registry.clone();
    small_registry
        .operations
        .retain(|signature| ["fork", "pass", "join"].contains(&signature.operation.as_str()));
    assert_eq!(
        compile(
            &graph,
            &small_registry,
            crate::construction::model::profile::Limits {
                max_nodes: 3,
                ..limits
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
    assert_eq!(
        compile(
            &graph,
            &registry,
            crate::construction::model::profile::Limits {
                max_identifier_bytes: 2,
                ..limits
            }
        )
        .err(),
        Some(crate::construction::Failure::InvalidIdentifier)
    );
}

#[test]
fn stale_compilation_materials_fail_closed() {
    let graph = nested();
    let registry = fixture_registry();
    let limits = crate::construction::model::profile::Limits::default();
    let output = admitted(&graph, &registry);
    let candidate = serde_json::to_vec(&graph).required("candidate JSON");
    let digest = blake3::hash(&candidate).to_hex().to_string();
    let modules = fixture_modules();
    let observation = synthetic_receipt(&candidate, &modules, "fixture.ncl");
    let materials = fixture_materials(&modules, &observation, &digest);
    crate::construction::association::validate_compilation_binding(
        &output, &candidate, &registry, &materials, limits,
    )
    .required("fresh exact binding");
    let mut modified = candidate.clone();
    modified.push(b' ');
    assert!(
        crate::construction::association::validate_compilation_binding(
            &output, &modified, &registry, &materials, limits
        )
        .is_err()
    );
    let mut changed = registry.clone();
    changed.implementation = "new-implementation".into();
    assert!(
        crate::construction::association::validate_compilation_binding(
            &output, &candidate, &changed, &materials, limits
        )
        .is_err()
    );
    let mut changed_observation: serde_json::Value =
        serde_json::from_slice(&observation).required("synthetic observation");
    changed_observation["cohort"] = serde_json::json!({"changed": true});
    let changed_observation =
        serde_json::to_vec(&changed_observation).required("changed observation");
    let stale = crate::construction::association::CompilationMaterials {
        evaluator_binding: &changed_observation,
        ..materials
    };
    assert_eq!(
        crate::construction::association::validate_compilation_binding(
            &output, &candidate, &registry, &stale, limits
        )
        .err(),
        Some(crate::construction::Failure::StaleBinding)
    );
}
#[test]
fn explicit_origin_offsets_and_exact_materials_are_checked() {
    let registry = fixture_registry();
    let mut graph = make_candidate(operation(("tick", "tick")));
    graph["body"]["origin"] = serde_json::json!({"template": "authored", "generation": "fixture", "location": {
        "logical_module": "fixture.ncl", "start": 0, "end": 9
    }});
    let compiled = admitted(&graph, &registry);
    assert!(
        compiled
            .origins
            .iter()
            .any(|origin| origin.supplied && origin.location.is_some())
    );
    graph["body"]["origin"]["location"]["end"] = serde_json::json!(999);
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidLocation,
    );
    graph["body"]["origin"]["location"]["end"] = serde_json::json!(9);
    graph["body"]["origin"]["location"]["logical_module"] = serde_json::json!("unsubmitted.ncl");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidLocation,
    );
}

#[test]
fn exact_materials_and_observations_are_checked() {
    let registry = fixture_registry();
    let candidate =
        serde_json::to_vec(&make_candidate(operation(("tick", "tick")))).required("candidate");
    let digest = blake3::hash(&candidate).to_hex().to_string();
    let modules = fixture_modules();
    let observation = synthetic_receipt(&candidate, &modules, "fixture.ncl");
    let selected = fixture_materials(&modules, &observation, &digest);
    for stale in [
        crate::construction::association::CompilationMaterials {
            entry: "missing.ncl",
            ..selected
        },
        crate::construction::association::CompilationMaterials {
            evaluator_profile: "legacy-profile",
            ..selected
        },
        crate::construction::association::CompilationMaterials {
            evaluated_output_blake3: "bad-hash",
            ..selected
        },
        crate::construction::association::CompilationMaterials {
            library: b"changed library",
            ..selected
        },
        crate::construction::association::CompilationMaterials {
            contracts: b"changed contract",
            ..selected
        },
    ] {
        assert_eq!(
            compile_json(
                &candidate,
                &registry,
                &stale,
                crate::construction::model::profile::Limits::default()
            )
            .err(),
            Some(crate::construction::Failure::InvalidMaterials)
        );
    }
    let mut malformed_observation: serde_json::Value =
        serde_json::from_slice(&observation).required("synthetic observation");
    malformed_observation["observed"][0]["source_blake3"] = serde_json::json!("incorrect");
    let malformed_observation =
        serde_json::to_vec(&malformed_observation).required("malformed observation");
    let malformed = crate::construction::association::CompilationMaterials {
        evaluator_binding: &malformed_observation,
        ..selected
    };
    assert_eq!(
        compile_json(
            &candidate,
            &registry,
            &malformed,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidMaterials)
    );
}

#[test]
fn unicode_origin_offsets_must_be_character_boundaries() {
    let registry = fixture_registry();
    let mut graph = make_candidate(operation(("tick", "tick")));
    graph["body"]["origin"] = serde_json::json!({"template": "unicode", "location": {
        "logical_module": "fixture.ncl", "start": 1, "end": 2
    }});
    let mut modules = fixture_modules();
    modules[0].bytes = "μ source".as_bytes();
    assert_eq!(
        compile_modules(
            &graph,
            &registry,
            &modules,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidLocation)
    );
    graph["body"]["origin"]["location"]["start"] = serde_json::json!(0);
    let output = compile_modules(
        &graph,
        &registry,
        &modules,
        crate::construction::model::profile::Limits::default(),
    )
    .required("aligned Unicode origin");
    assert!(output.origins.iter().any(|origin| {
        origin
            .location
            .as_ref()
            .is_some_and(|location| location.start == 0 && location.end == 2)
    }));
}
#[test]
fn source_submitted_import_and_parameter_drift_invalidate_association() {
    let graph = make_candidate(operation(("tick", "tick")));
    let bytes = serde_json::to_vec(&graph).required("candidate");
    let digest = blake3::hash(&bytes).to_hex().to_string();
    let registry = fixture_registry();
    let mut prior: Option<crate::construction::Output> = None;
    for (source, unused) in [
        (b"source-a".as_slice(), b"import-a".as_slice()),
        (b"source-b".as_slice(), b"import-a".as_slice()),
        (b"source-b".as_slice(), b"import-b".as_slice()),
    ] {
        let modules = [
            crate::construction::association::CapturedModule {
                logical_id: "entry.ncl",
                bytes: source,
            },
            crate::construction::association::CapturedModule {
                logical_id: "unused.ncl",
                bytes: unused,
            },
            crate::construction::association::CapturedModule {
                logical_id: crate::construction::model::profile::LIBRARY_MODULE,
                bytes: LIBRARY,
            },
            crate::construction::association::CapturedModule {
                logical_id: crate::construction::model::profile::CONTRACTS_MODULE,
                bytes: CONTRACTS,
            },
        ];
        let evidence = synthetic_receipt(&bytes, &modules, "entry.ncl");
        let materials = crate::construction::association::CompilationMaterials {
            entry: "entry.ncl",
            evaluator_identity: "fixture",
            lowerer_identity: "lowerer",
            toolchain_identity: "toolchain",
            ..fixture_materials(&modules, &evidence, &digest)
        };
        let output = crate::construction::compile_json(
            &bytes,
            &registry,
            &materials,
            crate::construction::model::profile::Limits::default(),
        )
        .required("exact drift candidate");
        if let Some(previous) = prior {
            assert_eq!(
                output.artifact, previous.artifact,
                "diagnostic/source association is not machine meaning"
            );
            assert_ne!(output.binding, previous.binding);
            assert_eq!(
                crate::construction::association::validate_compilation_binding(
                    &previous,
                    &bytes,
                    &registry,
                    &materials,
                    crate::construction::model::profile::Limits::default()
                ),
                Err(crate::construction::Failure::StaleBinding)
            );
        }
        prior = Some(output);
    }
    let mut parameterized = graph.clone();
    parameterized["parameters"] = serde_json::json!({"changed":true});
    assert_ne!(
        admitted(&graph, &registry).binding,
        admitted(&parameterized, &registry).binding
    );
}
