use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod gaf;
use gaf::GafRecord;

mod framing;
mod gam;
mod gamp;

fn main() {
    if let Err(e) = main_() {
        eprintln!("ERROR: {}", e);
    }
}

fn main_() -> Result<(), Box<dyn std::error::Error>> {
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
    println!("GAF: {} records", count);

    // GAM I/O
    let in_file = "data/example.gam";
    let out_file = "data/example.out.gam";
    let f = File::open(in_file)?;
    let f = BufReader::new(f);
    let gam = gam::parse(f)?;
    let of = File::create(out_file)?;
    gam::write(&gam, of)?;
    assert!(gam == gam::parse(File::open(out_file)?)?);
    println!("GAM: {} records", gam.len());

    // GAMP I/O
    let in_file = "data/example.gamp";
    let out_file = "data/example.out.gamp";
    let f = File::open(in_file)?;
    let f = BufReader::new(f);
    let gamp = gamp::parse(f)?;
    let of = File::create(out_file)?;
    gamp::write(&gamp, of)?;
    assert!(gamp == gamp::parse(File::open(out_file)?)?);
    println!("GAMP: {} records", gamp.len());
    Ok(())
}
