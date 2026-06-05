use criterion::{Criterion, criterion_group, criterion_main};

fn bench_parser(c: &mut Criterion) {
    // let input = "x = 5 + 3; y = x + 2;";

    c.bench_function("parser", |b| {
        b.iter(|| {
            // call your parser here
            // parser::parse(input)
        })
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
