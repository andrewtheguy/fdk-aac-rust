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
//! Advanced audio coding (AAC) decoder

// Modules
pub mod aacdecoder;
pub mod block;
pub mod channel;
pub mod channel_info;
pub mod conceal;
pub mod config;
pub mod constants;
pub mod error_codes;
pub mod huff_dec;
pub mod intensity;
pub mod interleaver;
pub mod inverse_quantization;
pub mod ms_stereo;
pub mod output_info;
pub mod params;
pub mod pns;
pub mod process;
pub mod sr_info;
pub mod tns;
pub mod utils;

// Re-exports
pub use crate::{
    aac_dec::{constants::MAX_CHANNELS, error_codes::AacDecoderError, output_info::OutputInfo},
    common::audio_channel_type::AudioChannelType,
    tp_dec::{MAX_CONF_SIZE, TRANSPORTDEC_INBUF_SIZE},
};

// Imports
use crate::{
    aac_dec::{aacdecoder::AacDecoder, conceal::A_CONCEAL_AU, params::Params},
    common::{bitstream::Bitstream, flags::AACDecFlags},
    tp_dec::{ReconfigState, TransportDec},
};

/// AAC decoder instance.
#[derive(Debug)]
pub struct AacDecoderInstance {
    /// Parameters for the AAC decoder.
    params: Params,

    /// AAC decoder handle.
    aac_decoder: AacDecoder,

    /// Transport decoder handle.
    transport_decoder: TransportDec,
}

impl Default for AacDecoderInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl AacDecoderInstance {
    /// Creates a new `AacDecoderInstance` instance.
    pub fn new() -> AacDecoderInstance {
        AacDecoderInstance {
            params: Params::new(),
            aac_decoder: AacDecoder::new(),
            transport_decoder: TransportDec::new(),
        }
    }

    /// Decodes an `AAC` frame. The decoded output signal is stored in `time_data`.
    ///
    /// # Parameters
    ///
    /// - `time_data`: External output buffer, where the decoded PCM samples will be stored into.
    ///
    /// # Return
    ///
    /// - `Result<OutputInfo, (AacDecoderError, OutputInfo)>`
    ///   - `Ok(output_info)`: If decoding is successful, it returns the information which
    ///     describes the output signal.
    ///   - `Err((error, output_info))`: If decoding fails, it returns an error and the output
    ///     information. A decode error (`AacDecoderError::is_decode_error`) still leaves a
    ///     frame in `time_data`: the one error concealment produced.
    pub fn decode(
        &mut self,
        time_data: &mut [f32],
    ) -> Result<OutputInfo, (AacDecoderError, OutputInfo)> {
        if Self::is_concealment_au(self.transport_decoder.bs_mut()) {
            // Conceal frame if concealment byte sequence matches.
            return self.conceal(time_data);
        }

        //  Read transport header.
        if let Err(e) = self.transport_decoder.read_access_unit() {
            return Err((AacDecoderError::from(e), OutputInfo::default()));
        }

        // Decode bistream
        self.aac_decoder.process(
            Some(&mut self.transport_decoder),
            time_data,
            &mut self.params,
            AACDecFlags::empty(),
        )
    }

    /// Fills AAC decoder's internal input buffer with one access unit.
    ///
    /// # Parameters
    ///
    /// - `buffer`: External input buffer.
    /// - `bytes_valid`: Number of bitstream bytes in the external bitstream buffer.
    ///
    /// # Return
    ///
    /// - `Result<usize, AacDecoderError>`.
    ///   - `Ok(usize)`: Number of remaining valid bytes in the external bitstream buffer, which
    ///     is zero.
    ///   - `AacDecoderError`: AAC decoder error.
    pub fn fill(&mut self, buffer: &[u8], bytes_valid: usize) -> Result<usize, AacDecoderError> {
        match self.transport_decoder.fill_data(buffer, bytes_valid) {
            Ok(valid_bytes) => Ok(valid_bytes),
            Err((_remain_bytes, e)) => Err(e.into()),
        }
    }

    /// Clears internal bit stream buffer of transport layers.
    /// The decoder starts decoding at new data passed after this event
    /// and any previous bit stream data is discarded.
    pub fn clear(&mut self) {
        self.transport_decoder.reset();
    }

    /// Returns true if the AU concealment byte sequence is found in the given bitstream. See
    /// `aac::aac_dec::conceal::conceal_constants::A_CONCEAL_AU`.
    fn is_concealment_au(bs: &mut Bitstream) -> bool {
        if bs.valid_bits() >= (8 * A_CONCEAL_AU.len()) as isize {
            let mut is_conceal_au = true;
            for (i, &byte) in A_CONCEAL_AU.iter().enumerate() {
                if bs.read(8) != u32::from(byte) {
                    is_conceal_au = false;
                    bs.push(-8 * (i + 1) as isize);
                    break;
                }
            }
            is_conceal_au
        } else {
            false
        }
    }

    /// Explicitly configures the decoder by passing a raw AudioSpecificConfig
    /// (ASC), contained in a binary buffer. ER AAC ELD in mono or stereo, without
    /// SBR, is the one configuration taken.
    ///
    /// # Parameters
    ///
    /// - `conf`: Buffer containing the binary configuration.
    ///
    /// # Return
    ///
    /// - `Result<(), AacDecoderError>`.
    pub fn config_raw(&mut self, conf: &[u8]) -> Result<(), AacDecoderError> {
        let asc = self.transport_decoder.parse_config(conf)?;

        let mut is_config_changed = false;
        for config_mode in ReconfigState::all() {
            self.aac_decoder
                .update_config(&asc, config_mode, &mut is_config_changed)?;

            if config_mode == ReconfigState::DetCfgChange && is_config_changed {
                self.aac_decoder.free_memory();
            }
        }

        self.transport_decoder.set_config_found();

        Ok(())
    }

    /// Flushes all filterbanks to get all delayed audio without having new input
    /// data. New input data will not be considered.
    ///
    /// # Parameters
    ///
    /// - `time_data`: Time domain input/ouptut data buffer.
    ///
    /// # Return
    ///
    /// - `Result<OutputInfo, (AacDecoderError, OutputInfo)>`.
    pub fn drain(
        &mut self,
        time_data: &mut [f32],
    ) -> Result<OutputInfo, (AacDecoderError, OutputInfo)> {
        self.clear();

        self.aac_decoder
            .process(None, time_data, &mut self.params, AACDecFlags::FLUSH)
    }

    /// Triggers the built-in error concealment to generate substitute signal for
    /// one lost frame. New input data will not be considered.
    ///
    /// # Parameters
    ///
    /// - `time_data`: Time domain input/ouptut data buffer.
    ///
    /// # Return
    ///
    /// - `Result<OutputInfo, (AacDecoderError, OutputInfo)>`.
    pub fn conceal(
        &mut self,
        time_data: &mut [f32],
    ) -> Result<OutputInfo, (AacDecoderError, OutputInfo)> {
        self.aac_decoder
            .process(None, time_data, &mut self.params, AACDecFlags::CONCEAL)
    }
}
