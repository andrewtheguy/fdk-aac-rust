/* -----------------------------------------------------------------------------
Software License for The Fraunhofer FDK AAC Codec Library for Android

© Copyright 2025 Fraunhofer-Gesellschaft zur Förderung der angewandten Forschung
e.V. All rights reserved.

 1.    INTRODUCTION
The Fraunhofer FDK AAC Codec Library for Android ("FDK AAC Codec") is software
that implements the MPEG Advanced Audio Coding ("AAC") encoding and decoding
scheme for digital audio. This FDK AAC Codec software is intended to be used on
a wide variety of Android devices.

AAC's HE-AAC and HE-AAC v2 versions are regarded as today's most efficient
general perceptual audio codecs. AAC-ELD is considered the best-performing
full-bandwidth communications codec by independent studies and is widely
deployed. AAC has been standardized by ISO and IEC as part of the MPEG
specifications.

Patent licenses for necessary patent claims for the FDK AAC Codec (including
those of Fraunhofer) may be obtained through Via Licensing
(www.vialicensing.com) or through the respective patent owners individually for
the purpose of encoding or decoding bit streams in products that are compliant
with the ISO/IEC MPEG audio standards. Please note that most manufacturers of
Android devices already license these patent claims through Via Licensing or
directly from the patent owners, and therefore FDK AAC Codec software may
already be covered under those patent licenses when it is used for those
licensed purposes only.

Commercially-licensed AAC software libraries, including floating-point versions
with enhanced sound quality, are also available from Fraunhofer. Users are
encouraged to check the Fraunhofer website for additional applications
information and documentation.

2.    COPYRIGHT LICENSE

Redistribution and use in source and binary forms, with or without modification,
are permitted without payment of copyright license fees provided that you
satisfy the following conditions:

You must retain the complete text of this software license in redistributions of
the FDK AAC Codec or your modifications thereto in source code form.

You must retain the complete text of this software license in the documentation
and/or other materials provided with redistributions of the FDK AAC Codec or
your modifications thereto in binary form. You must make available free of
charge copies of the complete source code of the FDK AAC Codec and your
modifications thereto to recipients of copies in binary form.

The name of Fraunhofer may not be used to endorse or promote products derived
from this library without prior written permission.

You may not charge copyright license fees for anyone to use, copy or distribute
the FDK AAC Codec software or your modifications thereto.

Your modified versions of the FDK AAC Codec must carry prominent notices stating
that you changed the software and the date of any change. For modified versions
of the FDK AAC Codec, the term "Fraunhofer FDK AAC Codec Library for Android"
must be replaced by the term "Third-Party Modified Version of the Fraunhofer FDK
AAC Codec Library for Android."

3.    NO PATENT LICENSE

NO EXPRESS OR IMPLIED LICENSES TO ANY PATENT CLAIMS, including without
limitation the patents of Fraunhofer, ARE GRANTED BY THIS SOFTWARE LICENSE.
Fraunhofer provides no warranty of patent non-infringement with respect to this
software.

You may use this FDK AAC Codec software or modifications thereto only for
purposes that are authorized by appropriate patent licenses.

4.    DISCLAIMER

This FDK AAC Codec software is provided by Fraunhofer on behalf of the copyright
holders and contributors "AS IS" and WITHOUT ANY EXPRESS OR IMPLIED WARRANTIES,
including but not limited to the implied warranties of merchantability and
fitness for a particular purpose. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
CONTRIBUTORS BE LIABLE for any direct, indirect, incidental, special, exemplary,
or consequential damages, including but not limited to procurement of substitute
goods or services; loss of use, data, or profits, or business interruption,
however caused and on any theory of liability, whether in contract, strict
liability, or tort (including negligence), arising in any way out of the use of
this software, even if advised of the possibility of such damage.

5.    CONTACT INFORMATION

Fraunhofer Institute for Integrated Circuits IIS
Attention: Audio and Multimedia Departments - FDK AAC LL
Am Wolfsmantel 33
91058 Erlangen, Germany

www.iis.fraunhofer.de/amm
amm-info@iis.fraunhofer.de
----------------------------------------------------------------------------- */
//! Decoding of long and short blocks

use crate::aac_dec::{
    channel_info::IcsInfo, constants, error_codes::AacDecoderError, huff_dec::HuffmanDecoder,
    pns, pns::PnsData, utils::*,
};
use crate::common::bitstream::Bitstream;
use itertools::izip;

