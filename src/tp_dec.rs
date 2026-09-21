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

// Modules
pub mod asc;
pub mod callbacks;
pub mod constants;
pub mod error_codes;
pub mod info;

// Re-exports
pub use {
    asc::AudioSpecificConfig,
    callbacks::TpDecCallBacks,
    constants::{MAX_CONF_SIZE, TRANSPORTDEC_INBUF_SIZE},
    error_codes::TpDecoderError,
};

// Imports
use crate::common::bitstream::{Bitstream, Mode};
use info::TpDecInfo;

// Enums
/// Reconfiguration states
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReconfigState {
    /// No config mode set at all.
    None = 0x00,
    /// Config mode signalizes the callback to work in config change detection mode.
    DetCfgChange = 0x01,
    /// Config mode signalizes the callback to work in memory allocation mode.
    AllocMem = 0x02,
}

impl From<ReconfigState> for u8 {
    fn from(state: ReconfigState) -> Self {
        match state {
            ReconfigState::None => 0x00,
            ReconfigState::DetCfgChange => 0x01,
            ReconfigState::AllocMem => 0x02,
        }
    }
}

impl TryFrom<u8> for ReconfigState {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(ReconfigState::None),
            0x01 => Ok(ReconfigState::DetCfgChange),
            0x02 => Ok(ReconfigState::AllocMem),
            _ => Err("invalid u8 value to convert to ReconfigState."),
        }
    }
}

impl ReconfigState {
    fn all() -> [ReconfigState; 2] {
        [ReconfigState::DetCfgChange, ReconfigState::AllocMem]
    }
}

// Structs
#[derive(Default, Debug)]
#[repr(C)]
/// Transport Decoder Data structure.
pub struct TpDecData {
    /// Audio specific config from the last config found.
    asc: AudioSpecificConfig,
    /// Transport decoder frame info.
    info: TpDecInfo,
    /// Indicates valid config and successful decoder initialisation.
    is_config_found: bool,
}

impl TpDecData {
    /// Returns new instance of `TpDecData`.
    pub fn new() -> TpDecData {
        TpDecData::default()
    }


    /// Initialises `TpDecData`.
    pub(super) fn init(&mut self) {
        self.info.init();
        self.asc.init();
    }

    /// Deinitialises `TpDecData`.
    pub fn deinit(&mut self) {
        self.asc.reset();
        self.info.reset();
        self.is_config_found = false;
    }

    /// Notes the access unit that was filled into the buffer.
    ///
    /// # Parameters
    ///
    /// - `bs`: Bitstream instance with valid internal data.
    ///
    /// # Return
    ///
    /// - `TpDecoderError`.
    fn read_header(&mut self, bs: &mut Bitstream) -> Result<(), TpDecoderError> {
        let mut err = Ok(());

        if bs.valid_bits() <= 0 {
            err = Err(TpDecoderError::NotEnoughBits);
        } else if !self.is_config_found {
            // Decoder needs to be configured with out of band config.
            err = Err(TpDecoderError::UnknownError);
        } else {
            // One Access Unit was filled into buffer, so get the length out of the
            // buffer.
            self.info.set_au_length(bs.valid_bits().try_into().unwrap());
            self.info.set_access_unit_anchor(bs.valid_bits());
        }

        if err.is_err() {
            self.info.reset();
        }

        err
    }

    /// Resets the `TpDecData`.
    fn reset(&mut self) {
        self.info.reset();
    }
}

#[repr(C)]
#[derive(Debug)]
/// Transport decoder structure.
pub struct TransportDec {
    /// Bitstream.
    pub bs: Bitstream,
    /// Transport decoder callbacks.
    cb: TpDecCallBacks,
    /// Transport Decoder Data.
    pub data: TpDecData,
}

impl Default for TransportDec {
    /// Returns default instance of `TransportDec`.
    fn default() -> Self {
        Self {
            bs: Default::default(),
            cb: Default::default(),
            data: Default::default(),
        }
    }
}

impl TransportDec {
    /// Returns new instance of `TransportDec`.
    pub fn new() -> Self {
        Self::default()
    }


    /// Initialises `TransportDec`.
    pub fn init(&mut self) {
        self.bs = Bitstream::new(TRANSPORTDEC_INBUF_SIZE, Mode::Reader);
        self.data.init();
    }

