//! Decode a `.rawpkts` stream of AAC-ELD access units to a 16-bit WAV.
//!
//!     cargo run --release --example decode -- tests/fixtures/eld_48k_stereo_480_music_128k.rawpkts out.wav
//!
//! `.rawpkts` is a little-endian `i32` length and the AudioSpecificConfig, then a
//! length and an access unit, repeated. A damaged unit is concealed and its
//! substitute frame written like any other; a unit the decoder cannot produce at all
//! is written as silence, so the output keeps its length.

use aac::aac_dec::AacDecoderInstance;
use std::{env, fs, process::ExitCode};

/// Frames are at most 512 samples in each of at most two channels.
const MAX_FRAME: usize = 2 * 512;

fn packets(stream: &[u8]) -> Result<Vec<&[u8]>, String> {
    let mut packets = Vec::new();
    let mut rest = stream;
    while !rest.is_empty() {
        let (len, tail) = rest.split_first_chunk::<4>().ok_or("a length is cut short")?;
        let len = usize::try_from(i32::from_le_bytes(*len)).map_err(|_| "a length is negative")?;
        if len > tail.len() {
            return Err(format!("a packet of {len} bytes is cut short"));
        }
        let (packet, tail) = tail.split_at(len);
        packets.push(packet);
        rest = tail;
    }
    Ok(packets)
}

fn wav(sampling_rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let block_align = channels * 2;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sampling_rate.to_le_bytes());
    out.extend_from_slice(&(sampling_rate * u32::from(block_align)).to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

fn run(input: &str, output: &str) -> Result<(), String> {
    let stream = fs::read(input).map_err(|e| format!("{input}: {e}"))?;
    let packets = packets(&stream).map_err(|e| format!("{input}: {e}"))?;
    let (config, units) = packets.split_first().ok_or(format!("{input}: empty"))?;

    let mut decoder = AacDecoderInstance::new();
    decoder
        .config_raw(config)
        .map_err(|e| format!("the AudioSpecificConfig {config:02x?} is refused: {e:?}"))?;

    let mut pcm = [0.0f32; MAX_FRAME];
    let mut samples = Vec::new();
    let mut shape = None;
    let (mut concealed, mut failed) = (0usize, 0usize);
    for (index, unit) in units.iter().enumerate() {
        decoder.fill(unit, unit.len()).map_err(|e| format!("unit {index}: {e:?}"))?;
        let info = match decoder.decode(&mut pcm) {
            Ok(info) => info,
            Err((e, info)) if e.is_decode_error() => {
                concealed += 1;
                info
            }
            Err((e, info)) => {
                eprintln!("unit {index}: {e:?}");
                failed += 1;
                pcm.fill(0.0);
                info
            }
        };
        let produced = usize::from(info.frame_size) * usize::from(info.num_channels);
        if produced == 0 {
            continue;
        }
        shape = Some((info.sampling_rate, info.num_channels));
        samples.extend(
            pcm[..produced].iter().map(|s| (s * 32768.0).round().clamp(-32768.0, 32767.0) as i16),
        );
    }

    let (sampling_rate, channels) = shape.ok_or("no unit decoded")?;
    fs::write(output, wav(sampling_rate, u16::from(channels), &samples))
        .map_err(|e| format!("{output}: {e}"))?;
    println!(
        "{output}: {} units, {concealed} concealed, {failed} failed, {sampling_rate} Hz, {channels} ch, {:.2} s",
        units.len(),
        samples.len() as f64 / f64::from(sampling_rate) / f64::from(channels),
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let [input, output] = args.as_slice() else {
        eprintln!("usage: decode INPUT.rawpkts OUTPUT.wav");
        return ExitCode::from(2);
    };
    match run(input, output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
