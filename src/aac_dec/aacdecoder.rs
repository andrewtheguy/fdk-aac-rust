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

use crate::{
    aac_dec::{
        config::Config,
        error_codes::AacDecoderError,
        interleaver,
        output_info::OutputInfo,
        params::Params,
        process::{InitState, Process},
    },
    common::{
        channel_map_descr::ChannelMapDescriptor,
        channel_order::ChannelOrder,
        flags::AACDecFlags,
    },
    tp_dec::{AudioSpecificConfig, ReconfigState, TransportDec},
};

/// AAC decoder.
#[repr(C)]
#[derive(Debug)]
pub struct AacDecoder {
    /// Configuration for the decoder.
    pub(super) config: Config,
    /// Describes the output channel mapping.
    pub(super) map_descr: ChannelMapDescriptor,
    /// AAC decoder instance.
    pub(super) aac_core: Process,
    /// Work buffer for core processing.
    pub(super) work_buffer_core: Vec<f32>,

    /// Work buffer for complete output.
    work_buffer_output: Vec<f32>,
}

impl Default for AacDecoder {
    fn default() -> Self {
        Self {
            config: Config::default(),
            map_descr: ChannelMapDescriptor::new(),
            aac_core: Process::new(),
            work_buffer_core: Default::default(),
            work_buffer_output: Default::default(),
        }
    }
}

impl AacDecoder {
    /// Creates a new `AacDecoder` instance.
    pub fn new() -> AacDecoder {
        let mut aac_dec = Self::default();

        aac_dec.map_descr.init(None, ChannelOrder::Wav);

        aac_dec
    }

    /// Frees what was allocated for the config the decoder runs with.
    pub(super) fn free_memory(&mut self) {
        self.aac_core.deinit();
        self.config = Config::default();

        if !self.work_buffer_core.is_empty() {
            self.work_buffer_core.clear();
            self.work_buffer_core.shrink_to_fit();
        }
    }

    /// Hands a parsed config to the decoder.
    ///
    /// # Parameters
    ///
    /// - `asc`: Audio specific config.
    /// - `config_mode`: Whether to detect a config change or to take the config over.
    /// - `is_config_changed`: Whether the config differs from the one the decoder runs with;
    ///   written when detecting, read when taking over.
    ///
    /// # Return
    ///
    /// - `Result<(), AacDecoderError>`
    pub(super) fn update_config(
        &mut self,
        asc: &AudioSpecificConfig,
        config_mode: ReconfigState,
        is_config_changed: &mut bool,
    ) -> Result<(), AacDecoderError> {
        let mut decoder_config = Config::default();
        if let Err(err) = decoder_config.init(asc) {
            if config_mode == ReconfigState::AllocMem {
                self.free_memory();
            }
            return Err(err);
        }

        // Detect config change.
        if config_mode == ReconfigState::DetCfgChange {
            *is_config_changed = decoder_config.is_config_change(&self.config);
        }

        // Initialize AAC core decoder, and update decoder config.
        if let Err(err) = self
            .aac_core
            .init(&decoder_config, config_mode, *is_config_changed)
        {
            if config_mode == ReconfigState::AllocMem {
                self.free_memory();
            }
            return Err(err);
        }

        if config_mode == ReconfigState::AllocMem {
            if self.work_buffer_core.is_empty() {
                let work_buffer_len = usize::from(decoder_config.num_channels)
                    * usize::from(decoder_config.frame_length);
                self.work_buffer_core = vec![0.0f32; work_buffer_len];
            }

            // Store new decoder config.
            self.config = decoder_config;
        }

        Ok(())
    }

    /// Creates output information based on the current decoder state and the given return value.
    ///
    /// # Parameters
    ///
    /// - `ret_val`: Return value of the last decoding operation.
    ///
    /// # Return
    ///
    /// - `Result<OutputInfo, (AacDecoderError, OutputInfo)>`
    fn create_output_info(
        &self,
        ret_val: Result<(), AacDecoderError>,
    ) -> Result<OutputInfo, (AacDecoderError, OutputInfo)> {
        self.aac_core
            .create_output_info(ret_val, &self.config, &self.map_descr)
    }

