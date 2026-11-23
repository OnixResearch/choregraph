//! Independent synchronous rendezvous MDP enumerator, PRISM `.prism` /
//! `.props` exporter, and PRISM explicit-export conformance verifier.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use choregraph_choreography::endpoint::{LocalProtocol, LocalProtocolSetBody};
use choregraph_choreography::model::participants::{LabelId, PayloadTag, RoleId};

use crate::model::{
    AnalysisError, BranchProbability, ChoiceProbabilityAnnotation, OwnershipAbstraction,
    ProbabilityProvenance, ProbabilitySidecar, PropertySpec, RationalProb, StateInventory,
    StateSpaceBounds, validate_distribution,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum LocalStep {
    End,
    Send {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: usize,
    },
    Receive {
        peer: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: usize,
    },
    InternalChoice {
        choice_ordinal: usize,
        branches: Vec<(LabelId, usize)>,
    },
    Offer {
        from: RoleId,
        branches: Vec<(LabelId, usize)>,
    },
}

struct RoleAutomaton {
    role: RoleId,
    steps: Vec<LocalStep>,
    choice_count: usize,
}

fn flatten_local_protocol(
    role: RoleId,
    protocol: &LocalProtocol,
    abstraction: OwnershipAbstraction,
) -> Result<RoleAutomaton, AnalysisError> {
    let mut steps = Vec::new();
    let mut choice_count: usize = 0;
    let _root = flatten_node(protocol, abstraction, &mut steps, &mut choice_count)?;
    Ok(RoleAutomaton {
        role,
        steps,
        choice_count,
    })
}

fn flatten_node(
    protocol: &LocalProtocol,
    abstraction: OwnershipAbstraction,
    steps: &mut Vec<LocalStep>,
    choice_count: &mut usize,
) -> Result<usize, AnalysisError> {
    let id = steps.len();
    match protocol {
        LocalProtocol::End => {
            steps.push(LocalStep::End);
            Ok(id)
        }
        LocalProtocol::Send {
            peer,
            label,
            payload_tag,
            next,
        } => {
            steps.push(LocalStep::End);
            let next_id = flatten_node(next, abstraction, steps, choice_count)?;
            steps[id] = LocalStep::Send {
                peer: *peer,
                label: *label,
                payload_tag: *payload_tag,
                next: next_id,
            };
            Ok(id)
        }
        LocalProtocol::Receive {
            peer,
            label,
            payload_tag,
            next,
        } => {
            steps.push(LocalStep::End);
            let next_id = flatten_node(next, abstraction, steps, choice_count)?;
            steps[id] = LocalStep::Receive {
                peer: *peer,
                label: *label,
                payload_tag: *payload_tag,
                next: next_id,
            };
            Ok(id)
        }
        LocalProtocol::SendOwned { .. } | LocalProtocol::ReceiveOwned { .. } => match abstraction {
            OwnershipAbstraction::RejectOwnershipTransfer => {
                Err(AnalysisError::UnsupportedConstruct(String::from(
                    "ownership transfer (SendOwned/ReceiveOwned) is unsupported in the synchronous pilot",
                )))
            }
        },
        LocalProtocol::InternalChoice { branches } => {
            let ordinal = *choice_count;
            *choice_count = choice_count.saturating_add(1);
            steps.push(LocalStep::End);
            let mut flattened = Vec::with_capacity(branches.len());
            for branch in branches {
                let next_id = flatten_node(&branch.body, abstraction, steps, choice_count)?;
                flattened.push((branch.label, next_id));
            }
            steps[id] = LocalStep::InternalChoice {
                choice_ordinal: ordinal,
                branches: flattened,
            };
            Ok(id)
        }
        LocalProtocol::Offer { from, branches } => {
            steps.push(LocalStep::End);
            let mut flattened = Vec::with_capacity(branches.len());
            for branch in branches {
                let next_id = flatten_node(&branch.body, abstraction, steps, choice_count)?;
                flattened.push((branch.label, next_id));
            }
            steps[id] = LocalStep::Offer {
                from: *from,
                branches: flattened,
            };
            Ok(id)
        }
    }
}

/// One outgoing MDP transition choice (either a probabilistic distribution or
/// one nondeterministic action with probability 1).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MdpTransition {
    pub action: String,
    pub outcomes: Vec<(usize, RationalProb)>,
}

