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
//! AAC core concealment interface

use super::conceal_constants::AacDecoderRenderMode;
use super::conceal_info::ConcealmentInfo;
use super::conceal_params::ConcealmentParams;
use crate::aac_dec::channel_info::IcsInfo;
use crate::common::flags::ACFlags;

/// Concealment data
#[repr(C)]
#[derive(Default, Debug)]
pub struct ConcealmentData {
    params: ConcealmentParams,
    conceal_info: Vec<ConcealmentInfo>,
    td_noise_seed: u32,
}

impl ConcealmentData {

    /// Initializes concealment data with default values
    /// for a given number of channels.
    ///
    /// # Parameters
    ///
    /// - `num_channels`: Number of channels to be initialized in concealment data.
    pub fn init(&mut self, num_channels: usize) {
        self.params.init();

        self.conceal_info = vec![ConcealmentInfo::default(); num_channels];
        for ci in self.conceal_info.iter_mut().take(num_channels) {
            ci.init_channel_data();
        }
        self.td_noise_seed = 0;
    }

    /// This function applies different concealment techniques (Muting, Noise substitution,
    /// Interpolation) on the specific channel spectral data, based on current and previous
    /// frame spectral coefficients.
    ///
    /// # Parameters
    ///
    /// - `ics_info`: Individual channel stream info with valid internal data
    /// - `spectral_coefficient`: Spectral coefficients of current frame
    /// - `spectral_coefficient_prev`: Spectral coefficients of previous frame
    /// - `render_mode`: Type of AAC decoder's render mode (AacdecRenderMode)
    /// - `channel`: Channel to be concealed
    /// - `ac_flags`: Scalefactors for each band in each window
    /// - `is_frame_ok`: Flag indicates whether current frame is Ok (or) defective
    #[expect(clippy::too_many_arguments)]
    pub fn apply(
        &mut self,
        ics_info: &mut IcsInfo,
        spectral_coefficient: &mut [f32],
        spectral_coefficient_prev: &mut [f32],
        render_mode: &mut AacDecoderRenderMode,
        channel: usize,
        ac_flags: ACFlags,
        is_frame_ok: bool,
    ) {
        self.conceal_info[channel].apply(
            &self.params,
            ics_info,
            spectral_coefficient,
            spectral_coefficient_prev,
            render_mode,
            ac_flags,
            is_frame_ok,
        );
    }

    /// Returns status of last frame in concealment.
    pub fn was_last_frame_ok(&self) -> bool {
        for ci in self.conceal_info.iter() {
            if !ci.is_prev_frame_ok[1] {
                return false; // Last frame was not okay.
            }
        }
        true // Last frame was okay.
    }

    /// This function does time domain fading (TDFading) in concealment case.
    ///
    /// In case of concealment, this function takes care of the fading, after time
    /// domain signal has been rendered by the respective signal rendering functions.
    ///   The fading out in case of ACELP decoding is not done by this function but by
    /// the ACELP decoder for the first concealed frame.
    ///
    /// TimeDomain fading never creates jumps in energy / discontinuities, it always
    /// does a continuous fading. To achieve this, fading is always done from a starting
    /// point to a target point, while the starting point is always determined to be the
    /// last target point. By varying the target point of a fading, the fading slope can
    /// be controlled.
    ///
    /// This principle is applied to the fading within a frame and the fading from
    /// frame to frame.
    ///
    /// One frame is divided into 8 subframes to obtain 8 parts of fading slopes
    /// within a frame, each maybe with its own gradient.
    ///
    /// # Parameters
    ///
    /// - `pcm_data`: PCM data to be faded
    /// - `channel`: Index of the channel
    pub fn timedomain_fading(&mut self, pcm_data: &mut [f32], channel: usize) {
        // noise seed update
        ConcealmentInfo::timedomain_noise_random(&mut self.td_noise_seed);
        self.td_noise_seed += 1;
        self.conceal_info[channel].timedomain_fading(&self.params, self.td_noise_seed, pcm_data);
    }

    /// De-allocates heap allocated memory.
    pub fn deinit_conceal_info(&mut self) {
        if self.conceal_info.capacity() != 0 {
            self.conceal_info.clear();
            // Deallocate memory by shrinking capacity to zero.
            self.conceal_info.shrink_to_fit();
        }
    }
}
