use std::{fs::File, io::prelude::*};

use crate::{
    framing::{self, vg, Error},
    gaf::GafRecord,
};

pub fn parse(data: impl Read) -> Result<Vec<vg::Alignment>, Error> {
    framing::parse::<vg::Alignment>(data)
}

pub fn parse_from_file(path: impl AsRef<std::path::Path>) -> Result<Vec<vg::Alignment>, Error> {
    let f = File::open(path)?;
    parse(f)
}

pub fn write(alignments: &[vg::Alignment], mut out_file: impl Write) -> Result<(), Error> {
    framing::write::<vg::Alignment>(alignments, &mut out_file)
}

pub fn write_to_file(
    alignments: &[vg::Alignment],
    path: impl AsRef<std::path::Path>,
) -> Result<(), Error> {
    let f = File::create(path)?;
    write(alignments, f)
}

impl From<GafRecord> for vg::Alignment {
    fn from(value: GafRecord) -> Self {
        let mut first = true;
        let mapping = value
            .path
            .iter()
            .enumerate()
            .map(|(rank, step)| {
                let offset = if first {
                    first = false;
                    value.path_start
                } else {
                    0
                };
                let position = vg::Position {
                    node_id: step.name.parse::<i64>().unwrap(),
                    offset,
                    is_reverse: step.is_reverse,
                    ..Default::default()
                };

                vg::Mapping {
                    position: Some(position),
                    rank: rank as i64 + 1,
                    ..Default::default()
                }
            })
            .collect::<Vec<_>>();

        let path = vg::Path {
            mapping,
            ..Default::default()
        };

        Self {
            name: value.query_name.clone(),
            path: Some(path),
            mapping_quality: value.mapq,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{gaf, gam};
    use prost_types::{value::Kind, Value};
    use std::fs::File;

    #[test]
    fn gam_read() {
        let in_file = "data/example.gam";
        let f = File::open(in_file).unwrap();
        let alignments: Vec<vg::Alignment> = parse(f).unwrap();
        let first = alignments[0].clone();

        let name = "FBtr0342963_e_1536_X_294766";
        let mapping_quality = 60;
        let sequence = "TAGATAAAAAATAAACGGAAAATTTGTTATTTCTTTCGTACATGGTAAAGAATCTTTTTTTACTTGTGTTTCTGTGATTTGAGTGTTTGAAAAATTTAAC";
        let score = 110;

        assert_eq!(first.name, name);
        assert_eq!(first.mapping_quality, mapping_quality);
        assert_eq!(first.sequence, sequence);
        assert_eq!(first.score, score);
    }

    #[test]
    fn gam_write() {
        let out_file = "data/example.out.gam";
        let of = File::create(out_file).unwrap();
        let alignment = vg::Alignment {
            name: "test".into(),
            mapping_quality: 1000,
            sequence: "AAAAATAAACGG".into(),
            score: 99,
            ..Default::default()
        };
        let alignments: Vec<vg::Alignment> = vec![alignment.clone()];
        write(&alignments, of).unwrap();

        let in_file = "data/example.out.gam";
        let f = File::open(in_file).unwrap();
        let alignments: Vec<vg::Alignment> = parse(f).unwrap();
        let first = alignments[0].clone();

        assert_eq!(first, alignment);
    }

    #[test]
    fn gam_edit() {
        let in_file = "data/example.gam";
        let f = File::open(in_file).unwrap();

        let alignments: Vec<vg::Alignment> = parse(f).unwrap();
        let mut alignment = alignments[0].clone();

        alignment.name = "new_name".into();
        alignment.annotation.as_mut().unwrap().fields.insert(
            "new_value".into(),
            Value {
                kind: Some(Kind::NumberValue(10.0)),
            },
        );

        let out_file = "data/example.out.gam";
        let of = File::create(out_file).unwrap();
        write(&(vec![alignment.clone()]), of).unwrap();

        let in_file = "data/example.out.gam";
        let f = File::open(in_file).unwrap();
        let alignments: Vec<vg::Alignment> = parse(f).unwrap();
        let first = alignments[0].clone();

        assert_eq!(first, alignment);
    }

    #[test]
    fn convert() {
        let gam = gam::parse_from_file("data/convert.gam").unwrap();
        let gaf = gaf::parse_from_file("data/convert.gaf");
        let gam_from_gaf: Vec<framing::vg::Alignment> =
            gaf.into_iter().map(|record| record.into()).collect();
        assert_eq!(gam, gam_from_gaf);
    }
}
