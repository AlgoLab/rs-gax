use std::io::prelude::*;

use crate::framing::{self, vg, Error};

pub fn parse(data: impl Read) -> Result<Vec<vg::Alignment>, Error> {
    framing::parse::<vg::Alignment>(data)
}

pub fn write(alignments: &Vec<vg::Alignment>, mut out_file: impl Write) -> Result<(), Error> {
    framing::write::<vg::Alignment>(alignments, &mut out_file)
}
