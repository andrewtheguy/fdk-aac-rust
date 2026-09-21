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
//! DCT IV
//!
//! The transform is calculated by the complex FFT of this crate, with some pre- and
//! post-twiddling. Supported lengths are 480 and 512.

use crate::common::fft::{fft, MAX_FFT_LENGTH};
use crate::common::tables::sine_tables;
use crate::common::tables::window_tables;
use itertools::izip;
use num_complex::Complex;
use std::f32::consts;

/// Calculate DCT type IV of `data.len()` length. The DCT IV is
/// calculated by a complex FFT, with some pre- and post twiddling.
/// A factor of `sqrt(2/p_in.len())` is NOT applied.
///
/// # Parameters
///
/// data input/output data (in place processing).
pub fn dctiv(data: &mut [f32]) {
    let dct_len = data.len();
    let fft_len = dct_len / 2;

    // get tables
    let twiddle =
        window_tables::get_table(dct_len as u16).unwrap();

    // pre-twiddling, straight into the buffer the FFT works on
    let mut spectrum = [Complex::<f32>::default(); MAX_FFT_LENGTH];
    let spectrum = &mut spectrum[..fft_len];
    {
        let (im_part, re_part) = data.split_at(fft_len);
        let (lower, upper) = spectrum.split_at_mut(fft_len / 2);

        let im_chunk = im_part.chunks_exact(2);
        let re_chunk = re_part.rchunks_exact(2);
        let twi_chunk = twiddle.chunks_exact(2);

        for (real, imag, twi, lo, hi) in
            izip!(re_chunk, im_chunk, twi_chunk, lower.iter_mut(), upper.iter_mut().rev())
        {
            let accu1 = Complex {
                re: real[1],
                im: imag[0],
            } * twi[0];
            let accu2 = Complex {
                re: real[0],
                im: imag[1],
            } * twi[1];

            *lo = Complex {
                re: accu1.im,
                im: accu1.re,
            };
            *hi = Complex {
                re: accu2.im,
                im: -accu2.re,
            };
        }
    }

    // fft
    fft(spectrum);

    // post-twiddling, back into the caller's buffer
    let mut sin_step = 0;
    let sin_twiddle = sine_tables::get_table(dct_len as u16, &mut sin_step).unwrap();

    let mut sin_twi = sin_twiddle[sin_step];
    let mut accu1 = spectrum[fft_len - 1];
    let mut accu2 = accu1 * sin_twi;

    data[0] = spectrum[0].re;
    data[1] = accu2.re;
    data[dct_len - 2] = accu2.im;
    data[dct_len - 1] = -spectrum[0].im;

    {
        let (left, right) = data.split_at_mut(fft_len);
        let (lower, upper) = spectrum.split_at(fft_len / 2);
        let le_chunk = left.chunks_exact_mut(2);
        let ri_chunk = right.rchunks_exact_mut(2);

        for (k, (l_data, r_data, lo, hi)) in
            izip!(le_chunk, ri_chunk, lower, upper.iter().rev()).enumerate().skip(1)
        {
            accu2 = Complex {
                re: lo.im,
                im: lo.re,
            } * sin_twi;

            accu1 = *hi;

            r_data[1] = -accu2.re;
            l_data[0] = accu2.im;

            sin_twi = sin_twiddle[(k + 1) * sin_step];

            accu2 = accu1 * sin_twi;
            l_data[1] = accu2.re;
            r_data[0] = accu2.im;
        }
    }

    accu1 *= consts::FRAC_1_SQRT_2;

    data[fft_len] = accu1.re + accu1.im;
    data[fft_len - 1] = accu1.re - accu1.im;
}

#[cfg(test)]
mod tests {
    use crate::common::dct;
    use std::f32::consts::PI;

    pub fn generate_sine(data: &mut [f32], f: f32, fs: f32) {
        let t = 1.0 / fs;
        for (i, dst) in data.iter_mut().enumerate() {
            *dst = (2.0 * PI * f * t * (i as f32)).sin();
        }
        // println!("{:.5?}", x);
    }

    pub fn compare_signals(p_data: &[f32], p_reference: &[f32], threshold: f32) -> i32 {
        let mut max_diff: f32 = 0.0;
        let mut max_value: f32 = 0.0;

        if p_data.len() == p_reference.len() {
            for n in 0..p_reference.len() {
                max_diff = max_diff.max((p_reference[n] - p_data[n]).abs());
                max_value = max_value.max(p_reference[n].abs());
            }
            let bits = (max_diff / max_value).log2().abs();

            if bits > threshold {
                println!("accuracy in bits; {bits} passed");
                0 // all fine
            } else {
                println!("accuracy in bits: {bits} - failed, below threshold {threshold}.");
                1 // accuracy too low
            }
        } else {
            1 // data length differs from reference
        }
    }

    pub fn dct4_ref(data: &[f32], p_out: &mut [f32]) {
        let dctiv_len = data.len();
        let len_flt = dctiv_len as f64;

        for k in (0..dctiv_len).step_by(1) {
            let mut mysum = 0.0;
            for n in (0..dctiv_len).step_by(1) {
                let k_flt = k as f64;
                let n_flt = n as f64;
                let val = data[n] as f64;
                mysum += val * ((PI as f64) / len_flt * (n_flt + 0.5) * (k_flt + 0.5)).cos();
            }
            p_out[k] = mysum as f32;
        }
    }

    #[derive(Debug)]
    enum TransformType {
        _DCTI,
        _DSTI,
        _DctII,
        _DSTII,
        _DCTIII,
        _DSTIII,
        DctIV,
        _DstIV,
    }

    const TTTT: [TransformType; 1] = [TransformType::DctIV]; // Transform Types To Test

    const TSTT: [usize; 2] = [480, 512]; // Transform Sizes To Test

    #[test]
    fn cmp_to_ref() {
        for tt in TTTT {
            for ts in TSTT {
                let mut p_data = vec![0f32; ts];

                generate_sine(&mut p_data, 1500.0, 48000.0);
                let p_data_ref = p_data.clone();
                let mut p_reference = vec![0f32; ts];

                println!("! TEST {tt:?}-{ts} !");

                match tt {
                    TransformType::DctIV => {
                        dct::dctiv(&mut p_data);
                        dct4_ref(&p_data_ref, &mut p_reference);
                    }
                    _ => {
                        panic!("Not implemented")
                    }
                }

                assert_eq!(
                    compare_signals(&p_data, &p_reference, 16.0),
                    0,
                    "Compare_signals call failed."
                );
            }
        }

        println!("! TEST DONE !");
    }
}
