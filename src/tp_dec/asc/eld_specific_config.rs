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
//! Enhanced low delay (ELD) specific config

use super::super::error_codes::TpDecoderError;
use super::helper_functions::get_sample_rate;
use crate::common::{bitstream::Bitstream, flags::ACFlags};

#[derive(Debug, PartialEq)]
/// ELD extension type.
enum EldExtType {
    /// Terminator.
    Term = 0x0,
    /// Spatial audio object coding (SAOC).
    Saoc = 0x1,
    /// Low delay spatial audio coding (MPEG surround).
    Ldsac = 0x2,
    /// Downscale information.
    DownscaleInfo = 0x3,
    /// Unknown extension type.
    Unknown,
}

impl From<u32> for EldExtType {
    fn from(value: u32) -> Self {
        match value {
            0x0 => EldExtType::Term,
            0x1 => EldExtType::Saoc,
            0x2 => EldExtType::Ldsac,
            0x3 => EldExtType::DownscaleInfo,
            _ => EldExtType::Unknown,
        }
    }
}

/// Parse the ELD specific config.
///
/// The error resilience tools (VCB11, RVLC, HCR), low delay SBR, low delay MPEG
/// surround and the downscaled mode are not decoded here, so a config announcing
/// one of them is refused.
///
/// # Parameters
///
/// - `sampling_frequency`: Sampling frequency.
/// - `ac_flags`: Audio coding flags (see common/flags.rs).
/// - `bs`: Bitstream instance with valid internal data.
///
/// # Return
///
/// - `TpDecoderError`.
pub(super) fn parse(
    sampling_frequency: u32,
    ac_flags: &mut ACFlags,
    bs: &mut Bitstream,
) -> Result<(), TpDecoderError> {
    ac_flags.insert(if bs.read_bit() != 0 {
        ACFlags::FRAME_LENGTH
    } else {
        ACFlags::empty()
    });

    // aacSectionDataResilienceFlag, aacScalefactorDataResilienceFlag,
    // aacSpectralDataResilienceFlag and ldSbrPresentFlag
    if bs.read(4) != 0 {
        return Err(TpDecoderError::UnsupportedFormat);
    }

    let mut eld_ext_cnt = 0;
    let mut eld_ext_type = bs.read(4);

    // Parse ExtTypeConfigData.
    while EldExtType::from(eld_ext_type) != EldExtType::Term
        && bs.valid_bits() >= 0
        && eld_ext_cnt < 15
    {
        eld_ext_cnt += 1;

        let mut eld_ext_len = bs.read(4);
        let mut len = eld_ext_len;

        if len == 0xf {
            len = bs.read(8);
            eld_ext_len += len;

            if len == 0xff {
                len = bs.read(16);
                eld_ext_len += len;
            }
        }

        match EldExtType::from(eld_ext_type) {
            EldExtType::Ldsac => return Err(TpDecoderError::UnsupportedFormat),
            EldExtType::DownscaleInfo => {
                let downscaled_sampling_frequency = get_sample_rate(bs, None, 4);
                if downscaled_sampling_frequency == 0 {
                    return Err(TpDecoderError::ParseError);
                }

                if bs.read(4) != 0x0 {
                    return Err(TpDecoderError::ParseError);
                }

                if downscaled_sampling_frequency != sampling_frequency {
                    return Err(TpDecoderError::UnsupportedFormat);
                }
            }
            _ => bs.push((eld_ext_len * 8) as isize),
        };

        eld_ext_type = bs.read(4);
    }
    if EldExtType::from(eld_ext_type) != EldExtType::Term {
        return Err(TpDecoderError::ParseError);
    }

    Ok(())
}