/// Reads section data
///
/// Reads the codebooks from `Bitstream` and its associated section length. For every scale
/// factor band the provided `code_book` slice gets filled on return.
/// While parsing the bitstream payload, `VCB11` extra treatment is considered, and `HCR`
/// related `hcr_data` may be updated.
///
/// # Parameters
///
/// - `bs`: Bitstream data to read from
/// - `ics_info`: Individual channel stream info data
/// - `code_book`: Code book description for each scale factor band filled on return
/// - `common_window`: Common windows signals whether stereo coding is possible
///
/// # Errors
///
/// Returns `AacDecoderError` type
/// - `AacDecOk` on success
/// - `AacDecInvalidCodeBook`, `AacDecDecodeFrameError`, `AacDecParseError` on failure
///
/// # Examples
///
/// ```
/// use aac::aac_dec::{block::read_section_data, channel_info::IcsInfo, constants};
/// use aac::common::bitstream::{Bitstream, Mode};
///
/// let bit_buffer = vec![0; 8];
/// let mut bs_reader = Bitstream::new(bit_buffer.len(), Mode::Reader);
/// bs_reader.init(&bit_buffer, 64);
/// let ics_info = IcsInfo::new();
/// let mut code_book = vec![0_u8; constants::MAX_WINS_X_SFBS];
/// let common_window = true;
///
/// let error_status = read_section_data(&mut bs_reader, &ics_info, &mut code_book, common_window);
/// ```
pub fn read_section_data(
    bs: &mut Bitstream,
    ics_info: &IcsInfo,
    code_book: &mut [u8],
    common_window: bool,
) -> AacDecoderError {
    let n_bits = if ics_info.is_long_block() { 5 } else { 3 };
    let sect_esc_val = (1 << n_bits) - 1;

    for code_book in code_book
        .chunks_exact_mut(constants::MAX_WINS_X_SFBS / ics_info.windows_per_frame())
        .take(ics_info.n_window_groups())
    {
        let mut band = 0;
        while band < ics_info.max_sf_bands() {
            let mut sect_len = 0;

            let sect_cb = bs.read(4) as u8;

            let mut sect_len_incr = bs.read(n_bits) as usize;
            while sect_len_incr == sect_esc_val {
                sect_len += sect_esc_val;
                sect_len_incr = bs.read(n_bits) as usize;
            }
            sect_len += sect_len_incr;

            if (band + sect_len) > code_book.len() {
                return AacDecoderError::DecodeFrameError;
            }

            // Check if decoded codebook index is feasible
            if (sect_cb == CBTYPE_BOOKSCL)
                || ((sect_cb == CBTYPE_INTENSITY_HCB2 || sect_cb == CBTYPE_INTENSITY_HCB)
                    && !common_window)
            {
                return AacDecoderError::InvalidCodeBook;
            }

            // Store codebook index
            code_book[band..band + sect_len].fill(sect_cb);
            band += sect_len;
        }
        code_book[band..].fill(CBTYPE_ZERO_HCB);
    }

    AacDecoderError::Ok
}

/// Reads scale factor data
///
/// Differential coded scale factors are read from `Bitstream` and fully decoded
/// with `HuffmanDecoder` by considering the `code_book` and additional side info.
/// Depending on the `code_book` the scale factors can also be used for intensity stereo
/// coding or perceptual noise substitution.
///
/// # Parameters
///
/// - `bs`: Bitstream data to read from
/// - `ics_info`: Individual channel stream info data
/// - `global_gain`: Global gain of the quantized spectrum
/// - `code_book`: Code book description for each scale factor band
/// - `scale_factor`: Scale factor data filled on return
/// - `pns`: Perceptual noise substitution related data updated on return
///
/// # Errors
///
/// Returns `AacDecoderError` type
/// - `AacDecOk` on success
/// - `AacDecParseError` on failure
///
/// # Examples
///
/// ```
/// use aac::aac_dec::{
///     block::read_scalefactor_data, channel_info::IcsInfo, constants, pns::PnsData,
/// };
/// use aac::common::bitstream::{Bitstream, Mode};
///
/// let bit_buffer = vec![0; 8];
/// let mut bs_reader = Bitstream::new(bit_buffer.len(), Mode::Reader);
/// bs_reader.init(&bit_buffer, 64);
/// let ics_info = IcsInfo::new();
/// let global_gain = 0_i16;
/// let code_book = vec![0_u8; constants::MAX_WINS_X_SFBS];
/// let mut scale_factor = vec![0_i16; constants::MAX_WINS_X_SFBS];
/// let mut pns_data = PnsData::default();
///
/// let error_status = read_scalefactor_data(
///     &mut bs_reader,
///     &ics_info,
///     global_gain,
///     &code_book,
///     &mut scale_factor,
///     &mut pns_data,
/// );
/// ```
pub fn read_scalefactor_data(
    bs: &mut Bitstream,
    ics_info: &IcsInfo,
    global_gain: i16,
    code_book: &[u8],
    scale_factor: &mut [i16],
    pns: &mut PnsData,
) -> AacDecoderError {
    let huff_dec = HuffmanDecoder::new(CBTYPE_BOOKSCL);
    let bands_per_window = constants::MAX_WINS_X_SFBS / ics_info.windows_per_frame();
    let mut factor = global_gain;
    let mut pns_nrg = global_gain - pns::NOISE_OFFSET - 256;
    let mut is_position = 0;

    for (group, (code_book, scale_factor)) in izip!(
        code_book.chunks_exact(bands_per_window),
        scale_factor.chunks_exact_mut(bands_per_window),
    )
    .take(ics_info.n_window_groups())
    .enumerate()
    {
        for band in 0..ics_info.max_sf_bands() {
            match code_book[band] {
                CBTYPE_ZERO_HCB => {
                    scale_factor[band] = 0;
                }
                CBTYPE_INTENSITY_HCB2 | CBTYPE_INTENSITY_HCB => {
                    let temp = huff_dec.read_word(bs);
                    is_position += temp - 60;
                    if !(-287..=287).contains(&is_position) {
                        return AacDecoderError::ParseError;
                    }
                    scale_factor[band] = is_position;
                }
                CBTYPE_NOISE_HCB => {
                    let temp = if pns.is_pns_active() {
                        huff_dec.read_word(bs) - 60
                    } else {
                        pns.set_is_pns_active(true);
                        bs.read(9) as i16
                    };
                    pns_nrg = pns::clamp_noise_range(pns_nrg + temp);
                    scale_factor[band] = pns_nrg;
                    pns.is_pns_used_mut()[group * bands_per_window + band] = true;
                }
                _ => {
                    let temp = huff_dec.read_word(bs);
                    factor += temp - 60;
                    if !(0..=255).contains(&factor) {
                        return AacDecoderError::ParseError;
                    }
                    scale_factor[band] = factor - 100;
                }
            }
        }
    }

    AacDecoderError::Ok
}

