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
//! Decoding of one audio channel

use super::{
    block,
    channel_info::IcsInfo,
    conceal::{AacDecoderRenderMode, ConcealmentData},
    constants,
    error_codes::AacDecoderError,
    intensity, inverse_quantization,
    ms_stereo::JointStereoData,
    pns::{PnsData, PnsInterChannelData},
    sr_info::SamplingRateInfo,
    tns::TnsData,
};
use crate::common::{
    bs_element_id::ChannelElementId,
    enums::AudioChannel,
    flags::{ACFlags, ChannelFlags},
    ld_filter_bank::{self},
};
use crate::tp_dec::TransportDec;
use itertools::izip;
use std::mem::size_of;
use zerocopy::FromBytes;

/// Size of work buffer section.
const WB_SECTION_SIZE: usize = constants::MAX_FRAMESIZE * 2 * size_of::<u32>();

// -- Common data structures -- //

use bitflags::bitflags;

bitflags! {
    /// States of channel element.
    #[repr(C)]
    #[derive(Copy, Clone, Default, Debug)]
    pub struct ElState: u32 {
        /// Channel element data has been read from bitstream.
        const READ = 1;
        /// Channel element spectral data decoded and is ready for output rendering.
        const DECODED = 2;
    }
}

/// Common Channel Data.
///
/// This structure is used for common data of CPE/SCE.
#[repr(C, align(4))]
#[derive(Debug)]
pub struct CommonChannelData {
    // The scratch_work_buffer is re-used as a [f32] and an [i16] buffer. Therefore, always make
    // sure that the buffer is memory aligned with std::mem::align_of::<f32>() and
    // std::mem::align_of::<i16>()
    scratch_work_buffer: [u8; WB_SECTION_SIZE],
    sr_info: SamplingRateInfo,
}

impl Default for CommonChannelData {
    // Default trait.
    fn default() -> Self {
        Self {
            scratch_work_buffer: [0_u8; WB_SECTION_SIZE],
            sr_info: SamplingRateInfo::default(),
        }
    }
}

impl CommonChannelData {
    /// Returns default initialized CommonChannelData instance.
    pub fn new() -> Self {
        Default::default()
    }

    /// Initializes CommonChannelData instance.
    ///
    /// # Parameters
    ///
    /// - `sr_info`: Sampling rate information.
    pub fn init(&mut self, sr_info: &SamplingRateInfo) {
        self.sr_info.clone_from(sr_info);
    }
}

/// Channel Data.
///
/// This structure is used for an audio channel.
#[repr(C)]
#[derive(Debug)]
pub struct ChannelInfo {
    pub ics_info: IcsInfo,
    // -- Common bit stream data -- //
    tns_data: TnsData,
    pns_data: PnsData,
    eld_overlap_buffer: Vec<f32>,
    // Spectral scale factors for each sfb in each  window.
    scale_factor: [i16; constants::MAX_WINS_X_SFBS],
    // section data: codebook for each window and sfb.
    code_book: [u8; constants::MAX_WINS_X_SFBS],
    // Output signal rendering mode
    render_mode: AacDecoderRenderMode,
    global_gain: u8,
}

impl ChannelInfo {
    /// Returns default initialized ChannelInfo instance.
    pub fn new() -> Self {
        ChannelInfo::default()
    }
}

impl Default for ChannelInfo {
    // Default trait.
    fn default() -> Self {
        Self {
            scale_factor: [0_i16; constants::MAX_WINS_X_SFBS],
            code_book: [0_u8; constants::MAX_WINS_X_SFBS],
            ics_info: Default::default(),
            render_mode: AacDecoderRenderMode::default(),
            global_gain: Default::default(),
            tns_data: Default::default(),
            pns_data: PnsData::new(),
            eld_overlap_buffer: Default::default(),
        }
    }
}

/// ChannelElement Data.
///
/// This structure is used for a channel element. For example: SCE/CPE
#[repr(C)]
#[derive(Default, Debug)]
pub struct ChannelElement {
    js_data: Option<Box<JointStereoData>>,
    pns_inter_channel_data: PnsInterChannelData,
    channel_info: Vec<ChannelInfo>,
    prev_spectral_data: Vec<f32>,
    common_window: bool,
    num_channels: u8,
    element_type: ChannelElementId,
    el_flags: ChannelFlags,
    state_flags: ElState,
}

