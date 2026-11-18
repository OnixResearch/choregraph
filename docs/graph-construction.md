# Composable Nickel graph construction (`choregraph.graph-construction.v1`)

This is a versioned, finite *construction* format, not a runtime graph language. Choregraph owns the Nickel helpers, contract, pure native elaboration and machine admission. Lattice is the concrete consumer of the expanded `choregraph.lattice-workflow-projection.v3`; Lattice owns the operation registry selection, actual configuration interpretation, runtime/effects/authority, persistence and UI. The profile does not change the restricted Nickel CLI, `choregraph.evaluated-graph.v1`, or Lattice projections v1/v2.

## Packaging and invocation

The supported evaluator is `nickel-eval.captured-modules.v1`, not the restricted CLI's declared-import closure. The caller supplies immutable UTF-8 Nickel bytes under logical IDs, including **exact** shipped bytes from [`library/nickel/graph-construction-v1.ncl`](../library/nickel/graph-construction-v1.ncl) as `choregraph/graph-construction-v1.ncl` and [`contracts/nickel/graph-construction-contracts-v1.ncl`](../contracts/nickel/graph-construction-contracts-v1.ncl) as `choregraph/graph-construction-contracts-v1.ncl`. The library's sibling import resolves within this captured map. The logical entry is another map member, e.g. `workflow.ncl`, importing `./choregraph/graph-construction-v1.ncl`. The evaluator must report its actual observed resolution closure, module identities and exact exported JSON under its bounded worker policy; the host passes that JSON and opaque evaluator observation with the same captured materials to native compilation. Merely listing imported modules or claiming an expected closure does not prove that evaluation observed them. The restricted CLI neither installs these modules nor loads its declared graph imports before evaluation.

Native entrypoint: `construction::compile_json(candidate: &[u8], registry: &model::host::TargetRegistry, materials: &association::CompilationMaterials<'_>, limits: model::profile::Limits) -> Result<Output, Failure>`, under `choregraph_nickel`. `association::validate_compilation_binding` recomputes the full output and binding against candidate, selected registry, materials and limits. The host supplies `evaluated_output_blake3`, entry, evaluator identity/profile and observation bytes, exact module bytes, library and contract bytes, lowerer and toolchain identities. Native binding includes the candidate, captured source and observed evaluator closure, parameters, selected registry implementation/signatures/rules, elaborator, expanded graph, v3 projection, admitted machine and origins. A receipt binds selected observations, not arbitrary Nickel semantics or source intent. Never accept author-selected `registry` alone as registry authority.

Persisted or materialized graphs use `construction::admission::lower_projection_json(projection: &[u8], registry: &model::host::TargetRegistry, limits: model::profile::Limits) -> Result<admission::ProjectionOutput, Failure>`. This entrypoint requires the explicit v3 schema and current host-selected registry, rechecks typed ordered signatures, implementations, structured configuration and concrete bindings, combines data-induced and explicit control dependencies, and calls existing `AdmittedMachine::admit_v2`. It does not accept a prior receipt as authority and does not reevaluate the author program. It returns the validated projection, existing admitted machine and canonical concrete-machine artifact bytes. A materialized edit may supply an empty `interfaces` array when construction boundaries are unavailable; supplied interfaces must resolve to existing ports with exact direction/type.

The v3 projection retains `interfaces: [{scope,inputs,outputs}]`; each ordered boundary is `{name,value_type,ports}`. These bounded maps retain nested instance exposures and resolve structural identity junctions to real numeric ports. Node port arrays preserve registry input order followed by output order. Construction-generated maps are total for expanded fragments; materialized maps are diagnostic authoring information, not executable authority.

Native material validation also checks the captured binding version/profile/policy, submitted-map digest, exact entry and observed module byte identities, sorted resolution edges and reachable closure, and exact output length/BLAKE3. It binds all cohort/limits receipt bytes; the host must still validate their expected values against its selected nickel-eval request. Correspondence cannot independently establish that an evaluator observed a closure. Unit fixtures use explicitly synthetic host evidence; the actual captured worker qualification belongs to nickel-eval and consumer integration.

### Rust API ownership

