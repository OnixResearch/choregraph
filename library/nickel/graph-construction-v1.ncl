# Construction is finite evaluated data. Export only the result of `finish`, not this module.
# In the captured profile this file and its sibling contract are immutable module bytes.
let Contracts = import "graph-construction-contracts-v1.ncl" in
{
  operation = fun fragment_key operation_name fields effect_list => {
    key = fragment_key,
    kind = { tag = "operation", operation = operation_name,
      configuration = fields, effects = effect_list },
  },
  template = fun template_name parameter_names template_body => {
    name = template_name, parameters = parameter_names, body = template_body,
  },
  instantiate = fun fragment_key template_name argument_values => {
    key = fragment_key, kind = { tag = "instantiate", template = template_name, arguments = argument_values },
  },
  input = fun root_name instance_key boundary_name => {
    root = root_name, instance = instance_key, direction = "input", boundary = boundary_name,
  },
  output = fun root_name instance_key boundary_name => {
    root = root_name, instance = instance_key, direction = "output", boundary = boundary_name,
  },
  connect = fun source_handle target_handle => { source = source_handle, target = target_handle },
  expose = fun boundary_name boundary_handle => { name = boundary_name, handle = boundary_handle },
  identity = fun fragment_key boundary_list => {
    key = fragment_key, kind = { tag = "identity", boundaries = boundary_list },
  },
  sequence = fun fragment_key child_parts => { key = fragment_key, kind = { tag = "sequence", parts = child_parts } },
  parallel = fun fragment_key child_parts => { key = fragment_key, kind = { tag = "parallel", parts = child_parts } },
  wire = fun fragment_key child_parts child_connections public_inputs public_outputs => {
    key = fragment_key, kind = { tag = "wire", parts = child_parts, connections = child_connections,
      inputs = public_inputs, outputs = public_outputs },
  },
  literal = fun literal_value => { tag = "literal", value = literal_value },
  parameter = fun parameter_name => { tag = "parameter", name = parameter_name },
  reference = fun ref_direction ref_name => {
    tag = "reference", reference = { direction = ref_direction, name = ref_name },
  },
  apply = fun operator_name argument_values => {
    tag = "apply", operator = operator_name, arguments = argument_values,
  },
  exact_binding = fun binding_profile reference_identity => {
    mode = "exact", reference = { profile = binding_profile, identity = reference_identity },
  },
  slot_binding = fun slot_name ref_direction ref_name => {
    mode = "slot", slot = slot_name, source = { direction = ref_direction, name = ref_name },
  },
  with_origin = fun fragment origin_info => fragment & { origin = origin_info },
  # The cap checks the finite input array before map allocates generated fragments.
  # Native limits independently count the expanded graph before allocation.
  generate = fun generation_cap finite_items builder =>
    let bounded = { cap = generation_cap, items = finite_items } | Contracts.BoundedGeneration in
    std.array.map builder bounded.items,
  finish = fun root_name target_name registry_name parameter_values template_values root_body =>
    ({
      schema = "choregraph.graph-construction.v1",
      root = root_name, target = target_name, registry = registry_name,
      parameters = parameter_values, templates = template_values, body = root_body,
    } | Contracts.ConstructionCandidate),
}
