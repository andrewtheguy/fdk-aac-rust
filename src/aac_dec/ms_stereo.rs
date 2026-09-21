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
//! Mid/side (M/S) coding

use itertools::izip;

use super::channel_info::IcsInfo;
use crate::common::bitstream::Bitstream;

/// Maximum number of bands
const JOINTSTEREO_MAX_BANDS: usize = 64;

/// Structure holds Joint stereo data
#[repr(C)]
#[derive(Debug)]
pub struct JointStereoData {
    ms_mask_present: u8,
    ms_used: [u8; JOINTSTEREO_MAX_BANDS], // Each item contains flags for up to 8 groups.
    max_sfb: usize,
}

impl Default for JointStereoData {
    fn default() -> Self {
        JointStereoData {
            ms_mask_present: 0,
            ms_used: [0; JOINTSTEREO_MAX_BANDS],
            max_sfb: 0,
        }
    }
}

impl JointStereoData {
    /// Returns JointStereoData instance with default values.
    pub fn new() -> JointStereoData {
        JointStereoData::default()
    }

    /// Reset JointStereoData with default values.
    pub fn reset(&mut self) {
        self.ms_mask_present = 0;
        self.ms_used[..].fill(0);
        self.max_sfb = 0;
    }

    /// Returns ms_used[] data as immutable
    pub fn ms_used(&self) -> &[u8] {
        &self.ms_used
    }

    /// Returns ms_used[] data as mutable
    pub fn ms_used_mut(&mut self) -> &mut [u8] {
        &mut self.ms_used
    }

    /// Reads joint stereo data from given bitstream.
    ///
    /// # Parameters
    ///
    /// - `bs`: Bitstream.
    /// - `ics_info`: Individual channel stream info.
    /// - `max_sfb`: Number of scale factor bands.
    ///
    /// # Return
    ///
    /// - returns '0' on success, otherwise '-1'.
    pub fn read(&mut self, bs: &mut Bitstream, ics_info: &IcsInfo, max_sfb: usize) -> i32 {
        let win_groups = ics_info.n_window_groups();
        self.max_sfb = max_sfb;
        self.ms_used[..max_sfb].fill(0_u8);
        self.ms_mask_present = bs.read(2) as u8;

        match self.ms_mask_present {
            // read ms_used
            1 => {
                for group in 0..win_groups {
                    self.ms_used
                        .iter_mut()
                        .take(max_sfb)
                        .for_each(|ms| *ms |= (bs.read_bit() << group) as u8);
                }
            }

            // full spectrum M/S, set all flags to 1
            2 => {
                self.ms_used[..max_sfb].fill(255_u8);
            }

            // no M/S, and 3 is reserved
            _ => (),
        } // match end

        0 // return 0 on success
    }

    /// Generate stereo output by applying M/S stereo.
    ///
    /// # Parameters
    ///
    /// - `ics_info`: Individual channel stream info.
    /// - `spectrum`: Input spectrum data, left and right channel.
    pub fn apply(&mut self, ics_info: &IcsInfo, spectrum: &mut [f32]) {
        let sfb_offsets = ics_info.scale_factor_bands();
        let win_group_lengths = ics_info.window_group_lengths();

        let (spectrum_left, spectrum_right) = spectrum.split_at_mut(spectrum.len() / 2);
        let win_length = spectrum_right.len() / ics_info.windows_per_frame();

        let mut group_offset = 0;
        for (group, windows_per_group) in win_group_lengths
            .iter()
            .enumerate()
            .take(ics_info.n_window_groups())
        {
            let g_mask = 1 << group;
            for (left_spec, right_spec) in izip!(
                spectrum_left[group_offset * win_length..].chunks_exact_mut(win_length),
                spectrum_right[group_offset * win_length..]
                    .chunks_exact_mut(win_length)
                    .take(usize::from(*windows_per_group))
            ) {
                for (ms, band_offsets) in self
                    .ms_used
                    .iter()
                    .zip(sfb_offsets.windows(2))
                    .take(self.max_sfb)
                {
                    if *ms & g_mask != 0 {
                        let offset_currband = usize::from(band_offsets[0]);
                        let offset_nextband = usize::from(band_offsets[1]);

                        self.generate_ms_output(
                            &mut left_spec[offset_currband..],
                            &mut right_spec[offset_currband..],
                            offset_nextband - offset_currband,
                        );
                    }
                }
            } // window - iterator
            group_offset += usize::from(*windows_per_group);
        } // group - iteartor

        // Reset ms_used flags if no explicit signaling was transmitted.
        if self.ms_mask_present == 2 {
            self.ms_used[..].fill(0);
        }
    }

