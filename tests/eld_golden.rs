//! Bit-exact regression check for the AAC-ELD decode path.
//!
//! Every stream under `tests/fixtures/` is decoded twice — as encoded, and with
//! deterministic damage so that concealment runs — and the decoder's `f32` output,
//! frame geometry and error sequence are hashed. `golden.txt` holds what the
//! unmodified AOSP decoder produced for the same input; `ELD_GOLDEN_WRITE=1`
//! regenerates it.
//!
//! The streams are raw access units in the `.rawpkts` layout: a little-endian `i32`
//! length and the AudioSpecificConfig, then a length and an access unit, repeated.
//! They were encoded by the FDK AAC 2.0.3 encoder as AAC-ELD without SBR.

use aac::aac_dec::AacDecoderInstance;
use std::{fmt::Write as _, fs, path::Path};

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

fn packets(stream: &[u8]) -> Vec<&[u8]> {
    let mut packets = Vec::new();
    let mut rest = stream;
    while !rest.is_empty() {
        let (len, tail) = rest.split_at(4);
        let len = i32::from_le_bytes(len.try_into().unwrap()) as usize;
        let (packet, tail) = tail.split_at(len);
        packets.push(packet);
        rest = tail;
    }
    packets
}

/// Damage some access units the same way on every run: bit flips in one, a
/// truncation in another, garbage in a third.
fn damaged(index: usize, unit: &[u8]) -> Vec<u8> {
    let mut unit = unit.to_vec();
    let mut state = 0x2545_f491_4f6c_dd1d_u64 ^ (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    match index % 41 {
        13 => {
            for _ in 0..4 {
                let at = next() as usize % unit.len();
                unit[at] ^= 1 << (next() % 8);
            }
        }
        27 => unit.truncate(unit.len() / 2),
        35 | 36 => unit.iter_mut().for_each(|b| *b = next() as u8),
        _ => {}
    }
    unit
}

fn decode(stream: &[u8], damage: bool) -> String {
    let packets = packets(stream);
    let mut decoder = AacDecoderInstance::new();
    decoder.config_raw(packets[0]).expect("the AudioSpecificConfig is accepted");

    let mut pcm = vec![0.0f32; 2 * 1024];
    let mut hash = Fnv::new();
    let (mut ok, mut failed, mut samples) = (0usize, 0usize, 0usize);
    for (index, unit) in packets[1..].iter().enumerate() {
        let unit = if damage { damaged(index, unit) } else { unit.to_vec() };
        let left = decoder.fill(&unit, unit.len()).expect("the access unit fits");
        assert_eq!(left, 0);
        let info = match decoder.decode(&mut pcm) {
            Ok(info) => {
                ok += 1;
                hash.bytes(b"ok");
                info
            }
            Err((error, info)) => {
                failed += 1;
                hash.bytes(format!("{error:?}").as_bytes());
                info
            }
        };
        let out = info;
        hash.bytes(&out.frame_size.to_le_bytes());
        hash.bytes(&out.num_channels.to_le_bytes());
        hash.bytes(&out.sampling_rate.to_le_bytes());
        let produced = usize::from(out.frame_size) * usize::from(out.num_channels);
        samples += produced;
        for sample in &pcm[..produced] {
            hash.bytes(&sample.to_bits().to_le_bytes());
        }
    }
    format!("ok={ok} failed={failed} samples={samples} hash={:016x}", hash.0)
}

#[test]
fn every_fixture_decodes_to_what_the_unmodified_decoder_produced() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".rawpkts"))
        .collect();
    names.sort();
    assert!(!names.is_empty());

    let mut report = String::new();
    for name in &names {
        let stream = fs::read(dir.join(name)).unwrap();
        writeln!(report, "{name} clean {}", decode(&stream, false)).unwrap();
        writeln!(report, "{name} damaged {}", decode(&stream, true)).unwrap();
    }

    let golden = dir.join("golden.txt");
    if std::env::var_os("ELD_GOLDEN_WRITE").is_some() {
        fs::write(&golden, &report).unwrap();
    }
    assert_eq!(report, fs::read_to_string(&golden).unwrap());
}