`construction::{compile_json, Output, Failure}` is the compilation boundary.
`construction::association` owns captured modules, compilation materials and binding validation;
`construction::admission` owns current-registry projection readmission.
The `construction::model` hub exposes responsibility-specific data modules:
`program` (candidates, templates and attribution), `interface` (ordered boundaries and wiring),
`term` (expressions, references and declared bindings), `host` (selected registries),
`profile` (exact schemas, shipped modules and bounds), and `projection` (v3 expansion and origins).
There are no flat compatibility reexports. Rust namespace changes preserve the wire keys and
serialized meaning, but source-bound identities still change with the selected source bytes.


## Authoring API

`let g = import "./choregraph/graph-construction-v1.ncl" in` yields pure curried constructors. Export the **result** of `g.finish root target registry parameters templates body`, not the module's function record. `finish` directly applies `ConstructionCandidate` to the output value; the contract checks every required field, nested fragment, expression, binding, handle, effect and literal JSON. It has no unused lazy validation field. Nickel's JSON export must force that contracted value; native admission independently parses closed DTOs, checks semantic rules and enforces limits.

| Helper | Arguments in order | Result |
|---|---|---|
| `operation` | `key operation configuration effects` | operation fragment |
| `template` | `name parameters body` | reusable template declaration |
| `instantiate` | `key template arguments` | new scoped instance of a named template |
| `input`, `output` | `root instance boundary` | typed-direction handle to an immediate wire child |
| `connect` | `source target` | directed output-to-input connection |
| `expose` | `name handle` | public input/output exposure |
| `identity` | `key boundaries` | structural pass-through, no runtime node |
| `sequence`, `parallel` | `key parts` | ordered composition fragments |
| `wire` | `key parts connections inputs outputs` | explicit child routing and exposed boundaries |
| `literal`, `parameter` | `value` / `name` | structured expression |
| `reference` | `direction name` | operation-local expression reference |
| `apply` | `operator arguments` | structured operator expression |
| `exact_binding` | `profile identity` | immutable content reference |
| `slot_binding` | `slot direction name` | operation-local slot source |
| `with_origin` | `fragment origin` | supplied attribution, optional |
| `generate` | `cap known_finite_array builder` | bounded `std.array.map` of author-known data |

`generate` contracts the array and cap **before** mapping: integral `0 <= cap <= 4096`, array length `<= cap`; its builder is not a runtime loop. The native compiler separately budgets expanded nodes, ports, edges, effects, instances, depth, boundaries, identifier/origin/parameter bytes and output bytes *before* expansion/allocation. It does not trust the author's cap or Nickel's contract as its resource admission.

The exact exported candidate shape is `{schema="choregraph.graph-construction.v1",root,target,registry,parameters,templates,body}`. A template is `{name,parameters,body}`. A fragment is `{key,kind}` or `{key,origin,kind}`: **`kind` is a nested record** with its own `tag`, not a flat fragment tag. The variants are `{tag="operation",operation,configuration,effects}`, `{tag="instantiate",template,arguments}`, `{tag="identity",boundaries}`, `{tag="sequence",parts}`, `{tag="parallel",parts}` and `{tag="wire",parts,connections,inputs,outputs}`. `configuration` and `arguments` are records of structured expressions. Identity has ordered `{name,value_type}` boundaries. A handle is `{root,instance,direction,boundary}` with `direction="input"|"output"`; `instance` names an immediate wire child key, not a private descendant. Connections are `{source,target}`; exposures are `{name,handle}`. Fragment keys and locally declared identifiers exclude `/`; generated nested public boundary names may contain `/`. Keys within siblings and boundary names are unique. Reusing an existing child handle references that child; only `instantiate` creates a new instance.

A closed graph example (registry-dependent `source` and `sink` signatures):

```nickel
let g = import "./choregraph/graph-construction-v1.ncl" in
let source = g.operation "src" "source" {} [] in
let sink = g.operation "dst" "sink" {} [] in
g.finish "example" "lattice-test" "registry-v1" {} []
  (g.wire "body" [source, sink]
    [g.connect (g.output "example" "src" "out")
               (g.input "example" "dst" "in")]
    [] [])
```

