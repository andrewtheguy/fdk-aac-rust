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
//! Audio specific config (ASC)

use super::super::error_codes::TpDecoderError;
use super::{
    eld_specific_config,
    helper_functions::{get_aot, get_sample_rate},
};
use crate::common::aot::AudioObjectType;
use crate::common::bitstream::Bitstream;
use crate::common::flags::ACFlags;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
/// Audio specific config structure.
pub struct AudioSpecificConfig {
    /// Audio object type.
    aot: AudioObjectType,
    /// Channel configuration index.
    channel_configuration: u8,
    /// Sampling frequency.
    sampling_frequency: u32,
    /// Sampling frequency index.
    sampling_frequency_index: u8,
    /// Audio codec flags.
    ac_flags: ACFlags,
}

impl AudioSpecificConfig {
    /// Returns new instance of `AudioSpecificConfig`.
    pub fn new() -> Self {
        AudioSpecificConfig::default()
    }

    /// Initialize an ASC structure.
    pub fn init(&mut self) {
        self.reset();
        self.aot = AudioObjectType::AotNone;
        self.sampling_frequency_index = 0xf;
        self.ac_flags = ACFlags::empty();
    }

    /// Resets the ASC instance.
    pub fn reset(&mut self) {
        self.channel_configuration = 0;
        self.sampling_frequency = 0;
    }

    /// Parses an AudioSpecificConfig. ER AAC ELD in mono or stereo is the one
    /// configuration taken; everything else is an unsupported format.
    ///
    /// # Parameters
    ///
    /// - `bs`: Bitstream instance with valid internal data.
    ///
    /// # Return
    ///
    /// - `TpDecoderError`.
    pub fn parse(&mut self, bs: &mut Bitstream) -> Result<(), TpDecoderError> {
        self.init();

        self.aot = get_aot(bs);
        if self.aot != AudioObjectType::AotErAacEld {
            return Err(TpDecoderError::UnsupportedFormat);
        }
        self.ac_flags.insert(ACFlags::ER | ACFlags::ELD);

        self.sampling_frequency = get_sample_rate(bs, Some(&mut self.sampling_frequency_index), 4);
        if self.sampling_frequency == 0 {
            return Err(TpDecoderError::ParseError);
        }

        self.channel_configuration = bs.read(4) as u8;
        if self.channel_configuration == 0 || self.channel_configuration > 2 {
            return Err(TpDecoderError::UnsupportedFormat);
        }

        eld_specific_config::parse(self.sampling_frequency, &mut self.ac_flags, bs)?;

        if bs.read(2) > 0 {
            // epConfig > 0 not supported
            return Err(TpDecoderError::UnsupportedFormat);
        }

        if bs.valid_bits() < 0 {
            return Err(TpDecoderError::NotEnoughBits);
        }

        Ok(())
    }

    /// Returns Audio Object Type.
    pub fn aot(&self) -> AudioObjectType {
        self.aot
    }

    /// Returns the channel configuration index.
    pub fn channel_config(&self) -> u8 {
        self.channel_configuration
    }

    /// Returns the sampling frequency.
    pub fn sampling_frequency(&self) -> u32 {
        self.sampling_frequency
    }

    /// Returns the sampling frequency index.
    pub fn sampling_frequency_index(&self) -> u8 {
        self.sampling_frequency_index
    }

    /// Returns samples per frame.
    pub fn samples_per_frame(&self) -> u16 {
        if self.ac_flags.contains(ACFlags::FRAME_LENGTH) {
            480
        } else {
            512
        }
    }

    /// Returns audio codec flags.
    pub fn ac_flags(&self) -> ACFlags {
        self.ac_flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::bitstream::Mode;

    fn parse(conf: &[u8]) -> (AudioSpecificConfig, Result<(), TpDecoderError>) {
        let mut bs = Bitstream::new(16, Mode::Reader);
        bs.init(conf, 8 * conf.len());
        let mut asc = AudioSpecificConfig::new();
        let result = asc.parse(&mut bs);
        (asc, result)
    }

    #[test]
    fn init() {
        let mut asc: AudioSpecificConfig = Default::default();
        asc.init();
        assert!(asc.sampling_frequency_index == 15);
        assert!(asc.aot == AudioObjectType::AotNone);
        assert!(asc.ac_flags == ACFlags::empty());
        assert!(asc.channel_configuration == 0);
        assert!(asc.sampling_frequency == 0);
    }

    #[test]
    fn eld_48k_stereo_480() {
        let (asc, result) = parse(&[0xf8, 0xe6, 0x50, 0x00]);
        assert_eq!(result, Ok(()));
        assert!(asc.aot() == AudioObjectType::AotErAacEld);
        assert_eq!(asc.sampling_frequency(), 48000);
        assert_eq!(asc.sampling_frequency_index(), 3);
        assert_eq!(asc.channel_config(), 2);
        assert_eq!(asc.samples_per_frame(), 480);
        assert!(asc.ac_flags() == ACFlags::ER | ACFlags::ELD | ACFlags::FRAME_LENGTH);
    }

    #[test]
    fn eld_48k_stereo_512() {
        let (asc, result) = parse(&[0xf8, 0xe6, 0x40, 0x00]);
        assert_eq!(result, Ok(()));
        assert_eq!(asc.samples_per_frame(), 512);
    }

    #[test]
    fn truncated() {
        let (_, result) = parse(&[0xf8, 0xe6, 0x50]);
        assert_eq!(result, Err(TpDecoderError::NotEnoughBits));
    }

    #[test]
    fn aac_lc() {
        let (_, result) = parse(&[0x12, 0x10]);
        assert_eq!(result, Err(TpDecoderError::UnsupportedFormat));
    }
}
