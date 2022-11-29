use std::io::prelude::*;

use crate::framing::{self, vg, Error};

pub fn parse(data: impl Read) -> Result<Vec<vg::Alignment>, Error> {
    framing::parse::<vg::Alignment>(data)
}

pub fn write(alignments: &Vec<vg::Alignment>, mut out_file: impl Write) -> Result<(), Error> {
    framing::write::<vg::Alignment>(alignments, &mut out_file)
}

#[cfg(test)]
mod tests {
    use prost_types::{value::Kind, Value};

    use super::*;
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
}
