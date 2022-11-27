use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod gaf;
use gaf::GafRecord;

use crate::gam::Gam;

mod gam;

fn main() {
    println!("Hello, world!");

    // GAF I/O
    let f = File::open("data/example.gaf").unwrap();
    let f = BufReader::new(f);
    let mut of = File::create("data/example.out.gaf").unwrap();
    let mut count = 0;
    for line in f.lines() {
        count += 1;
        let mut gr: GafRecord = GafRecord::new(); // TODO/FIXME: move this outside
        gr.parse_gaf_record(&line.unwrap());
        gr.write(&mut of).ok();
    }
    println!("{} records", count);

    // GAM I/O
    let in_file = "data/example.gam";
    let out_file = "data/example.out.gam";
    let f = File::open(in_file).unwrap();
    let gam = Gam::parse(f);
    let of = File::create(out_file).unwrap();
    gam.unwrap().clone().write(of).unwrap();
}
