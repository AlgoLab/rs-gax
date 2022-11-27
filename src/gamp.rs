use std::io::prelude::*;

use crate::framing::{self, vg, Error};

pub fn parse(data: impl Read) -> Result<Vec<vg::MultipathAlignment>, Error> {
    framing::parse::<vg::MultipathAlignment>(data)
}

pub fn write(
    alignments: &Vec<vg::MultipathAlignment>,
    mut out_file: impl Write,
) -> Result<(), Error> {
    framing::write::<vg::MultipathAlignment>(alignments, &mut out_file)
}
