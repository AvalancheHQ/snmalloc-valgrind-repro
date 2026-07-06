# snmalloc-valgrind-repro

Minimal reproduction of a failure the CodSpeed Wizard hit while setting up
[`botirkhaltaev/runic`](https://github.com/botirkhaltaev/runic).

The `runic-bench` suite uses `snmalloc-rs` as a `#[global_allocator]` in one of
its Criterion benchmarks. When run under CodSpeed **simulation** mode (which
executes the benchmark under valgrind), the process aborts at startup with:

```
Failed to initialise snmalloc.
failed to execute the benchmark process, exit code: 134
```

The Wizard concluded "snmalloc is incompatible with valgrind" and switched the
setup to walltime mode. This repo exists to check whether that conclusion is
correct, and if the failure is really caused by valgrind or by something else.

This crate reproduces the setup as minimally as possible:

- `snmalloc-rs = "0.3"` with default features (same as runic)
- `codspeed-criterion-compat` aliased back to `criterion` (same as runic)
- a single trivial Criterion workload

## Reproduce

```bash
cargo codspeed build
codspeed run --mode simulation -- cargo codspeed run
```