impl ChannelElement {
    /// Returns default initialized ChannelElement instance.
    pub fn new() -> Self {
        ChannelElement::default()
    }

    /// Initializes ChannelElement instance.
    ///
    /// # Parameters
    ///
    /// - `sr_info`: Sampling rate information.
    /// - `element_type`: Type of the element (example: SCE/CPE).
    /// - `frame_length`: Input frame length.
    /// - `element_flags`: Element flags.
    /// - `ac_flags`: Audio codec flags.
    pub fn init(
        &mut self,
        sr_info: &SamplingRateInfo,
        element_type: ChannelElementId,
        frame_length: usize,
        element_flags: ChannelFlags,
        ac_flags: ACFlags,
    ) {
        let num_channels = Self::get_num_channels(element_type);

        if num_channels > 1 {
            self.js_data = Some(Box::new(JointStereoData::new()));
        }

        // Allocate memory for channel_info.
        self.channel_info = Vec::new();
        for _ch in 0..num_channels {
            self.channel_info.push(ChannelInfo::new());
        }

        for channel_info in self.channel_info.iter_mut() {
            channel_info.eld_overlap_buffer = vec![0.0_f32; frame_length + (frame_length / 2)];
            channel_info.render_mode = AacDecoderRenderMode::EldFb;
            channel_info
                .tns_data
                .init(sr_info.sampling_rate_index(), ac_flags);
        }

        self.prev_spectral_data = vec![0.0_f32; num_channels * frame_length];
        self.num_channels = num_channels as u8;
        self.element_type = element_type;
        self.el_flags = element_flags;
    }

    /// Clears the element flags.
    pub fn clear_flags(&mut self) {
        self.el_flags = ChannelFlags::empty();
    }

    /// Set element flags.
    pub fn insert_flags(&mut self, flags_to_set: ChannelFlags) {
        self.el_flags.insert(flags_to_set);
    }

    /// Returns the number of channels in the channel element.
    pub fn num_channels(&self) -> u8 {
        self.num_channels
    }

    /// Returns the state flags of the channel element.
    pub fn state_flags(&self) -> ElState {
        self.state_flags
    }

    /// Returns number of channels for the channel element.
    ///
    /// # Parameters
    ///
    /// - `element_type`: Type of the element (example: SCE/CPE).
    pub(super) fn get_num_channels(element_type: ChannelElementId) -> usize {
        match element_type {
            ChannelElementId::Cpe => 2,
            ChannelElementId::Sce => 1,
            _ => 0,
        }
    }

    /// Resets the channel element.
    pub fn reset(&mut self) {
        for ci in self
            .channel_info
            .iter_mut()
            .take(self.num_channels as usize)
        {
            // Reset TNS Data.
            ci.tns_data.reset();
            ci.pns_data.init();
        }

        // The channels of an ELD channel pair always share their window.
        self.common_window = self.num_channels == 2;

        if self.num_channels > 1 {
            // Reset ms_stereo data.
            if let Some(js_data) = self.js_data.as_deref_mut() {
                js_data.reset();
            }
        }

        self.state_flags = ElState::empty();
    }

