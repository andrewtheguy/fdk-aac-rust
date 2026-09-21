//! The `aac` crate checked against Fraunhofer's C codec, FDK AAC 2.0.3.
//!
//!     eld-reference encode IN.wav OUT.rawpkts [BITRATE] [FRAME]
//!     eld-reference compare STREAM.rawpkts...
//!     eld-reference fixtures DIR
//!
//! `encode` turns a 16-bit WAV at 48, 44.1 or 32 kHz, mono or stereo, into AAC-ELD
//! without SBR — 128 000 bit/s and 480-sample frames unless told otherwise — so any
//! audio can be put through the decoder (`cargo run --example decode`). `compare`
//! decodes streams with both decoders and fails when they differ by more than the C
//! decoder's 16-bit fixed point explains; it takes undamaged streams, as the two
//! conceal with different random noise. `fixtures` writes the streams under
//! `tests/fixtures`.

mod encoder;
mod rawpkts;
mod signals;
mod wav;

use aac::aac_dec::AacDecoderInstance;
use fdk_aac::dec::{Decoder, Transport};
use std::process::ExitCode;

/// The C decoder is 16-bit fixed point and this one is floating point, so they agree
/// to the C decoder's rounding and no further: within a couple of steps on a quiet
/// signal, and about 80 dB below a loud one, where the rounding error scales with
/// it. A stream passes on either count; a decoding fault fails both by a wide margin.
const ROUNDING_STEPS: i32 = 2;
const MIN_SNR_DB: f64 = 70.0;

fn fixture(path: &str, kind: &str, rate: u32, channels: usize, bitrate: u32, granule: u32, seconds: f64) {
    let pcm = signals::signal(kind, rate, channels, seconds);
    let encoded = encoder::encode(&pcm, rate, channels, bitrate, granule);
    rawpkts::write(path, &encoded.config, &encoded.units);
    report(path, &encoded);
}

fn report(path: &str, encoded: &encoder::Encoded) {
    let hex: String = encoded.config.iter().map(|b| format!("{b:02x}")).collect();
    let bytes: usize = encoded.units.iter().map(Vec::len).sum();
    println!("{path}: asc={hex} units={} bytes={bytes}", encoded.units.len());
}

fn fixtures(dir: &str) {
    std::fs::create_dir_all(dir).unwrap();
    // The Apple Screen Sharing shape: 48 kHz, stereo, 480-sample frames, no SBR.
    for (kind, bitrate) in [("music", 128_000), ("transients", 96_000), ("noise", 32_000), ("sweep", 64_000), ("music", 24_000), ("noise", 256_000)] {
        fixture(&format!("{dir}/eld_48k_stereo_480_{kind}_{}k.rawpkts", bitrate / 1000), kind, 48_000, 2, bitrate, 480, 3.0);
    }
    // The rest of what plain AAC-ELD allows and the tables still carry.
    fixture(&format!("{dir}/eld_48k_stereo_512_music_96k.rawpkts"), "music", 48_000, 2, 96_000, 512, 2.0);
    fixture(&format!("{dir}/eld_44k_stereo_480_transients_64k.rawpkts"), "transients", 44_100, 2, 64_000, 480, 2.0);
    fixture(&format!("{dir}/eld_48k_mono_480_music_48k.rawpkts"), "music", 48_000, 1, 48_000, 480, 2.0);
    fixture(&format!("{dir}/eld_32k_mono_512_noise_24k.rawpkts"), "noise", 32_000, 1, 24_000, 512, 2.0);
}

fn encode(input: &str, output: &str, bitrate: u32, granule: u32) -> Result<(), String> {
    let wav = wav::read(input)?;
    if ![48_000, 44_100, 32_000].contains(&wav.rate) || ![1, 2].contains(&wav.channels) {
        return Err(format!(
            "{input}: {} Hz in {} channels; the decoder takes 48, 44.1 or 32 kHz, mono or stereo \
             (ffmpeg -i IN -ar 48000 -ac 2 -c:a pcm_s16le OUT.wav)",
            wav.rate, wav.channels
        ));
    }
    if ![480, 512].contains(&granule) {
        return Err(format!("frames are 480 or 512 samples, not {granule}"));
    }
    let encoded = encoder::encode(&wav.samples, wav.rate, wav.channels, bitrate, granule);
    rawpkts::write(output, &encoded.config, &encoded.units);
    report(output, &encoded);
    Ok(())
}

