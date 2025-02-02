// Copyright 2020 TiKV Project Authors. Licensed under Apache-2.0.

use criterion::{criterion_group, criterion_main, Criterion};

fn init_opentelemetry() {
    use tracing_subscriber::prelude::*;

    let opentelemetry = tracing_opentelemetry::layer();
    tracing_subscriber::registry()
        .with(opentelemetry)
        .try_init()
        .unwrap();
}

fn opentelemetry_harness(n: usize) {
    fn dummy_opentelementry(n: usize) {
        for _ in 0..n {
            let child = tracing::span!(tracing::Level::TRACE, "child");
            let _enter = child.enter();
        }
    }

    let root = tracing::span!(tracing::Level::TRACE, "parent");
    let _enter = root.enter();

    dummy_opentelementry(n);
}

fn rustracing_harness(n: usize) {
    fn dummy_rustracing(n: usize, span: &rustracing::span::Span<()>) {
        for _ in 0..n {
            let _child_span = span.child("child", |c| c.start_with_state(()));
        }
    }

    let (span_tx, span_rx) = crossbeam::channel::bounded(1000);

    {
        let tracer = rustracing::Tracer::with_sender(rustracing::sampler::AllSampler, span_tx);
        let parent_span = tracer.span("parent").start_with_state(());
        dummy_rustracing(n, &parent_span);
    }

    let _r = span_rx.iter().collect::<Vec<_>>();
}

fn minitrace_harness(n: usize) {
    use minitrace::prelude::*;

    fn dummy_minitrace(n: usize) {
        for _ in 0..n {
            let _guard = LocalSpan::enter_with_local_parent("child");
        }
    }

    let _spans = {
        let (root_span, collector) = Span::root("parent");
        let _g = root_span.set_local_parent();

        dummy_minitrace(n);

        collector
    }
    .collect();
}

fn tracing_comparison(c: &mut Criterion) {
    init_opentelemetry();

    let mut bgroup = c.benchmark_group("compare");

    for n in &[1, 10, 100, 1000] {
        bgroup.bench_function(format!("Tokio Tracing/{n}"), |b| {
            b.iter(|| opentelemetry_harness(*n))
        });
        bgroup.bench_function(format!("Rustracing/{n}"), |b| {
            b.iter(|| rustracing_harness(*n))
        });
        bgroup.bench_function(format!("minitrace/{n}"), |b| {
            b.iter(|| minitrace_harness(*n))
        });
    }

    bgroup.finish();
}

criterion_group!(benches, tracing_comparison);
criterion_main!(benches);
