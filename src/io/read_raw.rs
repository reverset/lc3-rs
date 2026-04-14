#![deprecated(
    since = "0.9.5",
    note = "Will be replaced with a similar format that allows for more sections than just one .orig"
)]

use std::io::Read;

use crate::io::{AssemblyInfo, DataInfo};

// TODO improve to allow for more orig sections
pub fn read(mut data: &[u8]) -> AssemblyInfo {
    let mut res = Vec::new();

    let mut orig = None;
    loop {
        let mut buf = [0u8; 2];
        if data.read_exact(&mut buf).is_err() {
            // we're done reading
            break;
        }

        // concatenate the two bytes.
        let value = ((buf[0] as u16) << 8) | ((buf[1] as u16) & 0b11111111);

        if orig.is_none() {
            // first two bytes are the "origin" bytes (where the code starts)
            orig = Some(value);
        } else {
            res.push(value as i16);
        }
    }

    AssemblyInfo {
        data: vec![DataInfo {
            orig: orig.unwrap(),
            data: res,
        }],
    }
}