    /// Reads the side information and the spectral data of one channel from the
    /// bitstream and inverse quantizes the spectrum.
    fn read_channel(
        &mut self,
        common_channel_data: &mut CommonChannelData,
        tp_dec: &mut TransportDec,
        spectral_data: &mut [f32],
        channel: usize,
        ac_flags: ACFlags,
    ) -> Result<(), AacDecoderError> {
        fn check(error: AacDecoderError) -> Result<(), AacDecoderError> {
            if error == AacDecoderError::Ok {
                Ok(())
            } else {
                Err(error)
            }
        }

        let bs = &mut tp_dec.bs;
        let frame_length = spectral_data.len();
        let quantized_spectrum =
            <[i16]>::mut_from_bytes(&mut common_channel_data.scratch_work_buffer)
                .expect("Cannot interpret the scratch buffer as i16 slice");

        if self.num_channels == 1 {
            self.channel_info[channel].global_gain = bs.read(8) as u8;

            // Read individual channel info.
            check(self.channel_info[channel].ics_info.read(
                bs,
                &common_channel_data.sr_info,
                ac_flags,
            ))?;
        } else {
            self.channel_info[channel].global_gain = bs.read(8) as u8;
        }

        let ch_info = &mut self.channel_info[channel];

        check(block::read_section_data(
            bs,
            &ch_info.ics_info,
            &mut ch_info.code_book,
            self.common_window,
        ))?;

        check(block::read_scalefactor_data(
            bs,
            &ch_info.ics_info,
            ch_info.global_gain.into(),
            &ch_info.code_book,
            &mut ch_info.scale_factor,
            &mut ch_info.pns_data,
        ))?;

        ch_info.tns_data.read_datapresent_flag(bs);
        check(ch_info.tns_data.read(bs, &ch_info.ics_info))?;

        check(block::read_spectral_data(
            bs,
            &ch_info.ics_info,
            &ch_info.code_book,
            &mut quantized_spectrum[..frame_length],
        ))?;

        let mut band_is_noise = [true; constants::MAX_WINS_X_SFBS];
        check(inverse_quantization::inverse_quantize_spectral_data(
            &ch_info.ics_info,
            &ch_info.code_book,
            &ch_info.scale_factor,
            &quantized_spectrum[..frame_length],
            spectral_data,
            &mut band_is_noise,
        ))?;

        ch_info.render_mode = AacDecoderRenderMode::EldFb;

        Ok(())
    }

    /// Reads the channel element: a `single_channel_element()` or a
    /// `channel_pair_element()` in the ER AAC ELD syntax with `epConfig` 0.
    ///
    /// # Parameters
    ///
    /// - `common_channel_data`: Common channel data.
    /// - `tp_dec`: Transport decoder data.
    /// - `spectral_data`: Spectral data of the channels of the element.
    /// - `ac_flags`: Audio codec flags.
    ///
    /// # Return
    ///
    ///   - `AacDecoderError`.
    pub fn read(
        &mut self,
        common_channel_data: &mut CommonChannelData,
        tp_dec: &mut TransportDec,
        spectral_data: &mut [f32],
        ac_flags: ACFlags,
    ) -> Result<(), AacDecoderError> {
        let frame_length = spectral_data.len() / self.num_channels as usize;

        if self.num_channels == 2 {
            // Read individual channel info, which both channels share.
            let error = self.channel_info[usize::from(AudioChannel::Left)]
                .ics_info
                .read(&mut tp_dec.bs, &common_channel_data.sr_info, ac_flags);
            if error != AacDecoderError::Ok {
                return Err(error);
            }
            self.channel_info[usize::from(AudioChannel::Right)].ics_info =
                self.channel_info[usize::from(AudioChannel::Left)].ics_info;

            if let Some(js_data) = self.js_data.as_deref_mut() {
                let max_sfb = self.channel_info[usize::from(AudioChannel::Left)]
                    .ics_info
                    .max_sf_bands();
                if js_data.read(
                    &mut tp_dec.bs,
                    &self.channel_info[usize::from(AudioChannel::Left)].ics_info,
                    max_sfb,
                ) != 0
                {
                    return Err(AacDecoderError::ParseError);
                }
            }
        }

        for (channel, spectrum) in spectral_data
            .chunks_exact_mut(frame_length)
            .take(self.num_channels as usize)
            .enumerate()
        {
            self.read_channel(common_channel_data, tp_dec, spectrum, channel, ac_flags)?;
        }

        // Set state flags (read).
        self.state_flags.insert(ElState::READ);
        Ok(())
    }

