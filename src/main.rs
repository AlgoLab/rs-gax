use gax::{convert_gaf_to_gam, convert_gam_to_gaf, gaf, gam, gamp};
use gfa::{gfa::GFA, parser::GFAParser};

fn main() {
    if let Err(e) = main_() {
        eprintln!("ERROR: {}", e);
    }
}

fn main_() -> Result<(), Box<dyn std::error::Error>> {
    let graph: GFA<usize, ()> = GFAParser::new().parse_file("data/convert.gfa")?;
    let gaf = gaf::parse_from_file("data/convert.gaf");
    let gam = gam::parse_from_file("data/convert.gam")?;

    let generated_gaf = convert_gam_to_gaf(&gam, &graph);
    let index = 256;
    assert_eq!(gaf[index].path, generated_gaf[index].path);

    // gam.iter().zip(generated_gam.iter()).enumerate().for_each(|(i, (a, b))| {
    //     dbg!(&gaf[i].path);
    //     assert_eq!(a.path, b.path);
    // });

    return Ok(());

    // GAF I/O
    let gaf = gaf::parse_from_file("data/example.gaf");
    gaf::write_to_file(&gaf, "data/example.out.gaf")?;
    assert!(gaf == gaf::parse_from_file("data/example.out.gaf"));
    println!("GAF: {} records", gaf.len());

    // GAM I/O
    let gam = gam::parse_from_file("data/example.gam")?;
    gam::write_to_file(&gam, "data/example.out.gam")?;
    assert!(gam == gam::parse_from_file("data/example.out.gam")?);
    println!("GAM: {} records", gam.len());

    // GAMP I/O
    let gamp = gamp::parse_from_file("data/example.gamp")?;
    gamp::write_to_file(&gamp, "data/example.out.gamp")?;
    assert!(gamp == gamp::parse_from_file("data/example.out.gamp")?);
    println!("GAMP: {} records", gamp.len());
    Ok(())
}
