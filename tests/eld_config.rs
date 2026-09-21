//! Which AudioSpecificConfigs the decoder takes: plain ER AAC ELD, mono or stereo,
//! and nothing else.

use aac::aac_dec::{AacDecoderError, AacDecoderInstance};

/// An ER AAC ELD config: object type 39, then the fields given, then
/// `ELDEXT_TERM` and `epConfig` 0.
fn eld(sampling_frequency_index: u32, channel_configuration: u32, eld_flags: u32) -> Vec<u8> {
    let mut bits: u64 = 0;
    let mut width = 0;
    let mut push = |value: u32, n: u32| {
        bits = (bits << n) | u64::from(value);
        width += n;
    };
    push(31, 5);
    push(39 - 32, 6);
    push(sampling_frequency_index, 4);
    push(channel_configuration, 4);
    // frameLengthFlag, the three resilience flags, ldSbrPresentFlag
    push(eld_flags, 5);
    push(0, 4);
    push(0, 2);
    let pad = (8 - width % 8) % 8;
    bits <<= pad;
    width += pad;
    (0..width / 8).rev().map(|i| (bits >> (8 * i)) as u8).collect()
}

fn config(asc: &[u8]) -> Result<(), AacDecoderError> {
    AacDecoderInstance::new().config_raw(asc)
}

#[test]
fn plain_eld_is_accepted() {
    // 48 kHz stereo with 480-sample frames: what Apple Screen Sharing sends.
    assert_eq!(eld(3, 2, 0b10000), [0xf8, 0xe6, 0x50, 0x00]);
    for sampling_frequency_index in [3, 4, 5] {
        for channel_configuration in [1, 2] {
            for frame_length_flag in [0, 1] {
                let asc = eld(sampling_frequency_index, channel_configuration, frame_length_flag << 4);
                assert_eq!(config(&asc), Ok(()), "{asc:02x?}");
            }
        }
    }
}

#[test]
fn everything_else_is_refused() {
    // AAC-LC, 44.1 kHz, stereo.
    assert_eq!(config(&[0x12, 0x10]), Err(AacDecoderError::UnsupportedFormat));
    // HE-AAC with explicit SBR signalling.
    assert_eq!(config(&[0x2b, 0x92, 0x08, 0x00]), Err(AacDecoderError::UnsupportedFormat));
    // ER AAC LD.
    assert_eq!(config(&[0xb9, 0x90, 0x00]), Err(AacDecoderError::UnsupportedFormat));
    // ELD with more than two channels, or a program config element.
    assert_eq!(config(&eld(3, 6, 0b10000)), Err(AacDecoderError::UnsupportedFormat));
    assert_eq!(config(&eld(3, 0, 0b10000)), Err(AacDecoderError::UnsupportedFormat));
    // ELD with VCB11, RVLC, HCR or low delay SBR.
    for flag in [0b01000, 0b00100, 0b00010, 0b00001] {
        assert_eq!(config(&eld(3, 2, 0b10000 | flag)), Err(AacDecoderError::UnsupportedFormat));
    }
}