    /// Generates stereo output for a given band.
    ///
    /// # Parameters
    ///
    /// - `spec_left_currband`: Left channel specturm from currrent band offset.
    /// - `spec_right_currband`: Right channel spectrum from current band offset.
    /// - `n_sfb_bands`: Number of scale factor bands.
    fn generate_ms_output(
        &self,
        spec_left_currband: &mut [f32],
        spec_right_currband: &mut [f32],
        n_sfb_bands: usize,
    ) {
        for (l, r) in spec_left_currband
            .chunks_exact_mut(4)
            .zip(spec_right_currband.chunks_exact_mut(4))
            .take(n_sfb_bands >> 2)
        {
            //  t = *l
            // *l += *r
            // *r = t - *r
            let mut tmp = l[0];
            l[0] += r[0];
            r[0] = tmp - r[0];

            tmp = l[1];
            l[1] += r[1];
            r[1] = tmp - r[1];

            tmp = l[2];
            l[2] += r[2];
            r[2] = tmp - r[2];

            tmp = l[3];
            l[3] += r[3];
            r[3] = tmp - r[3];
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        aac_dec::sr_info::SamplingRateInfo,
        common::{bitstream::Mode, flags::ACFlags},
    };

    use super::*;

    /// An ELD `ics_info()`, which is `max_sfb` alone, followed by `tail`.
    fn read_ics_info(max_sfb: u32, tail: u32, tail_bits: u8) -> (IcsInfo, Bitstream) {
        let mut bitstream_writer = Bitstream::new(8, Mode::Writer);
        bitstream_writer.write(max_sfb, 6);
        bitstream_writer.write(tail, tail_bits);
        bitstream_writer.sync();

        let mut bitstream_reader = Bitstream::new(bitstream_writer.buffer().len(), Mode::Reader);
        bitstream_reader.init(bitstream_writer.buffer(), 6 + usize::from(tail_bits));

        let mut sr_info = SamplingRateInfo::new();
        let _ = sr_info.init(480, 3, 48000);

        let mut ics_info = IcsInfo::new();
        ics_info.read(&mut bitstream_reader, &sr_info, ACFlags::ER | ACFlags::ELD);
        (ics_info, bitstream_reader)
    }

    #[test]
    fn new_jointstereo_data() {
        let jsd = JointStereoData::new();
        assert_eq!(jsd.ms_mask_present, 0);
        assert_eq!(jsd.ms_used, [0_u8; JOINTSTEREO_MAX_BANDS]);
        assert_eq!(jsd.max_sfb, 0);
    }

    #[test]
    fn generate_ms_output_samples() {
        let jsd = JointStereoData::new();
        let mut spec_left = [
            1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
            16.0,
        ];
        let mut spec_right = [
            2.0_f32, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0,
        ];
        let spec_left_ref = [
            3.0_f32, 5.0, 7.0, 9.0, 11.0, 13.0, 15.0, 17.0, 19.0, 21.0, 23.0, 25.0, 27.0, 29.0,
            31.0, 33.0,
        ];
        let spec_right_ref = [-1.0_f32; 16];
        let n_max_sfb = 16;
        jsd.generate_ms_output(&mut spec_left, &mut spec_right, n_max_sfb);

        assert_eq!(spec_left_ref[..], spec_left[..]);
        assert_eq!(spec_right_ref[..], spec_right[..]);
    }

    #[test]
    fn read_ms_mask_present_1() {
        // 0b01       - ms_mask_present
        // 0b01010101 - ms_used flags
        let (ics_info, mut bitstream_reader) = read_ics_info(8, 0b0101010101, 10);
        let max_sfb = ics_info.max_sf_bands();

        let mut jsd = JointStereoData::new();
        let ret = jsd.read(&mut bitstream_reader, &ics_info, max_sfb);

        assert_eq!(ret, 0);
        assert_eq!(jsd.max_sfb, 8);
        assert_eq!(jsd.ms_mask_present, 1);
        assert_eq!(jsd.ms_used[0..max_sfb], [0, 1, 0, 1, 0, 1, 0, 1]);
    }

    #[test]
    fn read_ms_mask_present_2() {
        let (ics_info, mut bitstream_reader) = read_ics_info(30, 0b10, 2);
        let max_sfb = ics_info.max_sf_bands();

        let mut jsd = JointStereoData::new();
        let ret = jsd.read(&mut bitstream_reader, &ics_info, max_sfb);

        assert_eq!(ret, 0);
        assert_eq!(jsd.max_sfb, 30);
        assert_eq!(jsd.ms_mask_present, 2);
        assert_eq!(jsd.ms_used[0..max_sfb], [255; 30]);
    }

    #[test]
    fn apply_ms_stereo() {
        let (ics_info, mut bitstream_reader) = read_ics_info(2, 0b0110, 4);

        let mut jsd = JointStereoData::new();
        assert_eq!(jsd.read(&mut bitstream_reader, &ics_info, 2), 0);
        assert_eq!(jsd.ms_used[0..2], [1, 0]);

        const FRAME_LENGTH: usize = 480;
        let mut spectrum = vec![0.0_f32; FRAME_LENGTH * 2];
        spectrum[..FRAME_LENGTH].fill(0.05_f32); // Left channel
        spectrum[FRAME_LENGTH..].fill(0.025_f32); // Right channel

        jsd.apply(&ics_info, &mut spectrum);

        // Only the first scale factor band is M/S coded.
        let band = usize::from(ics_info.scale_factor_bands()[1]);
        let (left, right) = spectrum.split_at(FRAME_LENGTH);
        assert!(left[..band].iter().all(|&s| s == 0.05_f32 + 0.025_f32));
        assert!(right[..band].iter().all(|&s| s == 0.05_f32 - 0.025_f32));
        assert!(left[band..].iter().all(|&s| s == 0.05_f32));
        assert!(right[band..].iter().all(|&s| s == 0.025_f32));
    }
}
