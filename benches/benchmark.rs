use criterion::{criterion_group, criterion_main, Criterion};
use gax::{gaf, gam, gamp};
use std::io::{sink, Write};
use std::process::Command;

fn read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    group.sample_size(10);

    group.bench_function("gaf", |b| {
        b.iter(|| {
            let gaf = gaf::parse_from_file("data/example.gaf");
            write!(sink(), "{:?}", gaf).unwrap();
        })
    });

    group.bench_function("gam", |b| {
        b.iter(|| {
            let gam = gam::parse_from_file("data/example.gam");
            write!(sink(), "{:?}", gam).unwrap();
        })
    });

    group.bench_function("gamp", |b| {
        b.iter(|| {
            let gamp = gamp::parse_from_file("data/example.gamp");
            write!(sink(), "{:?}", gamp).unwrap();
        })
    });
}

fn vg_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("vg read");
    group.sample_size(10);

    group.bench_function("gam", |b| {
        b.iter(|| {
            Command::new("vg")
                .arg("view")
                .arg("-a")
                .arg("data/example.gam")
                .output()
                .expect("failed to execute process");
        })
    });
    group.bench_function("gamp", |b| {
        b.iter(|| {
            Command::new("vg")
                .arg("view")
                .arg("-jK")
                .arg("data/example.gamp")
                .output()
                .expect("failed to execute process");
        })
    });
}

criterion_group!(benches, read, vg_read);
criterion_main!(benches);
