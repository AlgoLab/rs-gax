use bytes::Buf;
use flate2::{read::MultiGzDecoder, Compression, GzBuilder};
use prost::{
    encoding::{decode_varint, encode_varint},
    Message,
};
use std::io::prelude::*;
use std::io::Cursor;

// Include the `vg` module, which is generated from vg.proto.
#[allow(clippy::all)]
pub mod vg {
    include!(concat!(env!("OUT_DIR"), "/vg.rs"));
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gam {
    pub alignments: Vec<vg::Alignment>,
}

impl Gam {
    pub fn parse(data: impl Read) -> Result<Self, GamError> {
        // Decompress data
        let mut decoder = MultiGzDecoder::new(data);
        let mut data = vec![];
        decoder.read_to_end(&mut data)?;
        let mut cursor = Cursor::new(data);

        let mut alignments = vec![];
        while cursor.remaining() != 0 {
            // Read number of messages in the group
            let number_messages = decode_varint(&mut cursor)? - 1;

            // Read type tag
            let type_tag_len = decode_varint(&mut cursor)?;
            let mut type_tag = vec![0; type_tag_len as usize];
            cursor.read_exact(&mut type_tag)?;
            let type_tag = String::from_utf8(type_tag)?;
            // Should always be equal to GAM
            assert!(type_tag == "GAM");

            for _ in 0..number_messages {
                // Read all messages in the group
                let message_len = decode_varint(&mut cursor)?;
                let mut buffer = vec![0; message_len as _];
                cursor.read_exact(&mut buffer)?;
                let mut tmp = &buffer[..];
                let alignment = vg::Alignment::decode(&mut tmp)?;
                alignments.push(alignment);
            }
        }
        Ok(Self { alignments })
    }

    pub fn write(mut self, mut out_file: impl Write) -> Result<(), GamError> {
        let mut buf = vec![];
        while !self.alignments.is_empty() {
            let end_index = self.alignments.len().min(1000);
            let alignments: Vec<vg::Alignment> = self.alignments.drain(..end_index).collect();
            write_group(alignments, &mut buf)?;
        }

        // Compress data
        // FIXME For big files we should use multi streams
        // This is not currently supported by flate2, see this PR:
        // https://github.com/rust-lang/flate2-rs/pull/325
        let mut encoder = GzBuilder::new().write(&mut out_file, Compression::new(9));
        encoder.write_all(&buf)?;
        Ok(())
    }
}

fn write_group(alignments: Vec<vg::Alignment>, mut file: impl Write) -> Result<(), GamError> {
    let mut buf = vec![];
    // Write number of messages in the group
    encode_varint(alignments.len() as u64 + 1, &mut buf);
    // Write type tag
    encode_varint(3, &mut buf);
    file.write_all(&buf)?;
    file.write_all("GAM".as_bytes())?;

    // Write all messages
    for alignment in alignments {
        let mut buf = vec![];
        vg::Alignment::encode(&alignment, &mut buf)?;
        let mut buf_len = vec![];
        encode_varint(buf.len() as _, &mut buf_len);
        // Write message length
        file.write_all(&buf_len)?;
        // Write message
        file.write_all(&buf)?;
    }
    Ok(())
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum GamError {
    IoError(#[from] std::io::Error),
    Utf8Error(#[from] std::string::FromUtf8Error),
    ProstDecodeError(#[from] prost::DecodeError),
    ProstEncodeError(#[from] prost::EncodeError),
}
