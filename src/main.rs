use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod gaf;
use gaf::GafRecord;

fn main() {
    println!("Hello, world!");

    // GAF I/O
    let f = File::open("data/example.gaf").unwrap();
    let f = BufReader::new(f);
    let mut of = File::create("data/example.out.gaf").unwrap();
    for line in f.lines() {
        let mut gr: GafRecord = GafRecord::new(); // TODO/FIXME: move this outside
        gr.parse_gaf_record(&line.unwrap());
        gr.write(&mut of).ok();
    }

    // GAM I/O
    let f = File::open("data/example.gam").unwrap();
    // let f = BufReader::new(f);
    // let mut of = File::create("data/example.out.gaf").unwrap();
    // for line in f.lines() {
    //     let mut gr: GafRecord = GafRecord::new();
    //     gr.parse_gaf_record(&line.unwrap());
    //     gr.write(&mut of).ok();
    // }
}
