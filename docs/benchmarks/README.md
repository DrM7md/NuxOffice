# Benchmark methodology

Performance claims in this project are backed by reproducible measurements or not made at all. This page defines how we measure. Recorded results live under `benchmarks/results/`.

## Workloads (from blueprint section 11)

| Workload | Metrics |
|---|---|
| Open a 500k-row XLSX | Wall time, peak RSS, fidelity of the loaded data |
| Recalculate 100k formulas | Initial and incremental latency, result correctness |
| Edit a precedent that affects many formulas | Invalidation and recalculation latency |
| Scroll and search a huge sheet | Frame time, UI responsiveness |
| Round-trip an XLSM | Macro preservation, formatting changes, warnings emitted |
| Start with an empty workbook | Cold and warm startup time, memory |

## Rules

1. **Record the environment.** CPU, RAM, storage, OS and version, kernel, compiler and toolchain versions, application versions (including LibreOffice Calc and, where legally available, Microsoft Excel), build profile, and whether the cache was cold or warm.
2. **Use reproducible inputs.** Workbooks are generated in code from a seed or fetched from the pinned test corpus with a checksum (see upstream `docs/upstream/GRIDCRAFT_AGENTS.md`), so anyone can rerun the workload.
3. **Run multiple times.** Report the number of runs, median, and variance (or min/max). Never report a single lucky run.
4. **Same data, same machine** for every comparison. Cross-machine comparisons are not comparisons.
5. **Never fabricate.** If a measurement has not been taken, the table says "not measured".
6. **Correctness first.** A faster result that is wrong is a failure, not a win.

## Decision rule

Replace or reimplement an existing proven component only when measurements show unacceptable performance, incompatibility, maintenance or architectural constraints. Do not sacrifice correctness to improve a number.

## Status

No benchmarks have been run yet. The harness in `benchmarks/` is not implemented.
