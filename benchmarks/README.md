# Benchmarks

The benchmark harness and recorded results. Methodology and rules are in [docs/benchmarks](../docs/benchmarks/README.md).

## Planned layout

```text
benchmarks/
├── harness/     Rust benchmark binaries (criterion or custom) and generators for large inputs
├── scenarios/   One file per workload describing fixture, steps and metrics
└── results/     Recorded runs: environment, raw numbers (summarized), variance
```

Raw per-run output goes under `results/raw/` and is ignored by git; commit summaries only.

Nothing is implemented yet. Do not add numbers to this directory that were not produced by the harness.