/// Both decoders' output for one stream, as 16-bit samples.
fn decode_both(path: &str) -> Result<(Vec<i16>, Vec<i16>), String> {
    let (config, units) = rawpkts::read(path);

    let mut ours = AacDecoderInstance::new();
    ours.config_raw(&config).map_err(|e| format!("{path}: config refused: {e:?}"))?;
    let mut theirs = Decoder::new(Transport::Raw).map_err(|e| format!("{path}: {e:?}"))?;
    theirs.config_raw(&config).map_err(|e| format!("{path}: the C decoder refused the config: {e:?}"))?;

    let mut float = [0.0f32; 2 * 512];
    let mut fixed = [0i16; 2 * 512];
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for (index, unit) in units.iter().enumerate() {
        ours.fill(unit, unit.len()).map_err(|e| format!("{path}: unit {index}: {e:?}"))?;
        let info = ours.decode(&mut float).map_err(|(e, _)| format!("{path}: unit {index}: {e:?}"))?;
        let produced = usize::from(info.frame_size) * usize::from(info.num_channels);
        a.extend(float[..produced].iter().map(|s| (s * 32768.0).round().clamp(-32768.0, 32767.0) as i16));

        theirs.fill(unit).map_err(|e| format!("{path}: unit {index}: the C decoder: {e:?}"))?;
        theirs.decode_frame(&mut fixed).map_err(|e| format!("{path}: unit {index}: the C decoder: {e:?}"))?;
        let produced = theirs.decoded_frame_size();
        b.extend_from_slice(&fixed[..produced]);
    }
    Ok((a, b))
}

fn compare(paths: &[String]) -> Result<bool, String> {
    let mut agree = true;
    for path in paths {
        let (ours, theirs) = decode_both(path)?;
        if ours.len() != theirs.len() {
            return Err(format!("{path}: {} samples here, {} from the C decoder", ours.len(), theirs.len()));
        }
        let mut worst = 0i32;
        let (mut signal, mut noise) = (0.0f64, 0.0f64);
        for (&a, &b) in ours.iter().zip(&theirs) {
            let difference = i32::from(a) - i32::from(b);
            worst = worst.max(difference.abs());
            signal += f64::from(b) * f64::from(b);
            noise += f64::from(difference) * f64::from(difference);
        }
        let snr = if noise == 0.0 { f64::INFINITY } else { 10.0 * (signal / noise).log10() };
        let same = worst <= ROUNDING_STEPS || snr >= MIN_SNR_DB;
        let verdict = if same { "ok" } else { "DIFFERS" };
        println!("{path}: {} samples, worst difference {worst}, SNR {snr:.1} dB: {verdict}", ours.len());
        agree &= same;
    }
    Ok(agree)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let number = |s: &str| s.parse::<u32>().map_err(|e| format!("{s}: {e}"));
    let result = match args.as_slice() {
        ["fixtures", dir] => {
            fixtures(dir);
            Ok(true)
        }
        ["encode", input, output, rest @ ..] if rest.len() <= 2 => (|| {
            let bitrate = rest.first().map_or(Ok(128_000), |s| number(s))?;
            let granule = rest.get(1).map_or(Ok(480), |s| number(s))?;
            encode(input, output, bitrate, granule).map(|()| true)
        })(),
        ["compare", paths @ ..] if !paths.is_empty() => {
            compare(&paths.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        }
        _ => {
            eprintln!(
                "usage: eld-reference encode IN.wav OUT.rawpkts [BITRATE] [FRAME]\n       \
                 eld-reference compare STREAM.rawpkts...\n       \
                 eld-reference fixtures DIR"
            );
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
