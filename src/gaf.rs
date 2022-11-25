use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/**
 * We allow pretty much any field to be set as "*" in the GAF, which gets mapped to a -1
 * in the numeric fields (as there are no valid negative values)
 */
const MISSING_INT: i64 = -1;
const MISSING_CHAR: char = '*';
const MISSING_STRING: &str = "*";

/**
 * One step of a GAF path
 */
struct GafStep {
    name: String,      // Either a path name or segment/node name (see above)
    is_reverse: bool,  // In reverse orientation ('<' in GAF)
    is_stable: bool,   // True if it's a stable path name (as opposed to segment/node name)
    is_interval: bool, // True if it's an interval of a stable path (false if it's the whole path)
    start: i64, // 0-based start (inclusive). only defined if is_stable and is_interval are true
    end: i64,   // 0-based end (inclusive). only defined if is_stable and is_interval are true
}

impl GafStep {
    fn new() -> GafStep {
        GafStep {
            name: MISSING_STRING.to_owned(),
            is_reverse: false,
            is_stable: false,
            is_interval: false,
            start: MISSING_INT,
            end: MISSING_INT,
        }
    }

    /*
     * Write a GAF Step to a stream
     */
    fn write(&self, f: &mut File) -> std::io::Result<()> {
        if !self.is_stable || self.is_interval {
            if self.is_reverse {
                write!(f, "<")?;
            } else {
                write!(f, ">")?;
            }
        }
        write!(f, "{}", self.name)?;
        if self.is_interval {
            write!(f, ":{}-{}", self.start, self.end)?;
        }
        Ok(())
    }
}
/**
 * One line of GAF as described here: https://github.com/lh3/gfatools/blob/master/doc/rGFA.md
 */

pub struct GafRecord {
    query_name: String, // Query sequence name
    query_length: i64,  // Query sequence length
    query_start: i64,   // 0-based, closed
    query_end: i64,     // 0-based, open
    path_length: i64,
    path_start: i64,    // Start position on the path (0-based)
    path_end: i64,      // End position on the path (0-based)
    matches: i64,       // Number of residue matches
    block_length: i64,  // Alignment block length
    mapq: i32,          // Mapping quality (0-255; 255 for missing)
    strand: char,       // strand relative to the path + or -
    path: Vec<GafStep>, // the path

    // Map a tag name to its type and value
    // ex: "de:f:0.2183" in the GAF would appear as opt_fields["de"] = ("f", "0.2183")
    opt_fields: HashMap<String, (String, String)>,
}

impl GafRecord {
    // Init everything to missing
    pub fn new() -> GafRecord {
        GafRecord {
            query_name: MISSING_STRING.to_owned(),
            query_length: MISSING_INT,
            query_start: MISSING_INT,
            query_end: MISSING_INT,
            path_length: MISSING_INT,
            path_start: MISSING_INT,
            path_end: MISSING_INT,
            matches: MISSING_INT,
            block_length: MISSING_INT,
            mapq: 255,
            strand: MISSING_CHAR,
            path: Vec::new(),
            opt_fields: HashMap::new(),
        }
    }

