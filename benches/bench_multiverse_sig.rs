use multiverse_signatures::{common::sig_utils, multiverse_sig::*};
use rand::{thread_rng};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

//N denotes number of parties, K denotes weight per party, and T denotes threshold ratio
fn bench_multiverse_sig<const N: usize, const K: usize>(c: &mut Criterion) {

    let threshold : f64 = 0.5;
    let total_weight = K * N;
    let weight_threshold = ((total_weight as f64) * threshold) as usize;

    let mut rng = thread_rng();

    let crs = sig_utils::test_setup::<1000>(&mut rng);
    let addr_book = sig_utils::create_addr_book(N, K);

    let dealer = MultiverseParty::new(crs, weight_threshold, total_weight, &addr_book);

    let output = dealer.setup();

    let msg_to_sign = "Hello Multiverse";

    //let's collect signatures from 80 out of 100 parties
    let mut partial_sigs: Vec<MultiversePartialSig> = Vec::new();
    for id in 1..(2*N/3) {
        partial_sigs.push(dealer.sign(id, msg_to_sign.as_bytes(), &output));
    }

    c.bench_function(
        format!("bench_sign<N={},K={}>", N, K).as_str(),
        |b| {
            b.iter(|| dealer.sign(
                black_box(1),
                black_box(msg_to_sign.as_bytes()),
                black_box(&output)));
        },
    );

    let aggregate_sig = dealer.aggregate(&output, &partial_sigs).unwrap();
    c.bench_function(
        format!("bench_aggregate<N={},K={}>", N, K).as_str(),
        |b| {
            b.iter(|| dealer.aggregate(
                black_box(&output),
                black_box(&partial_sigs)));
        },
    );

    assert_eq!(dealer.verify(msg_to_sign.as_bytes(), &output, &aggregate_sig), true);
    c.bench_function(
        format!("bench_verify<N={},K={}>", N, K).as_str(),
        |b| {
            b.iter(|| dealer.verify(
                black_box(msg_to_sign.as_bytes()),
                black_box(&output),
                black_box(&aggregate_sig)));
        },
    );
}

mod perf;

criterion_group!(
    name = multiverse_sig;
    config = Criterion::default().
        with_profiler(perf::FlamegraphProfiler::new(5)).
        sample_size(10);
    targets = bench_multiverse_sig<50,10>,
        bench_multiverse_sig<100,10>,
        bench_multiverse_sig<200,10>
);
criterion_main!(multiverse_sig);