    /// Processes one frame of AAC data.
    ///
    /// # Parameters
    ///
    /// - `tp_dec_option`: Optional transport decoder.
    /// - `time_data`: Time domain audio samples (output buffer).
    /// - `params`: Decoder parameters.
    /// - `flags`: AAC Decoder flags.
    ///
    /// # Return
    ///
    /// - `Result<(), AacDecoderError>`
    pub(super) fn process(
        &mut self,
        tp_dec_option: Option<&mut TransportDec>,
        time_data: &mut [f32],
        params: &mut Params,
        flags: AACDecFlags,
    ) -> Result<OutputInfo, (AacDecoderError, OutputInfo)> {
        let mut error_status = AacDecoderError::Ok;
        let output_info;

        // Concealment and flushing are not possible at the same time.
        if flags.contains(AACDecFlags::CONCEAL | AACDecFlags::FLUSH) {
            return self.create_output_info(Err(AacDecoderError::Unknown));
        }

        // Decoder must at least be initialized by means of asc.
        if self.aac_core.init_state() == InitState::None {
            return self.create_output_info(Err(AacDecoderError::Unknown));
        }

        // Check and apply parameter changes.
        if let Err(e) = self.params_update(params) {
            return self.create_output_info(Err(e));
        }

        self.aac_core.reset();

        // Process AAC-core (parse access unit and extensions).
        match self.aac_core.decode_frame(
            tp_dec_option,
            &mut self.work_buffer_core,
            &mut self.map_descr,
            flags,
            &mut Some(&mut self.work_buffer_output),
            &mut self.config,
        ) {
            Ok(info) => {
                output_info = info;
            }
            Err((e, e_info)) => {
                output_info = e_info;
                error_status = e;
                if !e.is_output_valid() {
                    return self.create_output_info(Err(e));
                }
            }
        }

        // Check whether time data buffer is large enough.
        let buff_len = usize::from(output_info.num_channels) * usize::from(output_info.frame_size);
        if time_data.len() < buff_len {
            return self.create_output_info(Err(AacDecoderError::OutputBufferTooSmall));
        }

        // Interleave time data and adjust its scaling.
        let _ = interleaver::interleave_time_data(
            &mut time_data[..buff_len],
            &self.work_buffer_output[..buff_len],
            usize::from(output_info.num_channels),
            1.0_f32 / (1 << 15) as f32,
        );

        if error_status == AacDecoderError::Ok {
            Ok(output_info)
        } else {
            Err((error_status, output_info))
        }
    }

    /// Checks and applies parameter changes, if necessary. This function might re-allocate heap
    /// memory.
    ///
    /// # Parameters
    ///
    /// - `params`: Decoder parameters.
    ///
    /// # Return
    ///
    /// - `Result<(), AacDecoderError>`
    pub(super) fn params_update(&mut self, params: &mut Params) -> Result<(), AacDecoderError> {
        // In case of a config change restore all parameters.
        if self.aac_core.init_state() == InitState::Startup {
            params.restore();
        }

        self.aac_core.param_update(params, self.config.ac_flags)?;

        // Reallocate the output data workbuffer if necessary.
        self.reinit();

        if !self.map_descr.is_valid() {
            return Err(AacDecoderError::UnsupportedChannelconfig);
        }

        Ok(())
    }

    /// Re-initializes the output data workbuffer, if necessary. This function might re-allocate
    /// heap memory.
    pub(super) fn reinit(&mut self) {
        let max_output_frame_length = usize::from(self.config.samples_per_frame);
        let max_output_channels = usize::from(self.config.num_channels);

        if self.work_buffer_output.len() < (max_output_frame_length * max_output_channels) {
            self.work_buffer_output = vec![0_f32; max_output_frame_length * max_output_channels];
        }
    }

}
