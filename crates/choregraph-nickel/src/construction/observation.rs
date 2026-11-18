//! Exact evaluator receipt structure and submitted-module validation.

fn string<'a>(
    value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a str, crate::construction::Failure> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or(crate::construction::Failure::InvalidMaterials)
}
fn array<'a>(
    value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a [serde_json::Value], crate::construction::Failure> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .ok_or(crate::construction::Failure::InvalidMaterials)
}
fn matches_header(
    receipt: &serde_json::Value,
    materials: &crate::construction::association::CompilationMaterials<'_>,
) -> Result<(), crate::construction::Failure> {
    profiles(receipt)?;
    if string(receipt, "submitted_map_blake3")?
        != crate::construction::association::captured_map_blake3(materials.modules)
        || string(receipt, "output_blake3")? != materials.evaluated_output_blake3
    {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(())
}
fn profiles(receipt: &serde_json::Value) -> Result<(), crate::construction::Failure> {
    for (field, expected) in [
        ("version", "nickel-eval.captured-binding.v1"),
        (
            "profile",
            crate::construction::model::profile::CAPTURED_EVALUATOR_PROFILE,
        ),
        ("policy", "nickel-eval.logical-relative-nickel-json.v1"),
    ] {
        if string(receipt, field)? != expected {
            return Err(crate::construction::Failure::InvalidMaterials);
        }
    }
    Ok(())
}
fn module<'a>(
    value: &'a serde_json::Value,
    materials: &crate::construction::association::CompilationMaterials<'_>,
) -> Result<&'a str, crate::construction::Failure> {
    let name = string(value, "name")?;
    let source = materials
        .modules
        .iter()
        .find(|module| module.logical_id == name)
        .ok_or(crate::construction::Failure::InvalidMaterials)?;
    let observed_bytes = value
        .get("source_bytes")
        .and_then(serde_json::Value::as_u64);
    let source_bytes = u64::try_from(source.bytes.len()).ok();
    if observed_bytes != source_bytes
        || value
            .get("source_blake3")
            .and_then(serde_json::Value::as_str)
            != Some(blake3::hash(source.bytes).to_hex().as_str())
    {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    debug_assert_eq!(source.logical_id, name);
    debug_assert_eq!(observed_bytes, source_bytes);
    Ok(name)
}
fn edges<'a>(
    receipt: &'a serde_json::Value,
    observed: &[&str],
) -> Result<Vec<(&'a str, &'a str)>, crate::construction::Failure> {
    let rows = array(receipt, "edges")?;
    let mut edges = Vec::with_capacity(rows.len());
    for edge in rows {
        let from = string(edge, "from")?;
        let to = string(edge, "to")?;
        if observed.binary_search(&from).is_err() || observed.binary_search(&to).is_err() {
            return Err(crate::construction::Failure::InvalidMaterials);
        }
        edges.push((from, to));
    }
    if edges.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(edges)
}
fn reachable(
    entry: &str,
    observed: &[&str],
    edges: &[(&str, &str)],
) -> Result<(), crate::construction::Failure> {
    let mut reached = std::collections::BTreeSet::from([entry]);
    while extend_reached(&mut reached, observed, edges)? {}
    if observed.iter().any(|name| !reached.contains(name)) {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(())
}
fn extend_reached<'a>(
    reached: &mut std::collections::BTreeSet<&'a str>,
    observed: &[&str],
    edges: &[(&'a str, &'a str)],
) -> Result<bool, crate::construction::Failure> {
    let before = reached.len();
    for (from, to) in edges {
        if reached.contains(from) && !reached.contains(to) {
            if reached.len() >= observed.len() {
                return Err(crate::construction::Failure::InvalidMaterials);
            }
            reached.insert(*to);
        }
    }
    Ok(reached.len() != before)
}
fn receipt(
    candidate: &[u8],
    materials: &crate::construction::association::CompilationMaterials<'_>,
) -> Result<(), crate::construction::Failure> {
    let receipt: serde_json::Value = serde_json::from_slice(materials.evaluator_binding)
        .map_err(|_| crate::construction::Failure::InvalidMaterials)?;
    matches_header(&receipt, materials)?;
    receipt_shape(&receipt, candidate.len())?;
    let entry = receipt
        .get("entry")
        .ok_or(crate::construction::Failure::InvalidMaterials)?;
    if module(entry, materials)? != materials.entry {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    let observed = observed(&receipt, materials)?;
    reachable(materials.entry, &observed, &edges(&receipt, &observed)?)
}
fn observed<'a>(
    receipt: &'a serde_json::Value,
    materials: &crate::construction::association::CompilationMaterials<'_>,
) -> Result<Vec<&'a str>, crate::construction::Failure> {
    let observed = array(receipt, "observed")?
        .iter()
        .map(|value| module(value, materials))
        .collect::<Result<Vec<_>, _>>()?;
    if observed.windows(2).any(|pair| pair[0] >= pair[1]) || !observed.contains(&materials.entry) {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(observed)
}
fn receipt_shape(
    receipt: &serde_json::Value,
    candidate_bytes: usize,
) -> Result<(), crate::construction::Failure> {
    if receipt
        .get("output_bytes")
        .and_then(serde_json::Value::as_u64)
        != u64::try_from(candidate_bytes).ok()
        || !receipt
            .get("cohort")
            .is_some_and(serde_json::Value::is_object)
        || !receipt
            .get("limits")
            .is_some_and(serde_json::Value::is_object)
    {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(())
}
fn material_header(
    candidate: &[u8],
    materials: &crate::construction::association::CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    material_profile(materials)?;
    if materials.modules.len() > limits.max_instances || materials.modules.is_empty() {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    if materials.evaluator_binding.is_empty()
        || materials.evaluator_binding.len() > limits.max_output_bytes
    {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    if materials.evaluated_output_blake3 != blake3::hash(candidate).to_hex().as_str() {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(())
}
fn material_profile(
    materials: &crate::construction::association::CompilationMaterials<'_>,
) -> Result<(), crate::construction::Failure> {
    if materials.evaluator_profile
        != crate::construction::model::profile::CAPTURED_EVALUATOR_PROFILE
        || materials.library != crate::construction::model::profile::LIBRARY_BYTES
        || materials.contracts != crate::construction::model::profile::CONTRACTS_BYTES
    {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(())
}
fn valid_module(
    module: &crate::construction::association::CapturedModule<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    crate::construction::identifier(module.logical_id, limits)?;
    if !logical_module(module.logical_id) {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    if std::str::from_utf8(module.bytes).is_err() || module.bytes.len() > limits.max_candidate_bytes
    {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    Ok(())
}
fn logical_module(value: &str) -> bool {
    !value.starts_with('/')
        && !value.contains(':')
        && value
            .split('/')
            .all(|part| !matches!(part, "" | "." | ".."))
}
fn submitted<'a>(
    materials: &'a crate::construction::association::CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<std::collections::BTreeMap<&'a str, &'a [u8]>, crate::construction::Failure> {
    let mut names = std::collections::BTreeMap::new();
    let mut total = 0usize;
    for module in materials.modules {
        valid_module(module, limits)?;
        if names.len() >= limits.max_instances
            || names.insert(module.logical_id, module.bytes).is_some()
        {
            return Err(crate::construction::Failure::InvalidMaterials);
        }
        module_bytes(&mut total, module, limits)?;
    }
    Ok(names)
}
fn module_bytes(
    total: &mut usize,
    module: &crate::construction::association::CapturedModule<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    *total = total
        .checked_add(module.bytes.len())
        .ok_or(crate::construction::Failure::LimitExceeded)?;
    if *total > limits.max_output_bytes {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    Ok(())
}
pub(super) fn validate_materials(
    candidate: &[u8],
    materials: &crate::construction::association::CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    material_header(candidate, materials, limits)?;
    material_identifiers(materials, limits)?;
    let names = submitted(materials, limits)?;
    if !names.contains_key(materials.entry)
        || names
            .get(crate::construction::model::profile::LIBRARY_MODULE)
            .copied()
            != Some(crate::construction::model::profile::LIBRARY_BYTES)
        || names
            .get(crate::construction::model::profile::CONTRACTS_MODULE)
            .copied()
            != Some(crate::construction::model::profile::CONTRACTS_BYTES)
    {
        return Err(crate::construction::Failure::InvalidMaterials);
    }
    receipt(candidate, materials)
}
fn material_identifiers(
    materials: &crate::construction::association::CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    for value in [
        materials.entry,
        materials.evaluator_identity,
        materials.lowerer_identity,
        materials.toolchain_identity,
    ] {
        crate::construction::identifier(value, limits)?;
    }
    Ok(())
}
