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
//! MPEG transport format decoder
//!
//! Raw access units are the one transport: the AudioSpecificConfig arrives out of
//! band, and every access unit is handed over whole.

// Modules
pub mod asc;
pub mod constants;
pub mod error_codes;

// Re-exports
pub use {
    asc::AudioSpecificConfig,
    constants::{MAX_CONF_SIZE, TRANSPORTDEC_INBUF_SIZE},
    error_codes::TpDecoderError,
};

// Imports
use crate::common::bitstream::{Bitstream, Mode};

// Enums
/// Reconfiguration states
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReconfigState {
    /// The decoder is asked whether the config differs from the one it runs with.
    DetCfgChange,
    /// The decoder is asked to allocate for the config and to take it over.
    AllocMem,
}

impl ReconfigState {
    /// The order a config is handed to the decoder in.
    pub fn all() -> [ReconfigState; 2] {
        [ReconfigState::DetCfgChange, ReconfigState::AllocMem]
    }
}

#[derive(Debug)]
/// Transport decoder structure.
pub struct TransportDec {
    /// Bitstream.
    pub bs: Bitstream,
    /// Indicates valid config and successful decoder initialisation.
    is_config_found: bool,
}

impl Default for TransportDec {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportDec {
    /// Returns new instance of `TransportDec`.
    pub fn new() -> Self {
        Self {
            bs: Bitstream::new(TRANSPORTDEC_INBUF_SIZE, Mode::Reader),
            is_config_found: false,
        }
    }

    /// Parses a binary coded AudioSpecificConfig.
    ///
    /// # Parameters
    ///
    /// - `conf`: u8 buffer of the binary coded config.
    ///
    /// # Return
    ///
    ///   - `AudioSpecificConfig`, or the `TpDecoderError` that refused it.
    pub fn parse_config(&self, conf: &[u8]) -> Result<AudioSpecificConfig, TpDecoderError> {
        if conf.len() > MAX_CONF_SIZE {
            return Err(TpDecoderError::UnsupportedFormat);
        }

        let mut bs = Bitstream::new(MAX_CONF_SIZE, Mode::Reader);
        bs.init(conf, 8 * conf.len());

        let mut asc = AudioSpecificConfig::new();
        asc.parse(&mut bs)?;

        Ok(asc)
    }

    /// Notes that the decoder took a config over, which is what lets access units in.
    pub fn set_config_found(&mut self) {
        self.is_config_found = true;
    }

    /// Fills the bitstream buffer with one access unit.
    ///
    /// # Parameters
    ///
    /// - `buffer`: Input buffer holding the access unit.
    /// - `bytes_valid`: Number of valid bytes in `buffer`.
    ///
    /// # Return
    ///
    ///   - `Ok(usize)`: Number of bytes left over, which is always zero.
    ///   - `Err((usize, TpDecoderError))`: The bytes that did not fit.
    pub fn fill_data(
        &mut self,
        buffer: &[u8],
        bytes_valid: usize,
    ) -> Result<usize, (usize, TpDecoderError)> {
        self.bs.reset();
        let valid_bytes = self.bs.feed(buffer, bytes_valid);
        if valid_bytes != 0 {
            return Err((valid_bytes, TpDecoderError::TooManyBits));
        }

        Ok(valid_bytes)
    }

    /// Takes the access unit in the buffer as the next one to decode.
    ///
    /// # Return
    ///
    ///   - `TpDecoderError`.
    pub fn read_access_unit(&mut self) -> Result<(), TpDecoderError> {
        if self.bs.valid_bits() <= 0 {
            return Err(TpDecoderError::NotEnoughBits);
        }

        if !self.is_config_found {
            // Decoder needs to be configured with out of band config.
            return Err(TpDecoderError::UnknownError);
        }

        Ok(())
    }

    /// Returns the number of bits left in the access unit, which is the whole buffer.
    pub fn remaining_au_bits(&mut self) -> isize {
        self.bs.valid_bits()
    }

    /// Returns mutable reference of the `Bitstream` data.
    pub fn bs_mut(&mut self) -> &mut Bitstream {
        &mut self.bs
    }

    /// Discards the buffered access unit.
    pub fn reset(&mut self) {
        self.bs.reset();
    }

    /// Returns the number of trailing bits that need to be consumed to finalize the AU parsing:
    /// those up to the next byte boundary.
    ///
    /// # Parameters
    ///
    /// - `au_start_anchor`: Bit buffer position at the beginning of the AU.
    ///
    /// # Return
    ///
    /// - `isize`: Number of trailing bits in AU.
    pub fn trailing_bits(&mut self, au_start_anchor: isize) -> isize {
        (self.bs.valid_bits() - au_start_anchor) & 7
    }
}
