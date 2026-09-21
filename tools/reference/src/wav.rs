//! 16-bit PCM WAV, which is all the encoder takes.

use std::fs;

pub struct Wav {
    pub rate: u32,
    pub channels: usize,
    pub samples: Vec<i16>,
}

pub fn read(path: &str) -> Result<Wav, String> {
    let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(format!("{path}: not a RIFF/WAVE file"));
    }
    let (mut format, mut data) = (None, None);
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
        // A streamed file leaves the data length at its placeholder.
        let body = &bytes[at + 8..(at + 8 + len).min(bytes.len())];
        match &bytes[at..at + 4] {
            b"fmt " if body.len() >= 16 => {
                let field = |i: usize| u16::from_le_bytes([body[i], body[i + 1]]);
                let rate = u32::from_le_bytes(body[4..8].try_into().unwrap());
                format = Some((field(0), field(2), rate, field(14)));
            }
            b"data" => data = Some(body),
            _ => {}
        }
        at += 8 + len + (len & 1);
    }
    let (tag, channels, rate, bits) = format.ok_or(format!("{path}: no fmt chunk"))?;
    let data = data.ok_or(format!("{path}: no data chunk"))?;
    // 0xfffe is WAVE_FORMAT_EXTENSIBLE, which ffmpeg writes for PCM it could have called 1.
    if !(tag == 1 || tag == 0xfffe) || bits != 16 {
        return Err(format!(
            "{path}: format {tag:#x} at {bits} bits; 16-bit PCM is wanted \
             (ffmpeg -i IN -c:a pcm_s16le OUT.wav)"
        ));
    }
    let samples = data.as_chunks::<2>().0.iter().map(|s| i16::from_le_bytes(*s)).collect();
    Ok(Wav { rate, channels: usize::from(channels), samples })
}