The sample is illustrative, not an evaluated fixture or a claim those operation signatures are selected. Root inputs/outputs must be closed for machine admission. `identity` is structural and cannot create a runtime source/sink by itself.

## Semantics and selected registry

`TargetRegistry` has schema `choregraph.target-registry.v1`, selected `target`, `profile`, exact implementation identity, named `types` and `operations`. Each `ValueType` names `copyable` and `droppable` flags; explicit `Unit` is required. Every operation signature binds an exact implementation, ordered unique typed input/output boundaries, configuration fields (`name`, JSON value type, allowed local references and operator names) and exact effect-rule tuples (`from_state`, `action`, `resource`, `ability`). Exact type equality is the initial compatibility rule; no implicit coercion. Duplicate use of a noncopyable output, dropping a nondroppable output, wrong direction, missing ports, type mismatch, duplicate bindings, unknown registry operation, bad configuration expression, or mismatched effect rule is rejected. Effects have `{key,from_state,action,resource,ability,request,payload}` with each binding either `{mode="exact",reference={profile,identity}}` or `{mode="slot",slot,source={direction,name}}`. Exact references remain exact; scoped slot and declaration keys and local port/config references are rewritten across instances. No author purity label bypasses registry declarations.

Expressions are structured JSON records: `{tag="literal",value=<JSON>}`, `{tag="parameter",name}`, `{tag="reference",reference={direction="input"|"output"|"config"|"invocation",name}}`, `{tag="apply",operator,arguments=[...]}`. Literal strings are literal data, not parsed as Nickel expression source. Registry contracts select allowed operators and reference use for each configuration field. Lattice must revalidate **actual** elaborated configuration expressions and effect bindings against its selected implementation; a Choregraph registry identity is not a runtime permission.

If a Lattice implementation interprets a literal string as another expression language, Lattice must parse that **actual** materialized string and reject foreign-node/port references not represented by the structured remapping/binding set. A `literal` tag is not permission to bypass consumer expression checking. No general textual substitution or opaque expression-reference renaming is provided.

Every `OperationSignature` explicitly declares ordered `invocation: [{name,value_type}]` roles; `ExpandedNode.invocation` retains the exact selected contract. These are host invocation reference sources, **not ordinary data ports**: they have no numeric port ID, cannot be wired through a boundary handle, and do not manufacture data or readiness edges. `LocalReference {direction:"invocation",name}` resolves only a declared role; its expanded expression uses the owning numeric node and `port:null`. Slot bindings become `node/N/invocation/<role>` with an occurrence-scoped slot ID. Lattice's initial real target uses approved `dispatch-request` and `resolved-payload` roles, resolved from its **actual host-generated** canonical dispatch request and payload references, never by echoing an expected exact identity. A candidate cannot invent roles or relabel arbitrary stdin/config data as an invocation request. The registry names/types/usage contract and actual role interpretation remain host-selected.

Scoped declaration keys, slots and node/port/configuration/invocation source references are renamed; exact effect `resource` and `ability` strings may identify intentionally shared external resources and remain unchanged. Exact content references also remain exact and must be compared with actual runtime facts by the consumer.

A template instance receives a deterministic root/nested-key scope. Native elaboration fully renames nodes, ports, effects, slots, configuration and expression references, preserving exact content-addressed references. Numeric node and port allocation follows sorted scoped paths; machine node names are `lattice-node/N`, operations are `lattice/<key>`. Diagnostic origins carry element, template, instance path, local key, optional generation and source location; origins do not define semantic identity. Attribution is author-supplied `{template,generation,location}`; a location `{logical_module,start,end}` is meaningful only if validated as in-range UTF-8 byte boundaries against the exact captured module bytes. Missing locations remain unavailable, never invented.

`sequence` matches ordered output/input types and counts between parts and adds all-to-all exit-to-entry **control dependencies** between nonempty operations, including Unit and no-data steps. Empty identities propagate topology without a runtime identity node. `parallel` concatenates ordered boundaries with `part.key + "/" + boundary` names; the registry's copy/drop rules still apply. `wire` resolves only its immediate children and provides explicit routing and exposures. Data edges and control edges are separate in v3, then lower to existing machine dependencies; no new kernel readiness algorithm. A wiring cycle, including control/data cycles, is rejected. Bare `Unit` is an explicit declared type, not an implicit untyped edge.