/// Explicit enumerated synchronous MDP over projected local protocols.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumeratedMdp {
    pub roles: Vec<RoleId>,
    pub states: Vec<Vec<usize>>,
    pub transitions: Vec<Vec<MdpTransition>>,
    pub ended_states: BTreeSet<usize>,
    pub deadlock_states: BTreeSet<usize>,
    /// Maps `(decider, label)` -> set of state indices entered upon choosing `label`.
    pub branch_entry_states: BTreeMap<(RoleId, LabelId), BTreeSet<usize>>,
    pub provenance_classes: Vec<ProbabilityProvenance>,
    pub probabilistic_choices: usize,
    pub nondeterministic_choices: usize,
}

impl EnumeratedMdp {
    #[must_use]
    pub fn inventory(&self) -> StateInventory {
        let transition_count: usize = self.transitions.iter().map(Vec::len).sum();
        StateInventory {
            states: self.states.len(),
            transitions: transition_count,
            probabilistic_choices: self.probabilistic_choices,
            nondeterministic_choices: self.nondeterministic_choices,
            terminal_end_states: self.ended_states.len(),
            deadlock_states: self.deadlock_states.len(),
        }
    }
}

fn index_annotations<'a>(
    automata: &[RoleAutomaton],
    sidecar: &'a ProbabilitySidecar,
) -> Result<BTreeMap<(RoleId, usize), &'a ChoiceProbabilityAnnotation>, AnalysisError> {
    if sidecar.schema != crate::model::QUANTITATIVE_PILOT_SCHEMA {
        return Err(AnalysisError::InvalidSchema(sidecar.schema.clone()));
    }
    let mut map = BTreeMap::new();
    for annotation in &sidecar.choices {
        let key = (annotation.decider, annotation.choice_ordinal);
        if map.insert(key, annotation).is_some() {
            return Err(AnalysisError::UnusedOrUnknownAnnotation(format!(
                "duplicate probability annotation for role {} choice {}",
                annotation.decider.0, annotation.choice_ordinal
            )));
        }
        let Some(automaton) = automata.iter().find(|a| a.role == annotation.decider) else {
            return Err(AnalysisError::UnusedOrUnknownAnnotation(format!(
                "annotation names unknown role {}",
                annotation.decider.0
            )));
        };
        if annotation.choice_ordinal >= automaton.choice_count {
            return Err(AnalysisError::UnusedOrUnknownAnnotation(format!(
                "annotation choice ordinal {} exceeds role {} choices ({})",
                annotation.choice_ordinal, annotation.decider.0, automaton.choice_count
            )));
        }
        let probs: Vec<RationalProb> = annotation.branches.iter().map(|b| b.probability).collect();
        validate_distribution(&probs)?;
    }
    Ok(map)
}

fn match_branch_probabilities(
    branches: &[(LabelId, usize)],
    annotation_branches: &[BranchProbability],
) -> Result<Vec<(LabelId, usize, RationalProb)>, AnalysisError> {
    if branches.len() != annotation_branches.len() {
        return Err(AnalysisError::InvalidProbability(format!(
            "annotation branch count {} differs from protocol choice branch count {}",
            annotation_branches.len(),
            branches.len()
        )));
    }
    let mut matched = Vec::with_capacity(branches.len());
    for &(label, target) in branches {
        let found = annotation_branches
            .iter()
            .filter(|entry| entry.label == label)
            .collect::<Vec<_>>();
        if found.len() != 1 {
            return Err(AnalysisError::InvalidProbability(format!(
                "branch label {} must appear exactly once in probability annotation",
                label.0
            )));
        }
        matched.push((label, target, found[0].probability));
    }
    Ok(matched)
}

