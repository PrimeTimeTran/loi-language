use criterion::{Criterion, criterion_group, criterion_main};

fn bench_lexer(c: &mut Criterion) {
    // let input = "x = 5 + 3; y = x + 2;";

    c.bench_function("lexer", |b| {
        b.iter(|| {
            // call your lexer here
            // lexer::lex(input)
        })
    });
}

criterion_group!(benches, bench_lexer);
criterion_main!(benches);
