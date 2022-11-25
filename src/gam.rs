use std::io::Cursor;

use prost::Message;

// Include the `vg` module, which is generated from vg.proto.
pub mod vg {
    include!(concat!(env!("OUT_DIR"), "/vg.rs"));
}

pub fn deserialize(buf: &[u8]) -> Result<vg::Alignment, prost::DecodeError> {
    vg::Alignment::decode(&mut Cursor::new(buf))
}
