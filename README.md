# AAC-ELD decoder in Rust

**This is a Third-Party Modified Version of the Fraunhofer FDK AAC Codec Library for
Android.** It is not Fraunhofer's library and not AOSP's: it is what is left of the
decoder after everything that AAC-ELD does not need was taken out, on 2026-09-21.

It decodes one thing: **ER AAC ELD** (MPEG-4 audio object type 39), mono or stereo,
without SBR, from raw access units with the AudioSpecificConfig supplied out of band.
That is what Apple Screen Sharing's system audio is, and it is the reason this
repository exists: neither a browser's WebCodecs nor FFmpeg's native decoder takes
object type 39.

```toml
aac = { git = "https://github.com/andrewtheguy/fdk-aac-rust", tag = "<tag>" }
```

```rust
use aac::aac_dec::AacDecoderInstance;

let mut decoder = AacDecoderInstance::new();
decoder.config_raw(&[0xf8, 0xe6, 0x50, 0x00])?; // 48 kHz, stereo, 480-sample frames

let mut pcm = [0.0f32; 2 * 480]; // interleaved, normalised to ±1
decoder.fill(access_unit, access_unit.len())?;
match decoder.decode(&mut pcm) {
    Ok(info) => { /* info.frame_size samples per channel in pcm */ }
    // A damaged access unit is concealed: pcm holds the substitute frame.
    Err((error, _)) if error.is_decode_error() => {}
    Err((error, _)) => return Err(error),
}
```

## Where it comes from

- Upstream: AOSP `platform/external/aac`, subdirectory `rust/` — Fraunhofer's own
  Rust port of the FDK AAC decoder
- Tag: `android-17.0.0_r1`, commit `41f344ffc0bacea87cac5bb1756bd40761265d1e`
- Source: <https://android.googlesource.com/platform/external/aac/+/refs/tags/android-17.0.0_r1/rust>

The tag `android-17.0.0_r1` of this repository is that tree, byte for byte.

## What was modified

Everything below was removed; nothing was added to the signal path, and what remains
produces the same samples as the unmodified decoder (see **Testing**).

- every audio object type but ER AAC ELD: AAC-LC, HE-AAC and HE-AAC v2 (SBR,
  parametric stereo), ER AAC LC, LD and scalable, and USAC (LPD/ACELP, arithmetic
  coding, complex prediction, noise filling, pre-roll)
- within ELD: low delay SBR, low delay MPEG surround, the downscaled mode, the error
  resilience tools (VCB11, RVLC, HCR), and channel configurations beyond stereo — a
  config that asks for any of them is refused as an unsupported format
- every transport but raw access units: ADTS, ADIF, LATM and LOAS
- everything after the core decoder: MPEG-4 and MPEG-D DRC, the PCM downmix, the
  limiter, ancillary data
- error concealment methods other than noise substitution, which is the one LD and
  ELD always resolved to, and the decoder parameters that only selected them
- the C bindings, the example decoder and the Android build file

The public interface is `aac::aac_dec::AacDecoderInstance`, its `AacDecoderError`
and its `OutputInfo`; the rest of the crate is private.

## Testing

`cargo test`. `tests/eld_golden.rs` decodes ten AAC-ELD streams, each once as encoded
and once with deterministic damage so that concealment runs, and compares a hash of
the `f32` output, the frame geometry and the error sequence with `tests/fixtures/golden.txt`
— which holds what the **unmodified** `android-17.0.0_r1` decoder produced for the
same input. The streams were encoded with FDK AAC 2.0.3 as AAC-ELD without SBR: 48 kHz
stereo with 480-sample frames at several bitrates and signal types (the Apple Screen
Sharing shape), plus 512-sample frames, 44.1 and 32 kHz, and mono.

To hear it, `cargo run --release --example decode -- STREAM.rawpkts OUT.wav` decodes a
stream to a 16-bit WAV; a `.rawpkts` file is a little-endian `i32` length and the
AudioSpecificConfig, then a length and an access unit, repeated.

`tools/reference` checks the decoder against Fraunhofer's C codec. It links FDK AAC
2.0.3 through [`fdk-aac-prebuilt`](https://github.com/andrewtheguy/fdk-aac-prebuilt),
pinned to a commit in `tools/reference/Cargo.toml`, so it is not part of the crate. The
archives themselves are not public: either the machine can reach their releases through
`gh`, or point `FDK_AAC_PREBUILT_DIR` at a prefix built with that repository's
`./build.sh`:

```sh
export FDK_AAC_PREBUILT_DIR=/path/to/fdk-aac-prebuilt/dist/linux-x86_64
alias eld-reference='cargo run --release --manifest-path tools/reference/Cargo.toml --'

# any audio, through the C encoder, as AAC-ELD without SBR (128 kbit/s, 480-sample frames)
ffmpeg -i IN -ar 48000 -ac 2 -c:a pcm_s16le in.wav
eld-reference encode in.wav in.rawpkts [BITRATE] [FRAME]

# this decoder against the C decoder, stream by stream
eld-reference compare tests/fixtures/*.rawpkts in.rawpkts

# the fixtures themselves, byte for byte
eld-reference fixtures tests/fixtures
```

The C decoder is 16-bit fixed point and this one is floating point, so `compare` passes
a stream whose outputs are within two 16-bit steps of each other or 70 dB apart; on the
fixtures they are 76 to 87 dB apart. It takes undamaged streams only, as the two
decoders conceal with different random noise.

## Licence

`NOTICE` — the "Software License for The Fraunhofer FDK AAC Codec Library for
Android", which every source file also carries. It is not OSI-approved and **grants
no patent licence**. It requires a modified version to say that it is one, which is
what the first paragraph of this file is for.
