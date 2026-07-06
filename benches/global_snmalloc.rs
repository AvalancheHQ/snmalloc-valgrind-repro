// Mirrors runic/crates/runic-bench/benches/global_snmalloc.rs: snmalloc set as
// the global allocator, then a trivial Criterion workload. snmalloc initialises
// at process startup (before any benchmark body runs), so if it aborts under
// valgrind the whole binary dies at exit 134 regardless of what the bench does.
use criterion::{criterion_group, criterion_main, Criterion};

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

fn global_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("global/snmalloc/collections");
    group.bench_function("vec_push_clear", |b| {
        b.iter(|| {
            let mut v: Vec<u64> = Vec::new();
            for i in 0..1_024 {
                v.push(i);
            }
            v.clear();
            v
        });
    });
    group.finish();
}

criterion_group!(global_snmalloc, global_collections);
criterion_main!(global_snmalloc);
