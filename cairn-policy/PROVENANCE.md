# Cairn policy provenance

The local lifecycle policy was initialized from Cairn revision `70db8fe114910feb5d162fea5db8ad4ffdb6f176`.

The current upstream policy snapshot comes from Cairn revision `695124d459574ba7aeba6097310d237f393c243c`. The copied files are `contracts.ncl`, `lifecycle-store-contracts.ncl`, `nominal-evidence-identities.ncl`, and `upstream-default.ncl`.

`default.ncl` is the authoritative Choregraph overlay. It imports the pinned upstream default and uses explicit `force` priorities for project-owned fields. The policy permits only repository-owned source and evidence roots. It rejects generated outputs, build outputs, and vendored source as traceability evidence.

Refresh the runtime policy with:

```text
nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

After each refresh, run policy freshness, lifecycle validation, gates, and traceability coverage.
