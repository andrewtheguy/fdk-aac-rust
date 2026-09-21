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
//! Getting and setting of concealment parameters

use std::f32::consts::FRAC_1_SQRT_2;

use itertools::izip;

use super::conceal_constants::{self, FadeDirection};

/// Concealment Parameters
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ConcealmentParams {
    pub(super) fade_out_factor: [f32; conceal_constants::MAX_NUM_FADE_FACTORS],
    pub(super) fade_in_factor: [f32; conceal_constants::MAX_NUM_FADE_FACTORS],
    pub(super) num_fade_out_frames: i32,
    pub(super) num_fade_in_frames: i32,
    pub(super) num_mute_release_frames: i32,
    pub(super) comfort_noise_level: f32,
}

/// Default trait.
impl Default for ConcealmentParams {
    fn default() -> Self {
        let mut params = Self {
            fade_out_factor: Default::default(),
            fade_in_factor: Default::default(),
            num_fade_out_frames: conceal_constants::DFLT_FADEOUT_FRAMES,
            num_fade_in_frames: conceal_constants::DFLT_FADEIN_FRAMES,
            num_mute_release_frames: conceal_constants::DFLT_MUTE_RELEASE_FRAMES,
            comfort_noise_level: conceal_constants::DFLT_COMF_NOISE_LEVEL_FL
                / conceal_constants::MAX_COMF_NOISE_LEVEL,
        };

        // Init fade factors (symmetric).
        // Default fade factor value is FRAC_1_SQRT_2.
        let mut prev_fadeout = 1.0f32;
        izip!(
            params.fade_in_factor.iter_mut(),
            params.fade_out_factor.iter_mut(),
        )
        .for_each(|(fade_in, fade_out)| {
            prev_fadeout *= FRAC_1_SQRT_2;
            (*fade_in, *fade_out) = (prev_fadeout, prev_fadeout);
        });

        params
    }
}

// Methods of ConcealmentParams.
impl ConcealmentParams {

    /// Initializes the pre-allocated ConcealmentParams to default values.
    pub fn init(&mut self) {
        *self = Default::default();
    }

    /// Finds next fading frame in case of changing fading direction.
    ///
    /// This function determines the next fading index to be used for the fading
    /// direction to be changed.
    ///
    /// # Parameters
    ///
    /// - `act_fade_index`: Last index used for fading
    /// - `direction`: Direction of change for fading, value is type of FadeDirection.
    ///
    /// Returns index value.
    pub(super) fn find_equi_fade_frame(
        &self,
        act_fade_index: i32,
        direction: FadeDirection,
    ) -> i32 {
        let mut min_diff = 1.0_f32;
        let mut next_fade_index = 0;

        // Init depending on direction
        let (reference_val, fade_factor) = if direction == FadeDirection::OutToIn {
            // FADE-OUT => FADE-IN.
            let val = if act_fade_index < 0 {
                1.0_f32
            } else {
                self.fade_out_factor[act_fade_index as usize] / 2.0_f32
            };
            (val, self.fade_in_factor)
        } else {
            // FADE-IN => FADE-OUT.
            let val = if act_fade_index < 0 {
                1.0_f32
            } else {
                self.fade_in_factor[act_fade_index as usize] / 2.0_f32
            };
            (val, self.fade_out_factor)
        };

        // Search for minimum difference.
        for (index, value) in fade_factor
            .iter()
            .take(conceal_constants::MAX_NUM_FADE_FACTORS)
            .enumerate()
        {
            let diff = ((*value / 2.0_f32) - reference_val).abs();
            if diff < min_diff {
                min_diff = diff;
                next_fade_index = index;
            }
        }

        // Check and adjust depending on direction.
        if direction == FadeDirection::OutToIn {
            // FADE-OUT => FADE-IN.
            if ((fade_factor[next_fade_index] / 2.0_f32) <= reference_val) && (next_fade_index > 0)
            {
                next_fade_index -= 1;
            }
            if next_fade_index as i32 > (self.num_fade_in_frames - 1) {
                next_fade_index = (self.num_fade_in_frames - 1).max(0_i32) as usize;
            }
        } else {
            // FADE-IN => FADE-OUT.
            if (fade_factor[next_fade_index] / 2.0_f32) >= reference_val
                && (next_fade_index < conceal_constants::MAX_NUM_FADE_FACTORS - 1)
            {
                next_fade_index += 1;
            }
            if next_fade_index as i32 > (self.num_fade_out_frames - 1) {
                next_fade_index = (self.num_fade_out_frames - 1).max(0_i32) as usize;
            }
        }
        next_fade_index as i32
    }
}
