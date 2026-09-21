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
//! AAC decoder configuration

use super::constants::*;
use super::error_codes::AacDecoderError;
use crate::common::aot::AudioObjectType;
use crate::common::audio_channel_type::AudioChannelType;
use crate::common::bs_element_id::ChannelElementId;
use crate::common::flags::*;
use crate::tp_dec::AudioSpecificConfig;
use itertools::izip;

/// Element configuration.
#[repr(C)]
#[derive(Default, Debug, PartialEq)]
pub(super) struct ElementConfig {
    /// Element type.
    pub(super) element_type: ChannelElementId,
}

/// AAC decoder configuration, derived from the parsed ASC.
#[repr(C)]
#[derive(Default, Debug)]
pub struct Config {
    /// Element-wise configuration, such as the element ID and flags.
    pub(super) element_config: [ElementConfig; MAX_ELEMENTS],
    /// Audio channel type of each output audio channel.
    pub(super) channel_type: [AudioChannelType; MAX_CHANNELS],
    /// Audio channel index for each output audio channel.
    pub channel_indices: [u8; MAX_CHANNELS],
    /// Index to access one line of the channel map table.
    pub channel_map_index: u8,
    /// Number of bitstream elements.
    pub(super) num_elements: u8,
    /// Number of channel elements, i.e. the SCE or the CPE.
    pub(super) num_channel_elements: u8,
    /// Total number of channels given by the ASC.
    pub num_channels: u8,
    /// Channel config index as given by the ASC.
    pub(super) channel_config: u8,
    /// Audio codec flags.
    pub ac_flags: ACFlags,
    /// Audio object type.
    pub(super) aot: AudioObjectType,
    /// Frame length of the core decoder, which is also the number of samples per
    /// channel in one decoded frame.
    pub frame_length: u16,
    /// Samples per output frame.
    pub(super) samples_per_frame: u16,
    /// Sampling frequency of the decoder output.
    pub(super) sampling_frequency: u32,
    /// Sampling frequency index.
    pub(super) sampling_frequency_index: u8,
}

impl Config {
    /// Initializes the AAC decoder `Config` struct.
    ///
    /// # Parameters
    ///
    /// - `asc`: Audio specific config, which holds ER AAC ELD in mono or stereo.
    ///
    /// # Return
    ///
    /// - `Result<(), AacDecoderError>`
    pub fn init(&mut self, asc: &AudioSpecificConfig) -> Result<(), AacDecoderError> {
        *self = Default::default();

        // Build element table: the one channel element, then the ER extension
        // element and the terminator.
        let channel_element = match asc.channel_config() {
            1 => ChannelElementId::Sce,
            2 => ChannelElementId::Cpe,
            _ => return Err(AacDecoderError::UnsupportedChannelconfig),
        };
        self.element_config[0].element_type = channel_element;
        self.element_config[1].element_type = ChannelElementId::Ext;
        self.element_config[2].element_type = ChannelElementId::End;
        self.num_channel_elements = 1;
        self.num_elements = 3;

        // Set number of channels.
        self.num_channels = asc.channel_config();

        // Set channel map index according to given channel config.
        self.channel_map_index = asc.channel_config();

        // Set channel description (type and index)
        for (ch_index, (channel_type, channel_indices)) in izip!(
            self.channel_type.iter_mut(),
            self.channel_indices.iter_mut(),
        )
        .take(self.num_channels.into())
        .enumerate()
        {
            *channel_type = AudioChannelType::Front;
            *channel_indices = ch_index as u8;
        }

        self.channel_config = asc.channel_config();
        self.ac_flags = asc.ac_flags();

        self.aot = asc.aot();
        self.frame_length = asc.samples_per_frame();
        self.samples_per_frame = asc.samples_per_frame();

        self.sampling_frequency = asc.sampling_frequency();
        self.sampling_frequency_index = asc.sampling_frequency_index();

        self.check_sampling_rate()?;

        Ok(())
    }

    /// Compares two AAC decoder config structs and determine whether the decoder's
    /// configuration has changed.
    ///
    /// # Parameters
    ///
    /// - `config2`: Another AAC decoder config.
    ///
    /// # Return
    ///
    /// - `true` if configuration changed, otherwise `false`.
    pub fn is_config_change(&self, config2: &Self) -> bool {
        if (self.sampling_frequency != config2.sampling_frequency)
            || (self.samples_per_frame != config2.samples_per_frame)
        {
            return true;
        }

        if self.channel_config != config2.channel_config {
            return true;
        }

        if self.aot != config2.aot {
            return true;
        }

        false
    }

    /// Checks if the given sampling frequency is supported.
    fn check_sampling_rate(&self) -> Result<(), AacDecoderError> {
        // Verify if the sampling frequency is among the supported values.
        let supported_frequencies = [
            96000, 88200, 64000, 16000, 12000, 11025, 8000, 7350, 48000, 44100, 32000, 24000,
            22050,
        ];
        if !supported_frequencies.contains(&self.sampling_frequency) {
            return Err(AacDecoderError::UnsupportedSamplingrate);
        }

        Ok(())
    }
}
