use std::fs::File;
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
    // GAF I/O
    let in_file = "data/example.gaf";
    let out_file = "data/example.out.gaf";
    let f = File::open(in_file).unwrap();
    let f = BufReader::new(f);
    let of = File::create(out_file).unwrap();
    let gaf: Vec<GafRecord> = gaf::parse(f);
    gaf::write(&gaf, of)?;
    assert!(gaf == gaf::parse(File::open(out_file)?));
    println!("GAF: {} records", gaf.len());

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
