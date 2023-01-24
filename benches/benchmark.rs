use criterion::{criterion_group, criterion_main, Criterion};
use gax::{gaf, gam, gamp};
use std::fs::File;
use std::io::{sink, Write};
use std::process::Command;

fn read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    group.sample_size(10);

    group.bench_function("gaf", |b| {
        b.iter(|| {
            let f = File::open("data/example.gaf").unwrap();
            let gaf = gaf::parse(f);
            write!(sink(), "{:?}", gaf).unwrap();
        })
    });

    group.bench_function("gam", |b| {
        b.iter(|| {
            let f = File::open("data/example.gam").unwrap();
            let gam = gam::parse(f);
            write!(sink(), "{:?}", gam).unwrap();
        })
    });

    group.bench_function("gamp", |b| {
        b.iter(|| {
            let f = File::open("data/example.gamp").unwrap();
            let gamp = gamp::parse(f);
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
