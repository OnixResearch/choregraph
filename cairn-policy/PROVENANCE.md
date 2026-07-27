# Cairn policy provenance

The local lifecycle policy was initialized from Cairn revision `70db8fe114910feb5d162fea5db8ad4ffdb6f176`.

`contracts.ncl` and `default.ncl` are authoritative. The policy permits only repository-owned source and evidence roots. It rejects generated outputs, build outputs, and vendored source as traceability evidence.

Refresh the runtime policy with:

```text
nix run path:$CAIRN_SOURCE#cairn -- policy export --output cairn-policy/generated/cairn-policy.json
```

After each refresh, run policy freshness, lifecycle validation, gates, and traceability coverage.
