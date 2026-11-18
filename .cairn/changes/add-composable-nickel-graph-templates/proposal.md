# Add composable Nickel graph templates

## Problem

Lattice needs a captured Nickel-generated workflow that native Choregraph can admit without treating author labels, a Nickel contract, or an evaluator receipt as runtime authority. The restricted Nickel CLI cannot be repurposed as a pre-evaluation import loader, and Lattice v1/v2 projection semantics must remain stable. Template reuse currently requires manual graph copying and unreviewed identity decisions.

## Proposed capability

Add `choregraph.graph-construction.v1`: a finite Nickel helper library plus forced export contract; a selected typed native registry and pure scoped elaboration; a separate Lattice v3 projection with data and control dependencies; and a binding over exact captured inputs, evaluator observations, registry, expansion, admission, origins and artifact. The supported packaging route uses nickel-eval's closed captured-module profile. Lattice is the first consumer; Choregraph maintains the construction format and native compiler, nickel-eval maintains evaluator resolution observations, and Lattice maintains consumer registry truth and runtime authority.

## Scope and exclusions

Author immutable module bytes, versioned DTO and registry, bounded composition/instantiation/wiring, structured expressions, typed copy/drop/effect checks, precise attribution and stale-binding revalidation. Include positive/negative fixtures and a maintained validation rail. Do not alter restricted CLI, v1/v2 projection meanings, kernel transition algorithm, Lattice runtime, permissions, scheduling or persistence. No publication or adoption follows from source authoring alone.

## Evidence posture

This package records an authored implementation and acceptance requirements, not a passing evaluation or test report. Native checks, evaluator closure controls, Lattice import compatibility and product adoption must be separately observed before release claims.