    /**
     * Parse a single GAF record
     */
    pub fn parse_gaf_record(&mut self, line: &str) {
        let mut split = line.split('\t');
        // let x1 = split.next().unwrap();
        // println!("{x1}");
        // let x2 = split.next().unwrap();
        let mut token: &str = split.next().unwrap();
        self.query_name = token.to_string();

        token = split.next().unwrap();
        self.query_length = token.parse::<i64>().unwrap();

        token = split.next().unwrap();
        self.query_start = token.parse::<i64>().unwrap();

        token = split.next().unwrap();
        self.query_end = token.parse::<i64>().unwrap();

        token = split.next().unwrap();
        self.strand = token.chars().nth(0).unwrap();

        self.path.clear();
        token = split.next().unwrap();
        let c: char = token.chars().nth(0).unwrap();
        if c == '<' || c == '>' {
            // our path is a list of oriented segments or intervales
            let re = Regex::new(r"([><][^\s><]+(:\d+-\d+)?)").unwrap();
            for cap in re.captures_iter(token) {
                let mut step: GafStep = GafStep::new();
                let step_token: &str = &cap[1];
                step.is_reverse = &step_token[0..1] == "<";
                let colon: usize = match step_token.find(':') {
                    Some(pos) => pos,
                    None => usize::MAX,
                };
                if colon == usize::MAX {
                    // no colon, we interpret the step as a segID
                    step.name = step_token[1..].to_string();
                    step.is_stable = false;
                    step.is_interval = false;
                } else {
                    // colon, we interpret the step as a stable path interval
                    step.name = step_token[1..colon - 1].to_string();
                    step.is_stable = true;
                    step.is_interval = true;
                    let dash: usize = match step_token[colon..].find('-') {
                        Some(pos) => pos,
                        None => usize::MAX,
                    };
                    if dash == usize::MAX {
                        panic!("Error parsing GAF range of {}", step_token);
                    }
                    step.start = step_token[colon + 1..colon + dash].parse::<i64>().unwrap();
                    step.end = step_token[colon + 1 + dash..].parse::<i64>().unwrap();
                }
                self.path.push(step);
            }
        } else if token != "*" {
            // our path is a stable path name
            let mut step: GafStep = GafStep::new();
            step.name = token.to_string();
            step.is_reverse = false;
            step.is_stable = true;
            step.is_interval = false;
            self.path.push(step);
        }

        token = split.next().unwrap();
        self.path_length = token.parse::<i64>().unwrap();
        token = split.next().unwrap();
        self.path_start = token.parse::<i64>().unwrap();
        token = split.next().unwrap();
        self.path_end = token.parse::<i64>().unwrap();
        token = split.next().unwrap();
        self.matches = token.parse::<i64>().unwrap();
        token = split.next().unwrap();
        self.block_length = token.parse::<i64>().unwrap();

        token = split.next().unwrap();
        if token == MISSING_STRING {
            self.mapq = MISSING_INT as i32;
        } else {
            self.mapq = token.parse::<i32>().unwrap();
            if self.mapq >= 255 {
                self.mapq = MISSING_INT as i32;
            }
        }

        self.opt_fields.clear();
        for opt_token in split.collect::<Vec<&str>>() {
            let key: &str = &opt_token[..2];
            let typ: &str = &opt_token[3..4];
            let value: &str = &opt_token[5..];
            if self.opt_fields.get(key) != None {}
            match self
                .opt_fields
                .insert(key.to_string(), (typ.to_string(), value.to_string()))
            {
                Some(_) => panic!("Duplicate optional field found: {}", key),
                None => (),
            }
        }
    }

    /**
     * Write a GAF record to a stream
     */
    pub fn write(self, f: &mut File) -> std::io::Result<()> {
        if self.query_name.is_empty() {
            write!(f, "{}\t", MISSING_STRING)?;
        } else {
            write!(f, "{}\t", self.query_name)?;
        }
        write!(
            f,
            "{}\t{}\t{}\t{}\t",
            self.query_length, self.query_start, self.query_end, self.strand
        )?;
        if self.path.is_empty() {
            write!(
                f,
                "{}\t{}\t{}\t{}\t{}\t{}",
                MISSING_STRING,
                MISSING_STRING,
                MISSING_STRING,
                MISSING_STRING,
                MISSING_STRING,
                MISSING_STRING
            )?;
        } else {
            for step in self.path.iter() {
                step.write(f)?;
            }
            write!(
                f,
                "\t{}\t{}\t{}\t{}\t{}\t",
                self.path_length, self.path_start, self.path_end, self.matches, self.block_length
            )?
        }

        if self.mapq as i64 == MISSING_INT {
            write!(f, "{}", 255)?;
        } else {
            write!(f, "{}", self.mapq)?;
        }
        for (key, value) in self.opt_fields {
            write!(f, "\t{}:{}:{}", key, value.0, value.1)?;
        }
        writeln!(f, "")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_2() {
        let mut rec: GafRecord = GafRecord::new();
        assert_eq!(rec.query_length, -1);
        assert_eq!(rec.mapq, 255);

        // let line: &str = "read1\t6\t0\t6\t+\t>s2>s3>s4\t12\t2\t8\t6\t6\t60\tcg:Z:6M";
        let line: &str = "read2\t7\t0\t7\t-\t>chr1:5-8>foo:8-16\t11\t1\t8\t7\t7\t60\tcg:Z:7M";
        rec.parse_gaf_record(line);
        assert_eq!(rec.query_name, "read2");
        assert_eq!(rec.query_length, 7);
        assert_eq!(rec.strand, '-');
    }
}
