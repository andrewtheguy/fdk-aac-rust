//! The `.rawpkts` layout: a little-endian `i32` length and the AudioSpecificConfig,
//! then a length and an access unit, repeated.

use std::{fs, io::Write};

pub fn write(path: &str, config: &[u8], units: &[Vec<u8>]) {
    let mut file = fs::File::create(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    for packet in std::iter::once(config).chain(units.iter().map(Vec::as_slice)) {
        file.write_all(&(packet.len() as i32).to_le_bytes()).unwrap();
        file.write_all(packet).unwrap();
    }
}

pub fn read(path: &str) -> (Vec<u8>, Vec<Vec<u8>>) {
    let stream = fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut packets = Vec::new();
    let mut rest = stream.as_slice();
    while !rest.is_empty() {
        let (len, tail) = rest.split_first_chunk::<4>().expect("a length is cut short");
        let (packet, tail) = tail.split_at(i32::from_le_bytes(*len) as usize);
        packets.push(packet.to_vec());
        rest = tail;
    }
    let config = packets.remove(0);
    (config, packets)
}