/// Reads huffman coded spectral lines
///
/// The `HuffmanDecoder` reads all huffman coded spectral lines from `Bitstream` by using
/// signaled `code_book` and its associated huffman table. The whole provided `spectrum` buffer
/// gets filled on return.
///
/// # Parameters
///
/// - `bs`: Bitstream data to read from
/// - `ics_info`: Individual channel stream info data
/// - `code_book`: Code book description for each scale factor band
/// - `spectrum`: Spectral data buffer filled on return
///
/// # Errors
///
/// Returns `AacDecoderError` type
/// - `AacDecOk` on success
///
/// # Examples
///
/// ```
/// use aac::aac_dec::{block::read_spectral_data, channel_info::IcsInfo, constants};
/// use aac::common::bitstream::{Bitstream, Mode};
///
/// let bit_buffer = vec![0; 8];
/// let mut bs_reader = Bitstream::new(bit_buffer.len(), Mode::Reader);
/// bs_reader.init(&bit_buffer, 64);
/// let ics_info = IcsInfo::new();
/// let code_book = vec![0_u8; constants::MAX_WINS_X_SFBS];
/// let mut spectrum = vec![0_i16; constants::MAX_FRAMESIZE];
///
/// let error_status = read_spectral_data(&mut bs_reader, &ics_info, &code_book, &mut spectrum);
/// ```
pub fn read_spectral_data(
    bs: &mut Bitstream,
    ics_info: &IcsInfo,
    code_book: &[u8],
    spectrum: &mut [i16],
) -> AacDecoderError {
    let sf_bands = ics_info.scale_factor_bands();
    let window_length = spectrum.len() / ics_info.windows_per_frame();
    let mut group_offset = 0;

    for (code_book, windows_per_group) in izip!(
        code_book.chunks_exact(constants::MAX_WINS_X_SFBS / ics_info.windows_per_frame()),
        ics_info.window_group_lengths().iter()
    )
    .take(ics_info.n_window_groups())
    {
        let mut stop_line = usize::from(sf_bands[0]);
        let mut band = 0;

        while band < ics_info.max_sf_bands() {
            // Group contiguous bands with same code book
            let mut current_cb = code_book[band];
            loop {
                band += 1;
                if !ics_info.is_long_block()
                    || band >= (ics_info.max_sf_bands())
                    || code_book[band] != current_cb
                {
                    break;
                }
            }
            let start_line = stop_line;
            stop_line = usize::from(sf_bands[band]);

            if (16..=31).contains(&current_cb) {
                current_cb = CBTYPE_ESCBOOK; // VCB11 has to use escape codebook
            }
            let huffman_decoder = if (1..=12).contains(&current_cb) {
                Some(HuffmanDecoder::new(current_cb))
            } else {
                None
            };

            for spectrum in spectrum[group_offset * window_length..]
                .chunks_exact_mut(window_length)
                .take((*windows_per_group).into())
            {
                if let Some(huff_dec) = huffman_decoder {
                    huff_dec.read_buf(bs, &mut spectrum[start_line..stop_line]);
                } else {
                    spectrum[start_line..stop_line].fill(0);
                };
            }
        }
        group_offset += usize::from(*windows_per_group);
    }

    AacDecoderError::Ok
}