/// Independently enumerates the synchronous rendezvous MDP directly from a
/// complete projected `LocalProtocolSetBody` and `ProbabilitySidecar`.
pub fn enumerate_source_mdp(
    requested_roles: &[RoleId],
    local_set: &LocalProtocolSetBody,
    sidecar: &ProbabilitySidecar,
    abstraction: OwnershipAbstraction,
    bounds: StateSpaceBounds,
) -> Result<EnumeratedMdp, AnalysisError> {
    choregraph_choreography::endpoint::validate_complete_set(requested_roles, local_set)
        .map_err(|diag| AnalysisError::IncompleteEndpointSet(format!("{diag:?}")))?;

    let mut automata = Vec::with_capacity(local_set.locals.len());
    for local in &local_set.locals {
        automata.push(flatten_local_protocol(
            local.role,
            &local.protocol,
            abstraction,
        )?);
    }

    let annotations = index_annotations(&automata, sidecar)?;
    let role_to_pos: BTreeMap<RoleId, usize> = automata
        .iter()
        .enumerate()
        .map(|(idx, a)| (a.role, idx))
        .collect();

    let initial_tuple = vec![0usize; automata.len()];
    let mut state_index: BTreeMap<Vec<usize>, usize> = BTreeMap::new();
    state_index.insert(initial_tuple.clone(), 0);
    let mut states = vec![initial_tuple];
    let mut transitions: Vec<Vec<MdpTransition>> = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(0usize);

    let mut ended_states = BTreeSet::new();
    let mut deadlock_states = BTreeSet::new();
    let mut branch_entry_states: BTreeMap<(RoleId, LabelId), BTreeSet<usize>> = BTreeMap::new();
    let mut observed_prob_choices: BTreeSet<(RoleId, usize)> = BTreeSet::new();
    let mut observed_nondet_choices: BTreeSet<(RoleId, usize)> = BTreeSet::new();
    let mut total_transitions: usize = 0;

    while let Some(sid) = queue.pop_front() {
        let current = states[sid].clone();
        let all_end = current
            .iter()
            .enumerate()
            .all(|(pos, &step_id)| matches!(automata[pos].steps[step_id], LocalStep::End));
        if all_end {
            ended_states.insert(sid);
            transitions.push(vec![MdpTransition {
                action: String::from("terminated"),
                outcomes: vec![(sid, RationalProb::one())],
            }]);
            total_transitions = total_transitions.saturating_add(1);
            continue;
        }

        let mut outgoing = Vec::new();

        // 1. Synchronous Send + Receive rendezvous pairs.
        for (sender_pos, sender_auto) in automata.iter().enumerate() {
            if let LocalStep::Send {
                peer,
                label,
                payload_tag,
                next: sender_next,
            } = sender_auto.steps[current[sender_pos]]
                && let Some(&recv_pos) = role_to_pos.get(&peer)
                && let LocalStep::Receive {
                    peer: recv_peer,
                    label: recv_label,
                    payload_tag: recv_tag,
                    next: recv_next,
                } = automata[recv_pos].steps[current[recv_pos]]
                && recv_peer == sender_auto.role
                && recv_label == label
                && recv_tag == payload_tag
            {
                let mut next_tuple = current.clone();
                next_tuple[sender_pos] = sender_next;
                next_tuple[recv_pos] = recv_next;
                let next_sid = intern_state(
                    &mut state_index,
                    &mut states,
                    &mut queue,
                    next_tuple,
                    bounds,
                    total_transitions,
                )?;
                outgoing.push(MdpTransition {
                    action: format!(
                        "comm_{}_{}_{}_{}",
                        sender_auto.role.0, peer.0, label.0, payload_tag.0
                    ),
                    outcomes: vec![(next_sid, RationalProb::one())],
                });
            }
        }

        // 2. Decider InternalChoice synchronized with every peer currently at
        //    `Offer { from: decider, .. }`.
        for (decider_pos, decider_auto) in automata.iter().enumerate() {
            if let LocalStep::InternalChoice {
                choice_ordinal,
                ref branches,
            } = decider_auto.steps[current[decider_pos]]
            {
                // Find all peer roles currently at Offer from this decider.
                let mut offer_peers = Vec::new();
                for (peer_pos, peer_auto) in automata.iter().enumerate() {
                    if peer_pos == decider_pos {
                        continue;
                    }
                    if let LocalStep::Offer {
                        from,
                        branches: ref peer_branches,
                    } = peer_auto.steps[current[peer_pos]]
                        && from == decider_auto.role
                    {
                        offer_peers.push((peer_pos, peer_branches));
                    }
                }

                // Every offering peer must offer every label of the decider's choice.
                let mut branch_targets = Vec::with_capacity(branches.len());
                let mut all_offers_match = true;
                for &(label, decider_next) in branches {
                    let mut next_tuple = current.clone();
                    next_tuple[decider_pos] = decider_next;
                    for &(peer_pos, peer_branches) in &offer_peers {
                        if let Some(&(_, peer_next)) =
                            peer_branches.iter().find(|(lbl, _)| *lbl == label)
                        {
                            next_tuple[peer_pos] = peer_next;
                        } else {
                            all_offers_match = false;
                            break;
                        }
                    }
                    if !all_offers_match {
                        break;
                    }
                    branch_targets.push((label, next_tuple));
                }
                if !all_offers_match {
                    continue;
                }

                let choice_key = (decider_auto.role, choice_ordinal);
                if let Some(annotation) = annotations.get(&choice_key) {
                    observed_prob_choices.insert(choice_key);
                    let raw_branches: Vec<(LabelId, usize)> = branches.clone();
                    let matched = match_branch_probabilities(&raw_branches, &annotation.branches)?;
                    let mut outcomes = Vec::with_capacity(matched.len());
                    for ((label, next_tuple), (_, _, prob)) in
                        branch_targets.into_iter().zip(matched)
                    {
                        let next_sid = intern_state(
                            &mut state_index,
                            &mut states,
                            &mut queue,
                            next_tuple,
                            bounds,
                            total_transitions,
                        )?;
                        branch_entry_states
                            .entry((decider_auto.role, label))
                            .or_default()
                            .insert(next_sid);
                        outcomes.push((next_sid, prob));
                    }
                    outgoing.push(MdpTransition {
                        action: format!("pchoice_{}_{}", decider_auto.role.0, choice_ordinal),
                        outcomes,
                    });
                } else {
                    // Preserve unannotated choice as NONDETERMINISTIC (never invent uniform probs!).
                    observed_nondet_choices.insert(choice_key);
                    for (label, next_tuple) in branch_targets {
                        let next_sid = intern_state(
                            &mut state_index,
                            &mut states,
                            &mut queue,
                            next_tuple,
                            bounds,
                            total_transitions,
                        )?;
                        branch_entry_states
                            .entry((decider_auto.role, label))
                            .or_default()
                            .insert(next_sid);
                        outgoing.push(MdpTransition {
                            action: format!(
                                "ndchoice_{}_{}_{}",
                                decider_auto.role.0, choice_ordinal, label.0
                            ),
                            outcomes: vec![(next_sid, RationalProb::one())],
                        });
                    }
                }
            }
        }

        if outgoing.is_empty() {
            deadlock_states.insert(sid);
            // Add explicit self-loop on deadlock states so PRISM evaluates PCTL
            // properties cleanly while distinguishing `"protocol_deadlock"` from `"all_ended"`.
            outgoing.push(MdpTransition {
                action: String::from("deadlock_sink"),
                outcomes: vec![(sid, RationalProb::one())],
            });
        }
        total_transitions = total_transitions.saturating_add(outgoing.len());
        if total_transitions > bounds.max_transitions {
            return Err(AnalysisError::StateSpaceExhausted {
                observed_states: states.len(),
                observed_transitions: total_transitions,
            });
        }
        transitions.push(outgoing);
    }

    let mut provenance_set = BTreeSet::new();
    for annotation in &sidecar.choices {
        let key = (annotation.decider, annotation.choice_ordinal);
        if !observed_prob_choices.contains(&key) {
            return Err(AnalysisError::UnusedOrUnknownAnnotation(format!(
                "probability annotation for role {} choice {} was unreachable/unused",
                annotation.decider.0, annotation.choice_ordinal
            )));
        }
        provenance_set.insert(match annotation.provenance {
            ProbabilityProvenance::Hypothesis => 0u8,
            ProbabilityProvenance::ExternallyEstimated => 1u8,
            ProbabilityProvenance::ValidatedObservation => 2u8,
        });
    }
    let provenance_classes = provenance_set
        .into_iter()
        .map(|tag| match tag {
            0 => ProbabilityProvenance::Hypothesis,
            1 => ProbabilityProvenance::ExternallyEstimated,
            _ => ProbabilityProvenance::ValidatedObservation,
        })
        .collect();

    Ok(EnumeratedMdp {
        roles: automata.iter().map(|a| a.role).collect(),
        states,
        transitions,
        ended_states,
        deadlock_states,
        branch_entry_states,
        provenance_classes,
        probabilistic_choices: observed_prob_choices.len(),
        nondeterministic_choices: observed_nondet_choices.len(),
    })
}

