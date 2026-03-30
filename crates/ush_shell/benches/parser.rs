use std::collections::BTreeMap;

use criterion::{Criterion, criterion_group, criterion_main};
use ush_shell::parse_line;

fn bench_parser(criterion: &mut Criterion) {
    let aliases = BTreeMap::from([("ll".to_string(), "ls -la".to_string())]);
    criterion.bench_function("parse pipeline with helper", |bench| {
        bench.iter(|| {
            let _ = parse_line(
                "ll src | filter(it -> contains(it, \"rs\")) | len",
                &aliases,
            );
        });
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
