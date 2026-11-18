pub(super) fn origin(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    element: String,
    path: &[String],
    label: crate::construction::expand::OriginLabel<'_>,
    attribution: Option<&crate::construction::model::program::Attribution>,
) -> Result<(), crate::construction::Failure> {
    let (template, generation, location, supplied) = if let Some(attribution) = attribution {
        crate::construction::identifier(&attribution.template, expansion.limits)?;
        if let Some(generation) = &attribution.generation {
            crate::construction::identifier(generation, expansion.limits)?;
        }
        if let Some(location) = &attribution.location {
            location_valid(expansion, location)?;
        }
        (
            attribution.template.as_str(),
            attribution.generation.as_deref(),
            attribution.location.as_ref(),
            true,
        )
    } else {
        (label.template, None, None, false)
    };
    #[derive(serde::Serialize)]
    struct View<'b> {
        element: &'b str,
        template: &'b str,
        instance_path: &'b [String],
        local: &'b str,
        generation: Option<&'b str>,
        location: Option<&'b crate::construction::model::program::SourceLocation>,
        supplied: bool,
    }
    let view = View {
        element: &element,
        template,
        instance_path: path,
        local: label.local,
        generation,
        location,
        supplied,
    };
    let bytes = crate::construction::bounded_json(
        &view,
        expansion
            .limits
            .max_origin_bytes
            .saturating_sub(expansion.origin_bytes),
    )?;
    crate::construction::expand::take(
        &mut expansion.origin_bytes,
        bytes.len(),
        crate::construction::expand::Ceiling(expansion.limits.max_origin_bytes),
    )?;
    debug_assert!(
        generation.is_none_or(|value| value.len() <= expansion.limits.max_identifier_bytes)
    );
    debug_assert!(expansion.origin_bytes <= expansion.limits.max_origin_bytes);
    expansion
        .origins
        .push(crate::construction::model::projection::Origin {
            element,
            template: template.to_owned(),
            instance_path: path.to_vec(),
            local: label.local.to_owned(),
            generation: generation.map(str::to_owned),
            location: location.cloned(),
            supplied,
        });
    Ok(())
}

pub(super) fn endpoint(
    expansion: &mut crate::construction::expand::Expansion<'_>,
    path: &[String],
    label: crate::construction::expand::EndpointLabel<'_>,
    real: Option<(
        String,
        crate::construction::model::interface::Direction,
        String,
    )>,
) -> Result<String, crate::construction::Failure> {
    if expansion.endpoints.len() >= expansion.limits.max_ports {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    if !expansion.registry.types.contains_key(label.value_type) {
        return Err(crate::construction::Failure::IncompatibleType);
    }
    let id = crate::construction::expand::scoped(path, label.local)?;
    if id.len() > expansion.limits.max_identifier_bytes {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    debug_assert!(id.len() <= expansion.limits.max_identifier_bytes);
    if expansion
        .endpoints
        .insert(
            id.clone(),
            crate::construction::expand::Endpoint {
                value_type: label.value_type.to_owned(),
                real,
            },
        )
        .is_some()
    {
        return Err(crate::construction::Failure::DuplicateKey);
    }
    debug_assert!(expansion.endpoints.len() <= expansion.limits.max_ports);
    Ok(id)
}

fn location_valid(
    expansion: &crate::construction::expand::Expansion<'_>,
    location: &crate::construction::model::program::SourceLocation,
) -> Result<(), crate::construction::Failure> {
    let module = expansion
        .modules
        .iter()
        .find(|module| module.logical_id == location.logical_module)
        .ok_or(crate::construction::Failure::InvalidLocation)?;
    let text = std::str::from_utf8(module.bytes)
        .map_err(|_| crate::construction::Failure::InvalidLocation)?;
    let is_ordered = location.start <= location.end;
    let is_in_bounds = location.end <= text.len();
    let is_aligned = text.is_char_boundary(location.start) && text.is_char_boundary(location.end);
    if !is_ordered || !is_in_bounds || !is_aligned {
        return Err(crate::construction::Failure::InvalidLocation);
    }
    Ok(())
}