fn intern_state(
    state_index: &mut BTreeMap<Vec<usize>, usize>,
    states: &mut Vec<Vec<usize>>,
    queue: &mut VecDeque<usize>,
    tuple: Vec<usize>,
    bounds: StateSpaceBounds,
    observed_transitions: usize,
) -> Result<usize, AnalysisError> {
    if let Some(&existing) = state_index.get(&tuple) {
        return Ok(existing);
    }
    let next_id = states.len();
    if next_id.saturating_add(1) > bounds.max_states {
        return Err(AnalysisError::StateSpaceExhausted {
            observed_states: next_id.saturating_add(1),
            observed_transitions,
        });
    }
    state_index.insert(tuple.clone(), next_id);
    states.push(tuple);
    queue.push_back(next_id);
    Ok(next_id)
}

/// Exports an `EnumeratedMdp` to a PRISM MDP model (`.prism`).
#[must_use]
pub fn export_prism_mdp(mdp: &EnumeratedMdp) -> String {
    let max_state = mdp.states.len().saturating_sub(1).max(1);
    let mut out = String::new();
    out.push_str(
        "// Generated by choregraph-quantitative-role-pilot (synchronous rendezvous MDP)\n",
    );
    out.push_str("mdp\n\n");
    out.push_str("module protocol\n");
    out.push_str(&format!("  s : [0..{max_state}] init 0;\n"));
    for (sid, outgoing) in mdp.transitions.iter().enumerate() {
        for transition in outgoing {
            let mut rhs_parts = Vec::with_capacity(transition.outcomes.len());
            for &(target, prob) in &transition.outcomes {
                rhs_parts.push(format!("{} : (s'={target})", prob.as_prism_expr()));
            }
            out.push_str(&format!(
                "  [{}] s={sid} -> {};\n",
                transition.action,
                rhs_parts.join(" + ")
            ));
        }
    }
    out.push_str("endmodule\n\n");

    write_label_expr(&mut out, "all_ended", &mdp.ended_states);
    write_label_expr(&mut out, "protocol_deadlock", &mdp.deadlock_states);
    for ((decider, label), state_set) in &mdp.branch_entry_states {
        let name = format!("branch_{}_{}", decider.0, label.0);
        write_label_expr(&mut out, &name, state_set);
    }
    out
}

