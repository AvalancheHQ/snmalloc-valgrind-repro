# Findings: why snmalloc "fails under valgrind" during CodSpeed simulation

**Short version:** The Wizard's conclusion ("snmalloc is incompatible with
valgrind") is not the root cause and is too strong. snmalloc's *default* build
reserves a single **256 GiB** address range at startup; valgrind refuses an
`mmap` reservation that large and returns `EINVAL`, so snmalloc aborts. The same
binary initialises fine natively. Building snmalloc with its built-in
**`qemu` feature** (`SNMALLOC_QEMU_WORKAROUND`) shrinks the reservation to
32 GiB, valgrind accepts it, and simulation mode works.

All of the below was reproduced in the CodSpeed Wizard's own dev sandbox image
(`wizard-template-dev`, linux/amd64), with the exact tooling the Wizard used:

- `snmalloc-rs 0.3.8` / `snmalloc-sys 0.3.8` (cmake-built), same as `runic`
- `codspeed-criterion-compat 5.0.1`
- `cargo-codspeed 4.6.0`, `codspeed-runner 4.17.5`
- `valgrind-3.26.0.codspeed3` (CodSpeed's valgrind fork)

## What happens (default snmalloc, under valgrind)

snmalloc aborts at allocator initialisation, before any benchmark body runs:

```
Failed to initialise snmalloc.
==…== Process terminating with default action of signal 6 (SIGABRT)
==…==    at … abort
==…==    by … snmalloc::PALPOSIX<snmalloc::PALLinux>::error(char const*)
==…==    by … snmalloc::…::ensure_init_slow()
==…==    by … snmalloc::LocalAllocator<…>::init()
```
Exit code 134 (128 + SIGABRT). This is snmalloc's own PAL calling `abort()`.

### The actual failing syscall

With `valgrind --trace-syscalls=yes`, the line right before the abort is:

```
sys_mmap ( 0x0, 274877906944, PROT_READ|WRITE, MAP_PRIVATE|ANON|NORESERVE, -1, 0 ) --> Failure(0x16)   # EINVAL
sys_madvise ( 0x0, 274877906944, … ) --> Failure(0xc)                                                  # ENOMEM
Failed to initialise snmalloc.
```

`274877906944` = **256 GiB**. snmalloc reserves this as one flat, lazily-backed
(`MAP_NORESERVE`) address range. A real kernel grants it for free (it's only
virtual address space). **valgrind's address-space manager cannot satisfy a
reservation that large and returns `EINVAL`.** snmalloc treats that as fatal.

So the failure is genuinely valgrind-specific — it is NOT an environment / setup
problem, and NOT a problem in runic's own code. Natively the same binary gets
past init (it reaches the harness argument parser).

## The fix (proven)

Build snmalloc-rs with its `qemu` feature, which sets `SNMALLOC_QEMU_WORKAROUND`
in the C++ build (a mode meant exactly for emulators/valgrind that can't do the
huge reservation):

```toml
snmalloc-rs = { version = "0.3", features = ["qemu"] }
```

Re-running the identical binary under the identical valgrind:

```
sys_mmap ( 0x0, 34359738368, … )   # 32 GiB instead of 256 GiB
==…== Warning: set address range perms: large range [0x59cca000, 0x859cca000)   # 32 GiB, accepted
```

- reservation drops **256 GiB → 32 GiB**
- valgrind accepts it
- `Failed to initialise snmalloc` occurrences: **0**
- process runs to a clean valgrind exit (no SIGABRT)

## Reproduce it yourself

```bash
cargo codspeed build
BIN=target/codspeed/analysis/snmalloc-valgrind-repro/global_snmalloc
valgrind --trace-syscalls=yes "$BIN" --bench   # SIGABRT, "Failed to initialise snmalloc"

# then flip the dependency to features = ["qemu"], rebuild, and repeat -> no abort
```

## Open question (not yet tested)

Reproduced on CodSpeed's valgrind fork (`3.26.0.codspeed3`). The 256 GiB-mmap
`EINVAL` is a general valgrind address-space limitation, so upstream valgrind is
expected to behave the same, but that was not separately verified here. If
CodSpeed's fork can be configured to allow a larger managed address space, that
would be an alternative to asking users to enable snmalloc's `qemu` feature.

Evidence logs are under `evidence/`.
