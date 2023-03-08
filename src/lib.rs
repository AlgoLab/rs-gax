pub mod gaf;
pub mod gam;
pub mod gamp;
pub use framing::vg;

mod bindings;
mod framing;
mod graph;

impl vg::Edit {
    pub fn is_match(&self) -> bool {
        self.from_length == self.to_length && self.sequence.is_empty()
    }

    pub fn is_sub(&self) -> bool {
        self.from_length == self.to_length && !self.sequence.is_empty()
    }

    pub fn is_insertion(&self) -> bool {
        self.from_length == 0 && self.to_length > 0 && !self.sequence.is_empty()
    }

    pub fn is_deletion(&self) -> bool {
        self.from_length > 0 && self.to_length == 0
    }

    pub fn is_empty(&self) -> bool {
        self.to_length == 0 && self.from_length == 0 && self.sequence.is_empty()
    }
}

use gaf::GafRecord;
use gfa::gfa::GFA;

pub fn convert_gam_to_gaf(value: &[vg::Alignment], graph: &GFA<usize, ()>) -> Vec<GafRecord> {
    value
        .iter()
        .map(|g| GafRecord::convert_from_gam(g, graph))
        .collect()
}

pub fn convert_gaf_to_gam(value: &[GafRecord], graph: &GFA<usize, ()>) -> Vec<vg::Alignment> {
    value
        .iter()
        .map(|g| vg::Alignment::convert_from_gaf(g, graph))
        .collect()
}