fn write_label_expr(out: &mut String, name: &str, state_set: &BTreeSet<usize>) {
    if state_set.is_empty() {
        out.push_str(&format!("label \"{name}\" = false;\n"));
    } else {
        let disjuncts: Vec<String> = state_set.iter().map(|sid| format!("s={sid}")).collect();
        out.push_str(&format!("label \"{name}\" = {};\n", disjuncts.join(" | ")));
    }
}

/// Exports requested properties to a PRISM `.props` file.
#[must_use]
pub fn export_prism_props(properties: &[PropertySpec]) -> String {
    let mut out = String::new();
    for prop in properties {
        out.push_str(&prop.pctl_formula());
        out.push('\n');
    }
    out
}

/// Independent conformance check comparing PRISM's own explicit exported state
/// space (`-exportstates`, `-exporttrans`, `-exportlabels`) against our
/// independently enumerated `EnumeratedMdp`.
///
/// Catches swapped participants/labels, omitted branches, altered probabilities,
/// and an exporter that labels termination as deadlock.
pub fn verify_prism_export_conformance(
    expected: &EnumeratedMdp,
    states_txt: &str,
    trans_txt: &str,
    labels_txt: &str,
) -> Result<(), AnalysisError> {
    // 1. Parse PRISM states file: maps PRISM state index -> variable `s` value.
    let mut prism_to_s: BTreeMap<usize, usize> = BTreeMap::new();
    for (line_idx, line) in states_txt.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || line_idx == 0 {
            continue;
        }
        let Some((id_str, tuple_str)) = trimmed.split_once(':') else {
            return Err(AnalysisError::ConformanceMismatch(format!(
                "malformed PRISM states line: {trimmed}"
            )));
        };
        let prism_id: usize = id_str
            .trim()
            .parse()
            .map_err(|_| AnalysisError::ConformanceMismatch(format!("bad state id: {trimmed}")))?;
        let inner = tuple_str
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')');
        let s_val: usize = inner
            .trim()
            .parse()
            .map_err(|_| AnalysisError::ConformanceMismatch(format!("bad s value: {trimmed}")))?;
        prism_to_s.insert(prism_id, s_val);
    }
    if prism_to_s.len() != expected.states.len() {
        return Err(AnalysisError::ConformanceMismatch(format!(
            "PRISM state count {} differs from enumerated state count {}",
            prism_to_s.len(),
            expected.states.len()
        )));
    }

    // 2. Parse PRISM labels file and verify `all_ended`, `protocol_deadlock`,
    //    `deadlock` (must NEVER include any `all_ended` state!), and branch labels.
    let mut label_names: BTreeMap<usize, String> = BTreeMap::new();
    let mut states_by_label: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
    for (line_idx, line) in labels_txt.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if line_idx == 0 {
            for token in trimmed.split_whitespace() {
                let Some((num_str, quoted)) = token.split_once('=') else {
                    continue;
                };
                if let Ok(num) = num_str.parse::<usize>() {
                    let name = quoted.trim_matches('"').to_string();
                    label_names.insert(num, name.clone());
                    states_by_label.entry(name).or_default();
                }
            }
            continue;
        }
        let Some((state_str, rest)) = trimmed.split_once(':') else {
            continue;
        };
        let prism_id: usize = state_str.trim().parse().map_err(|_| {
            AnalysisError::ConformanceMismatch(format!("bad label state id: {trimmed}"))
        })?;
        let Some(&s_val) = prism_to_s.get(&prism_id) else {
            return Err(AnalysisError::ConformanceMismatch(format!(
                "unknown PRISM state {prism_id} in labels"
            )));
        };
        for label_id_str in rest.split_whitespace() {
            if let Ok(label_id) = label_id_str.parse::<usize>()
                && let Some(label_name) = label_names.get(&label_id)
            {
                states_by_label
                    .entry(label_name.clone())
                    .or_default()
                    .insert(s_val);
            }
        }
    }

    // PRISM built-in `deadlock` label MUST NOT contain any `all_ended` state.
    if let Some(prism_deadlocks) = states_by_label.get("deadlock") {
        for &ended in &expected.ended_states {
            if prism_deadlocks.contains(&ended) {
                return Err(AnalysisError::ConformanceMismatch(format!(
                    "exporter labeled terminal End state {ended} as PRISM deadlock"
                )));
            }
        }
    }

    let observed_ended = states_by_label
        .get("all_ended")
        .cloned()
        .unwrap_or_default();
    if observed_ended != expected.ended_states {
        return Err(AnalysisError::ConformanceMismatch(format!(
            "all_ended label mismatch: expected {:?}, PRISM exported {:?}",
            expected.ended_states, observed_ended
        )));
    }
    let observed_deadlock = states_by_label
        .get("protocol_deadlock")
        .cloned()
        .unwrap_or_default();
    if observed_deadlock != expected.deadlock_states {
        return Err(AnalysisError::ConformanceMismatch(format!(
            "protocol_deadlock label mismatch: expected {:?}, PRISM exported {:?}",
            expected.deadlock_states, observed_deadlock
        )));
    }
    for ((decider, label), expected_set) in &expected.branch_entry_states {
        let key = format!("branch_{}_{}", decider.0, label.0);
        let observed_set = states_by_label.get(&key).cloned().unwrap_or_default();
        if observed_set != *expected_set {
            return Err(AnalysisError::ConformanceMismatch(format!(
                "branch label {key} mismatch: expected {:?}, PRISM exported {:?}",
                expected_set, observed_set
            )));
        }
    }

    // 3. Parse PRISM transitions file:
    //    Header: `<states> <choices> <transitions>`
    //    Rows: `<src> <choice_idx> <dst> <prob> [<action>]`
    let mut grouped: BTreeMap<(usize, usize), (String, BTreeMap<usize, f64>)> = BTreeMap::new();
    for (line_idx, line) in trans_txt.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || line_idx == 0 {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(AnalysisError::ConformanceMismatch(format!(
                "malformed PRISM trans line: {trimmed}"
            )));
        }
        let src_prism: usize = parts[0].parse().map_err(|_| {
            AnalysisError::ConformanceMismatch(format!("bad src in trans: {trimmed}"))
        })?;
        let choice_idx: usize = parts[1].parse().map_err(|_| {
            AnalysisError::ConformanceMismatch(format!("bad choice_idx in trans: {trimmed}"))
        })?;
        let dst_prism: usize = parts[2].parse().map_err(|_| {
            AnalysisError::ConformanceMismatch(format!("bad dst in trans: {trimmed}"))
        })?;
        let prob: f64 = parts[3].parse().map_err(|_| {
            AnalysisError::ConformanceMismatch(format!("bad prob in trans: {trimmed}"))
        })?;
        let action = parts.get(4).copied().unwrap_or("").to_string();
        let src_s = *prism_to_s.get(&src_prism).ok_or_else(|| {
            AnalysisError::ConformanceMismatch(format!("unknown src {src_prism}"))
        })?;
        let dst_s = *prism_to_s.get(&dst_prism).ok_or_else(|| {
            AnalysisError::ConformanceMismatch(format!("unknown dst {dst_prism}"))
        })?;

        let entry = grouped
            .entry((src_s, choice_idx))
            .or_insert_with(|| (action.clone(), BTreeMap::new()));
        if !action.is_empty() && entry.0.is_empty() {
            entry.0 = action;
        }
        let prev = entry.1.entry(dst_s).or_insert(0.0);
        *prev += prob;
    }

    // Compare transitions per source state against `expected.transitions`.
    for (src_s, expected_outgoing) in expected.transitions.iter().enumerate() {
        let mut observed_choices: Vec<(String, BTreeMap<usize, f64>)> = grouped
            .iter()
            .filter(|((s, _), _)| *s == src_s)
            .map(|(_, val)| val.clone())
            .collect();
        observed_choices.sort_by(|a, b| a.0.cmp(&b.0));

        let mut expected_choices: Vec<(String, BTreeMap<usize, f64>)> = expected_outgoing
            .iter()
            .map(|t| {
                let mut dist = BTreeMap::new();
                for &(dst, prob) in &t.outcomes {
                    *dist.entry(dst).or_insert(0.0) += prob.as_f64();
                }
                (t.action.clone(), dist)
            })
            .collect();
        expected_choices.sort_by(|a, b| a.0.cmp(&b.0));

        if observed_choices.len() != expected_choices.len() {
            return Err(AnalysisError::ConformanceMismatch(format!(
                "state {src_s}: expected {} outgoing choices, PRISM exported {}",
                expected_choices.len(),
                observed_choices.len()
            )));
        }
        for ((obs_act, obs_dist), (exp_act, exp_dist)) in
            observed_choices.iter().zip(expected_choices.iter())
        {
            if obs_act != exp_act {
                return Err(AnalysisError::ConformanceMismatch(format!(
                    "state {src_s}: action mismatch: expected {exp_act}, PRISM exported {obs_act}"
                )));
            }
            if obs_dist.len() != exp_dist.len() {
                return Err(AnalysisError::ConformanceMismatch(format!(
                    "state {src_s} action {exp_act}: outcome count mismatch"
                )));
            }
            for (dst, exp_p) in exp_dist {
                let Some(obs_p) = obs_dist.get(dst) else {
                    return Err(AnalysisError::ConformanceMismatch(format!(
                        "state {src_s} action {exp_act}: missing target state {dst}"
                    )));
                };
                if (obs_p - exp_p).abs() > 1e-9 {
                    return Err(AnalysisError::ConformanceMismatch(format!(
                        "state {src_s} action {exp_act} -> {dst}: prob mismatch expected {exp_p}, got {obs_p}"
                    )));
                }
            }
        }
    }
    Ok(())
}
