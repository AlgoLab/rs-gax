use crate::{
    framing::{self, vg, Error},
    gaf::GafRecord,
    graph::GFAExt, complement,
};
use gfa::gfa::GFA;
use prost_types::{value::Kind, Struct, Value};
use std::{collections::BTreeMap, fs::File, io::prelude::*};

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

impl vg::Alignment {
    pub fn convert_from_gaf(value: &GafRecord, graph: &GFA<usize, ()>) -> Self {
        let mut mapping = value
            .path
            .iter()
            .enumerate()
            .map(|(rank, step)| {
                let offset = if rank == 0 { value.path_start } else { 0 };
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

        let mut annotation = BTreeMap::new();
        let mut sequence = String::new();
        if !value.path.is_empty() {
            let mut cur_mapping = 0;
            let mut cur_offset = value.path_start;

            let mut cur_len =
                graph.node_to_length(mapping[cur_mapping].position.as_ref().unwrap().node_id);
            let mut from_cg = false;
            for cigar in value.iter_cigar() {
                if !from_cg
                    && cigar.cat != ":"
                    && cigar.cat != "+"
                    && cigar.cat != "-"
                    && cigar.cat != "*"
                {
                    from_cg = true;
                }
                match cigar.cat.as_str() {
                    ":" | "M" | "=" | "X" => {
                        let mut match_len = cigar.length;
                        while match_len > 0 {
                            let current_match = match_len.min(
                                graph.node_to_length(
                                    mapping[cur_mapping].position.as_ref().unwrap().node_id,
                                ) - cur_offset as usize,
                            );
                            let mut edit_sequence = String::new();
                            if cigar.cat == "X" {
                                edit_sequence = "N".repeat(current_match);
                            }
                            let cur_position = mapping[cur_mapping].position.clone().unwrap();
                            sequence += &graph
                                .node_to_sequence(cur_position.node_id, cur_position.is_reverse)
                                [cur_offset as usize..cur_offset as usize + current_match];

                            let edit = vg::Edit {
                                from_length: current_match as i32,
                                to_length: current_match as i32,
                                sequence: edit_sequence,
                            };
                            match_len -= current_match;
                            cur_offset += current_match as i64;
                            mapping[cur_mapping].edit.push(edit);
                            if match_len > 0 {
                                cur_mapping += 1;
                                cur_offset = 0;
                                cur_len = graph.node_to_length(cur_position.node_id);
                            }
                        }
                    }
                    "+" | "I" | "S" => {
                        let mut target_mapping = cur_mapping;
                        if cur_offset == 0
                            && cur_mapping > 0
                            && (!mapping[cur_mapping - 1]
                                .position
                                .as_ref()
                                .unwrap()
                                .is_reverse
                                || cur_mapping == mapping.len())
                        {
                            // left-align insertion
                            target_mapping -= 1;
                        }
                        let edit_sequence = if cigar.cat == "+" {
                            cigar.query
                        } else {
                            "N".repeat(cigar.length)
                        };
                        sequence += &edit_sequence;

                        let edit = vg::Edit {
                            from_length: 0,
                            to_length: cigar.length as i32,
                            sequence: edit_sequence,
                        };

                        mapping[target_mapping].edit.push(edit);
                    }
                    "-" | "D" => {
                        let mut del_len = cigar.length;
                        while del_len > 0 {
                            let current_del = del_len.min(graph.node_to_length(
                                mapping[cur_mapping].position.as_ref().unwrap().node_id
                                    - cur_offset,
                            ));
                            let edit = vg::Edit {
                                from_length: current_del as i32,
                                to_length: 0,
                                sequence: String::new(),
                            };
                            del_len -= current_del;
                            cur_offset += current_del as i64;
                            mapping[cur_mapping].edit.push(edit);
                            if del_len > 0 {
                                cur_mapping += 1;
                                cur_offset = 0;
                                cur_len = graph.node_to_length(
                                    mapping[cur_mapping].position.as_ref().unwrap().node_id,
                                );
                            }
                        }
                    }
                    "*" => {
                        sequence += &cigar.query;
                        let edit = vg::Edit {
                            from_length: cigar.length as i32,
                            to_length: cigar.length as i32,
                            sequence: cigar.query,
                        };
                        mapping[cur_mapping].edit.push(edit);
                        cur_offset += 1;
                    }
                    _ => unreachable!(),
                }
                if cur_offset == cur_len as i64 {
                    cur_mapping += 1;
                    cur_offset = 0;
                    if cur_mapping < mapping.len() {
                        cur_len = graph.node_to_length(
                            mapping[cur_mapping].position.as_ref().unwrap().node_id,
                        );
                    }
                }
            }

            if from_cg {
                // remember that we came from a lossy cg-cigar -> GAM conversion path
                annotation.insert(
                    "from_cg".to_string(),
                    Value {
                        kind: Some(Kind::BoolValue(true)),
                    },
                );
            }
        }

        let path = vg::Path {
            mapping,
            ..Default::default()
        };

        let annotation = if annotation.is_empty() {
            None
        } else {
            Some(Struct { fields: annotation })
        };

        let mut alignment = Self {
            name: value.query_name.clone(),
            sequence: complement(sequence),
            path: Some(path),
            mapping_quality: value.mapq,
            annotation,
            ..Default::default()
        };

        for (key, value) in value.opt_fields.clone() {
            match key.as_str() {
                "dv" => {
                    // get the identity from the dv divergence field
                    alignment.identity = 1.0 - value.1.parse::<f64>().unwrap();
                }
                "AS" => {
                    // get the score from the AS field
                    alignment.score = value.1.parse::<i32>().unwrap();
                }
                "bq" => {
                    // get the quality from the bq field
                    todo!();
                }
                "fp" => {
                    // get the fragment_previous field
                    if let Some(fragment) = alignment.fragment_prev.as_mut() {
                        fragment.name = value.1;
                    }
                }
                "fn" => {
                    // get the fragment_next field
                    if let Some(fragment) = alignment.fragment_next.as_mut() {
                        fragment.name = value.1;
                    }
                }
                "pd" => {
                    //Is this read properly paired
                    if let Some(annotations) = alignment.annotation.as_mut() {
                        annotations.fields.insert(
                            "proper_pair".to_string(),
                            Value {
                                kind: Some(Kind::BoolValue(value.1 == "1")),
                            },
                        );
                    }
                }
                _ => (),
            }
        }

        alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // #[test]
    // fn convert() {
    //     let gam = gam::parse_from_file("data/convert.gam").unwrap();
    //     let gaf = gaf::parse_from_file("data/convert.gaf");
    //     let gam_from_gaf: Vec<framing::vg::Alignment> =
    //         gaf.into_iter().map(|record| record.into()).collect();
    //     assert_eq!(gam, gam_from_gaf);
    // }
}