Limits are conservative admission ceilings, not promises that every graph below every individual count succeeds. Serialization writes through a bounded writer; expanded configuration/parameter copies are charged before copying. Before invoking the existing non-streaming canonical artifact encoder, native admission reserves `4 * projection_json_bytes + 4096` within the selected output cap, then checks the actual artifact and aggregate returned serialized bytes. Resource rejection returns no output/machine/artifact. This does not establish a separate formal allocation theorem for the encoder.

## Laws, premises and non-claims

A pure observer compares ordered typed routing and registry operation/configuration/effect meaning, plus the **union** of data-induced and explicit control-node dependencies, modulo consistent scoped internal renaming. Redundant data/control representations of one node dependency do not differ under this observer. Diagnostic origins, intermediate interface records and numeric allocation are not operation meaning. Under equal selected registry and limits, closed well-typed fragments with disjoint keys/scopes, no aliasing or conflicting effects, and no intervening failing admission:

- Identity is a neutral *structural* connection where port types and linear usage permit it; an empty identity does not add a machine node.
- Sequence association is observational for admitted three-part compositions only if the same ordering and all-to-all barriers are preserved after regrouping. No general interchange between sequence and parallel is claimed: grouping changes dependency structure when barrier edges appear.
- Parallel association is observational up to deterministic boundary qualification only when the public renamed boundaries and explicit wiring are equivalent; unrelated metadata/origin differences do not establish graph equivalence.
- Two instances of one template produce disjoint scoped graph material, not shared runtime operations; reuse of a handle to one instance may share its output only when its type is copyable.
- Explicit ordered exposure arrays implement boundary permutations. Applying a permutation and its inverse preserves selected port correspondence; changing only the exposure order without adapting consumers is distinguishable.

The mathematical argument is structural and conditional, distinct from executed tests or a machine-checked theorem. An identity introduces only a junction; eliminating that junction contracts a directed path without changing its source, selected destination, type or required node dependency. Regrouping sequential composition creates the same adjacent output/input pairs and exit-to-entry barriers; empty fragments carry the last nonempty exit/next nonempty entry, so do not erase Unit ordering. Parallel regrouping preserves the disjoint node/routing union after an explicit boundary-name bijection. A boundary permutation followed by its inverse is the identity bijection on ordered endpoints. Sequential/parallel interchange is supported only when the selected wiring and all added barrier dependencies are equal under the observer (for example barriers already present); independence alone does not make an all-to-all barrier disappear. Registry copy/drop/effect premises and successful admission are necessary in every argument.

These are conditional comparison premises and test obligations, **not** proven source-level equational theorems. Existing Verus kernel proofs apply to the admitted machine's transition relation; they do not prove Nickel evaluation, captured-module resolver correctness, construction elaboration equivalence, consumer routing equivalence, effect authorization, external exactly-once behavior or runtime scheduler safety. Choregraph cannot confer Lattice authority.

## Adoption and evidence

Choregraph maintainers own source contracts, library bytes, native elaborator/binding and positive/negative fixtures. nickel-eval maintainers own the actual closed captured resolver and bounded worker observation. Lattice maintainers select and version their real registry implementation, provide captured immutable source/cohort and verify evaluator binding, revalidate v3 projection and actual expressions, perform controlled import/native admission, and retain consumer authority. Adoption proceeds only after maintained contract/evaluator/native positive and negative fixtures, origin/limit/stale-binding controls, kernel proof applicability and Lattice import/runtime integration are exercised by the responsible owners. Native unit fixtures, actual-export checks and captured-worker/consumer qualification are separate evidence surfaces; none alone establishes downstream adoption, publication or release readiness.

<!-- r[impl choregraph.construction.laws] -->
<!-- r[impl choregraph.construction.validation] -->

The focused actual-export integration test is explicitly ignored in ordinary Cargo runs rather than silently reporting success without evaluator output. The maintained `graph-construction` Nix check supplies the exact export directory and runs ignored tests with `--include-ignored`; missing export bytes then fail closed. Native unit fixture receipts remain explicitly synthetic and never constitute captured-worker observation.