    /// Decodes the ChannelElement.
    ///
    /// # Parameters
    ///
    /// - `spectral_data`: Spectral data.
    pub fn decode(&mut self, spectral_data: &mut [f32]) {
        if self.common_window {
            if let Some(js_data) = self.js_data.as_deref_mut() {
                let (left, right) = self.channel_info.split_at(usize::from(AudioChannel::Right));
                let (left, right) = (&left[0], &right[0]);

                self.pns_inter_channel_data.init();
                if left.pns_data.is_pns_active() || right.pns_data.is_pns_active() {
                    self.pns_inter_channel_data
                        .map_midside_mask_to_pns_correlation(
                            &left.ics_info,
                            &left.pns_data,
                            &right.pns_data,
                            js_data.ms_used_mut(),
                        );
                }

                js_data.apply(&left.ics_info, spectral_data);

                intensity::apply_is(
                    &left.ics_info,
                    &right.code_book,
                    &right.scale_factor,
                    js_data.ms_used(),
                    spectral_data,
                );
            }
        } // self.common_window - ends

        // Apply PNS and TNS for each channel_info.
        let frame_length = spectral_data.len() / self.num_channels as usize;
        for (channel, (ch_info, spectrum)) in izip!(
            self.channel_info.iter_mut(),
            spectral_data.chunks_exact_mut(frame_length)
        )
        .take(self.num_channels as usize)
        .enumerate()
        {
            ch_info.pns_data.apply(
                &mut self.pns_inter_channel_data,
                &ch_info.ics_info,
                spectrum,
                &ch_info.scale_factor,
                channel,
            );
            ch_info.tns_data.apply(&ch_info.ics_info, spectrum);
        }

        self.state_flags.insert(ElState::DECODED);
    }

    /// Renders the ChannelElement data to time domain.
    ///
    /// # Parameters
    ///
    /// - `common_channel_data`: Common channel data.
    /// - `conceal_data`: Concealment data.
    /// - `spectral_data`: Spectral data.
    /// - `time_data`: Rendered time domain signal.
    /// - `is_frame_ok`: Is the frame ok or not.
    /// - `channel`: Number of the first channel of the element.
    /// - `ac_flags`: Audio codec flags.
    ///
    /// # Return
    ///
    ///   - `AacDecoderError`.
    #[expect(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        common_channel_data: &mut CommonChannelData,
        conceal_data: &mut ConcealmentData,
        spectral_data: &mut [f32],
        time_data: &mut [f32],
        is_frame_ok: bool,
        channel: usize,
        ac_flags: ACFlags,
    ) -> AacDecoderError {
        let num_element_channels = usize::from(self.num_channels);
        let frame_size = time_data.len() / num_element_channels;

        let spec_frame_size = spectral_data.len() / num_element_channels;
        let prev_spec_frame_size = self.prev_spectral_data.len() / num_element_channels;
        let prev_spectral_data = &mut self.prev_spectral_data;

        for (num_channel, (spec, prev_spec, time_out, ch_info)) in izip!(
            spectral_data.chunks_exact_mut(spec_frame_size),
            prev_spectral_data.chunks_exact_mut(prev_spec_frame_size),
            time_data.chunks_exact_mut(frame_size),
            self.channel_info.iter_mut()
        )
        .take(num_element_channels)
        .enumerate()
        {
            let num_channel = channel + num_channel;

            // Conceal defective spectral data.
            conceal_data.apply(
                &mut ch_info.ics_info,
                &common_channel_data.sr_info,
                &mut spec[..frame_size],
                &mut prev_spec[..frame_size],
                &mut ch_info.render_mode,
                num_channel,
                ac_flags,
                is_frame_ok,
            );

            match ch_info.render_mode {
                AacDecoderRenderMode::EldFb => {
                    ld_filter_bank::synthesize(
                        &mut spec[..frame_size],
                        time_out,
                        &mut ch_info.eld_overlap_buffer,
                    );
                }
                _ => {
                    return AacDecoderError::Unknown;
                }
            }

            // Time domain fading.
            conceal_data.timedomain_fading(time_out, num_channel);
        }

        AacDecoderError::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn test_buff_alignment_w_f32() {
        let ccd = CommonChannelData::new();

        let scratch_buff =
            &ccd.scratch_work_buffer[..(2 * constants::MAX_FRAMESIZE * size_of::<f32>())];

        assert!(scratch_buff.len() % size_of::<f32>() == 0);

        let addr_of_val = std::ptr::addr_of!(scratch_buff);
        assert!(addr_of_val as usize % align_of::<f32>() == 0);
    }

    #[test]
    fn test_buff_alignment_w_i16() {
        let frame_length = 1024;
        let ccd = CommonChannelData::new();

        let scratch_buff = &ccd.scratch_work_buffer[..(frame_length * size_of::<i16>())];

        assert!(scratch_buff.len() % size_of::<i16>() == 0);

        let addr_of_val = std::ptr::addr_of!(scratch_buff);
        assert!(addr_of_val as usize % align_of::<i16>() == 0);
    }
}