    /// Configures TransportDec via a binary coded AudioSpecificConfig or StreamMuxConfig.
    ///
    /// # Parameters
    ///
    /// - `conf`: u8 buffer of the binary coded config (ASC or SMC).
    ///
    /// # Return
    ///
    ///   - `TpDecoderError`.
    pub fn out_of_band_config(&mut self, conf: &[u8]) -> Result<(), TpDecoderError> {
        let mut err = Ok(());
        let mut config_found = false;

        if conf.len() > MAX_CONF_SIZE {
            return Err(TpDecoderError::UnsupportedFormat);
        }

        let mut bs = Bitstream::new(MAX_CONF_SIZE, Mode::Reader);
        bs.init(conf, 8 * conf.len());

        for config_mode in ReconfigState::all() {
            if config_mode == ReconfigState::AllocMem {
                let num_bits = (conf.len() * 8) as isize - bs.valid_bits();
                bs.push(-num_bits);
            }
            self.cb.set_config_mode(config_mode);

            // Config transport decoder.
            let mut dummy_asc = AudioSpecificConfig::new();
            dummy_asc.init();

            err = dummy_asc.parse(&mut bs);

            if err.is_ok() {
                if self.cb.update_config_callback(&dummy_asc).is_err() {
                    err = Err(TpDecoderError::ParseError);
                    break;
                }
                self.data.asc = dummy_asc;
                config_found = true;
            }

            if err.is_ok()
                && (config_mode == ReconfigState::DetCfgChange)
                && self.cb.is_config_changed()
                && self.cb.free_mem_callback().is_err()
            {
                err = Err(TpDecoderError::ParseError);
            }

            // If an error is detected terminate config parsing to avoid that an invalid
            // config is accepted in the second pass.
            if err.is_err() {
                break;
            }
        }

        if err.is_ok() && config_found {
            self.data.is_config_found = true;
        }

        err
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

    /// Returns reference to a callback.
    pub fn callback(&mut self) -> &mut TpDecCallBacks {
        &mut self.cb
    }

    /// Takes the access unit in the buffer as the next one to decode.
    ///
    /// # Return
    ///
    ///   - `TpDecoderError`.
    pub fn read_access_unit(&mut self) -> Result<(), TpDecoderError> {
        if self.bs.valid_bits() <= 0 {
            self.data.reset();
            return Err(TpDecoderError::NotEnoughBits);
        }

        let mut err = self.data.read_header(&mut self.bs);
        if err == Err(TpDecoderError::NotEnoughBits) {
            err = Err(TpDecoderError::SyncError);
        }
        if err.is_ok() {
            self.data.reset();
        }

        err
    }

    /// Returns the remaining amount of bits of the current access unit. The result can be below
    /// zero, meaning that too many bits have been read.
    pub fn remaining_au_bits(&mut self) -> isize {
        let valid_bits = self.bs.valid_bits();

        if self.data.info.access_unit_anchor() > 0
            && self.data.info.au_length() > 0
            && valid_bits >= 0
        {
            return isize::try_from(self.data.info.au_length()).unwrap()
                - (self.data.info.access_unit_anchor() - valid_bits);
        }

        valid_bits
    }

    /// Returns the total amount of bits of the current access unit.
    pub fn total_au_bits(&self) -> i32 {
        self.data.info.au_length()
    }

    /// Returns a mutable reference to `Bitstream`.
    pub fn bs_mut(&mut self) -> &mut Bitstream {
        &mut self.bs
    }

    /// Discards the buffered access unit.
    pub fn reset(&mut self) {
        self.bs.reset();
        self.data.reset();
    }

    /// Returns the number of trailing bits that need to be consumed to finalize the AU parsing.
    ///
    /// # Parameters
    ///
    /// - `au_start_anchor`: Bit buffer position at the beginning of the AU.
    ///
    /// # Return
    ///
    /// - `isize`: Number of trailing bits in AU.
    pub fn trailing_bits(&mut self, au_start_anchor: isize) -> isize {
        let valid_bits = self.bs.valid_bits();

        if self.total_au_bits() > 0 {
            self.remaining_au_bits()
        } else {
            (valid_bits - au_start_anchor) & 7
        }
    }

}
