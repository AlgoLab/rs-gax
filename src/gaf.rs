use pyo3::FromPyObject;
use std::collections::HashMap;
use std::io::{Read, Write};

pub fn parse(data: impl Read) -> Vec<GafRecord> {
    let mut records = Vec::new();
    let mut string = String::new();
    let mut reader = std::io::BufReader::new(data);
    reader.read_to_string(&mut string).unwrap();
    for line in string.lines() {
        let gr = GafRecord::parse(line);
        records.push(gr);
    }
    records
}

pub fn write(records: &Vec<GafRecord>, mut out_file: impl Write) -> std::io::Result<()> {
    for record in records {
        record.write(&mut out_file)?;
    }
    Ok(())
}

/**
 * We allow pretty much any field to be set as "*" in the GAF, which gets mapped to a -1
 * in the numeric fields (as there are no valid negative values)
 */
const MISSING_INT: i64 = -1;
const MISSING_STRING: &str = "*";

/**
 * One step of a GAF path
 */
#[derive(Debug, PartialEq, Eq, Clone, FromPyObject)]
pub struct GafStep {
    pub name: String,       // Either a path name or segment/node name (see above)
    pub is_reverse: bool,   // In reverse orientation ('<' in GAF)
    pub is_stable: bool,    // True if it's a stable path name (as opposed to segment/node name)
    pub is_interval: bool, // True if it's an interval of a stable path (false if it's the whole path)
    pub start: Option<i64>, // 0-based start (inclusive). only defined if is_stable and is_interval are true
    pub end: Option<i64>, // 0-based end (inclusive). only defined if is_stable and is_interval are true
}

impl GafStep {
    /*
     * Write a GAF Step to a stream
     */
    fn write(&self, mut f: impl Write) -> std::io::Result<()> {
        if !self.is_stable || self.is_interval {
            if self.is_reverse {
                write!(f, "<")?;
            } else {
                write!(f, ">")?;
            }
        }
        write!(f, "{}", self.name)?;
        if self.is_interval {
            write!(f, ":{}-{}", self.start.unwrap(), self.end.unwrap())?;
        }
        Ok(())
    }
}

/**
 * One line of GAF as described here: https://github.com/lh3/gfatools/blob/master/doc/rGFA.md
 */

#[derive(Debug, Clone, PartialEq, Eq, FromPyObject)]
pub struct GafRecord {
    pub query_name: String, // Query sequence name
    pub query_length: i64,  // Query sequence length
    pub query_start: i64,   // 0-based, closed
    pub query_end: i64,     // 0-based, open
    pub path_length: i64,
    pub path_start: i64,    // Start position on the path (0-based)
    pub path_end: i64,      // End position on the path (0-based)
    pub matches: i64,       // Number of residue matches
    pub block_length: i64,  // Alignment block length
    pub mapq: i32,          // Mapping quality (0-255; 255 for missing)
    pub strand: char,       // strand relative to the path + or -
    pub path: Vec<GafStep>, // the path

    // Map a tag name to its type and value
    // ex: "de:f:0.2183" in the GAF would appear as opt_fields["de"] = ("f", "0.2183")
    pub opt_fields: HashMap<String, (String, String)>,
}

fn number_or_missing(token: &str) -> i64 {
    if token == "*" {
        MISSING_INT
    } else {
        token.parse::<i64>().unwrap()
    }
}

impl GafRecord {
    /**
     * Parse a single GAF record
     */
    pub fn parse(line: &str) -> Self {
        let mut split = line.split('\t');

        let mut token = split.next().unwrap();
        let query_name = token.to_string();
        token = split.next().unwrap();
        let query_length = number_or_missing(token);
        token = split.next().unwrap();
        let query_start = number_or_missing(token);
        token = split.next().unwrap();
        let query_end = number_or_missing(token);
        token = split.next().unwrap();
        let strand = token.chars().next().unwrap();

        token = split.next().unwrap();
        let mut path = Vec::new();
        if token.to_string().starts_with(['<', '>']) {
            // orientIntv
            let mut splits: Vec<_> = token.match_indices(['<', '>']).map(|(i, _)| i).collect();
            splits.push(token.len());

            for step_token in splits
                .windows(2)
                .map(|indexes| &token[indexes[0]..indexes[1]])
            {
                let is_reverse = &step_token[0..1] == "<";
                let s = match step_token.find(':') {
                    Some(colon) => {
                        let Some(dash) = step_token[colon..].find('-') else {
                            panic!("Error parsing GAF range of {}", step_token)
                        };
                        let start = step_token[colon + 1..colon + dash].parse::<i64>().unwrap();
                        let end = step_token[colon + 1 + dash..].parse::<i64>().unwrap();
                        // stableIntv
                        GafStep {
                            name: (step_token[1..colon - 1]).to_string(),
                            is_reverse,
                            is_stable: true,
                            is_interval: true,
                            start: Some(start),
                            end: Some(end),
                        }
                    }
                    None => {
                        // segId
                        GafStep {
                            name: (step_token[1..]).to_string(),
                            is_reverse,
                            is_stable: false,
                            is_interval: false,
                            start: None,
                            end: None,
                        }
                    }
                };
                path.push(s);
            }
        } else {
            // stableId
            path.push(GafStep {
                name: token.to_string(),
                is_reverse: false,
                is_stable: true,
                is_interval: false,
                start: None,
                end: None,
            });
        }

        token = split.next().unwrap();
        let path_length = number_or_missing(token);
        token = split.next().unwrap();
        let path_start = number_or_missing(token);
        token = split.next().unwrap();
        let path_end = number_or_missing(token);
        token = split.next().unwrap();
        let matches = number_or_missing(token);
        token = split.next().unwrap();
        let block_length = number_or_missing(token);

        token = split.next().unwrap();
        let mapq = if token == MISSING_STRING {
            MISSING_INT as _
        } else {
            token.parse::<i32>().unwrap().min(MISSING_INT as _)
        };

        let mut opt_fields = HashMap::new();
        for opt_token in split.collect::<Vec<&str>>() {
            let key: &str = &opt_token[..2];
            let typ: &str = &opt_token[3..4];
            let value: &str = &opt_token[5..];
            if opt_fields
                .insert(key.to_string(), (typ.to_string(), value.to_string()))
                .is_some()
            {
                panic!("Duplicate optional field found: {}", key)
            }
        }

        Self {
            query_name,
            query_length,
            query_start,
            query_end,
            path_length,
            path_start,
            path_end,
            matches,
            block_length,
            mapq,
            strand,
            path,
            opt_fields,
        }
    }

    /**
     * Write a GAF record to a stream
     */
    pub fn write(&self, mut f: impl Write) -> std::io::Result<()> {
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
                step.write(&mut f)?;
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
        for (key, value) in &self.opt_fields {
            write!(f, "\t{}:{}:{}", key, value.0, value.1)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaf_read() {
        // let line: &str = "read1\t6\t0\t6\t+\t>s2>s3>s4\t12\t2\t8\t6\t6\t60\tcg:Z:6M";
        let line: &str = "read2\t7\t0\t7\t-\t>chr1:5-8>foo:8-16\t11\t1\t8\t7\t7\t60\tcg:Z:7M";
        let rec: GafRecord = GafRecord::parse(line);
        assert_eq!(rec.query_name, "read2");
        assert_eq!(rec.query_length, 7);
        assert_eq!(rec.strand, '-');
    }
}
