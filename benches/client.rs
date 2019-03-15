#[macro_use]
extern crate criterion;

use criterion::Criterion;

fn bench_client_request(c: &mut Criterion) {
    // c.bench_function("client request", move |b| {
    //     b.iter(|| {
    //         CoAPClient::get("coap://127.0.0.1:5683/guess/whos/back").unwrap();
    //     })
    // });
}

criterion_group!(benches, bench_client_request);
criterion_main!(benches);
