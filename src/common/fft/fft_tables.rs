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
//! FFT tables

use num_complex::Complex;

#[rustfmt::skip]
pub static FFT16_W16: [Complex<f32>; 2] = [
    Complex {re: 0.923879533, im: 0.382683432},
    Complex {re: 0.382683432, im: 0.923879533},
];

#[rustfmt::skip]
pub static ROT_VECTOR_240: [Complex<f32>; 210] = [
    Complex { re: 0.999657325, im: 0.026176948 },     Complex { re: 0.998629535, im: 0.052335956 },     Complex { re: 0.996917334, im: 0.078459096 },     Complex { re: 0.994521895, im: 0.104528463 },     Complex { re: 0.991444861, im: 0.130526192 },     Complex { re: 0.987688341, im: 0.156434465 },     Complex { re: 0.983254908, im: 0.182235525 },     Complex { re: 0.978147601, im: 0.207911691 },     Complex { re: 0.972369920, im: 0.233445364 },     Complex { re: 0.965925826, im: 0.258819045 },     Complex { re: 0.958819735, im: 0.284015345 },     Complex { re: 0.951056516, im: 0.309016994 },     Complex { re: 0.942641491, im: 0.333806859 },     Complex { re: 0.933580426, im: 0.358367950 },     Complex { re: 0.923879533, im: 0.382683432 },
    Complex { re: 0.998629535, im: 0.052335956 },     Complex { re: 0.994521895, im: 0.104528463 },     Complex { re: 0.987688341, im: 0.156434465 },     Complex { re: 0.978147601, im: 0.207911691 },     Complex { re: 0.965925826, im: 0.258819045 },     Complex { re: 0.951056516, im: 0.309016994 },     Complex { re: 0.933580426, im: 0.358367950 },     Complex { re: 0.913545458, im: 0.406736643 },     Complex { re: 0.891006524, im: 0.453990500 },     Complex { re: 0.866025404, im: 0.500000000 },     Complex { re: 0.838670568, im: 0.544639035 },     Complex { re: 0.809016994, im: 0.587785252 },     Complex { re: 0.777145961, im: 0.629320391 },     Complex { re: 0.743144825, im: 0.669130606 },     Complex { re: 0.707106781, im: 0.707106781 },
    Complex { re: 0.996917334, im: 0.078459096 },     Complex { re: 0.987688341, im: 0.156434465 },     Complex { re: 0.972369920, im: 0.233445364 },     Complex { re: 0.951056516, im: 0.309016994 },     Complex { re: 0.923879533, im: 0.382683432 },     Complex { re: 0.891006524, im: 0.453990500 },     Complex { re: 0.852640164, im: 0.522498565 },     Complex { re: 0.809016994, im: 0.587785252 },     Complex { re: 0.760405966, im: 0.649448048 },     Complex { re: 0.707106781, im: 0.707106781 },     Complex { re: 0.649448048, im: 0.760405966 },     Complex { re: 0.587785252, im: 0.809016994 },     Complex { re: 0.522498565, im: 0.852640164 },     Complex { re: 0.453990500, im: 0.891006524 },     Complex { re: 0.382683432, im: 0.923879533 },
    Complex { re: 0.994521895, im: 0.104528463 },     Complex { re: 0.978147601, im: 0.207911691 },     Complex { re: 0.951056516, im: 0.309016994 },     Complex { re: 0.913545458, im: 0.406736643 },     Complex { re: 0.866025404, im: 0.500000000 },     Complex { re: 0.809016994, im: 0.587785252 },     Complex { re: 0.743144825, im: 0.669130606 },     Complex { re: 0.669130606, im: 0.743144825 },     Complex { re: 0.587785252, im: 0.809016994 },     Complex { re: 0.500000000, im: 0.866025404 },     Complex { re: 0.406736643, im: 0.913545458 },     Complex { re: 0.309016994, im: 0.951056516 },     Complex { re: 0.207911691, im: 0.978147601 },     Complex { re: 0.104528463, im: 0.994521895 },     Complex { re: 0.000000000, im: 1.000000000 },
    Complex { re: 0.991444861, im: 0.130526192 },     Complex { re: 0.965925826, im: 0.258819045 },     Complex { re: 0.923879533, im: 0.382683432 },     Complex { re: 0.866025404, im: 0.500000000 },     Complex { re: 0.793353340, im: 0.608761429 },     Complex { re: 0.707106781, im: 0.707106781 },     Complex { re: 0.608761429, im: 0.793353340 },     Complex { re: 0.500000000, im: 0.866025404 },     Complex { re: 0.382683432, im: 0.923879533 },     Complex { re: 0.258819045, im: 0.965925826 },     Complex { re: 0.130526192, im: 0.991444861 },     Complex { re: 0.000000000, im: 1.000000000 },     Complex { re: -0.130526192, im: 0.991444861 },     Complex { re: -0.258819045, im: 0.965925826 },     Complex { re: -0.382683432, im: 0.923879533 },
    Complex { re: 0.987688341, im: 0.156434465 },     Complex { re: 0.951056516, im: 0.309016994 },     Complex { re: 0.891006524, im: 0.453990500 },     Complex { re: 0.809016994, im: 0.587785252 },     Complex { re: 0.707106781, im: 0.707106781 },     Complex { re: 0.587785252, im: 0.809016994 },     Complex { re: 0.453990500, im: 0.891006524 },     Complex { re: 0.309016994, im: 0.951056516 },     Complex { re: 0.156434465, im: 0.987688341 },     Complex { re: 0.000000000, im: 1.000000000 },     Complex { re: -0.156434465, im: 0.987688341 },     Complex { re: -0.309016994, im: 0.951056516 },     Complex { re: -0.453990500, im: 0.891006524 },     Complex { re: -0.587785252, im: 0.809016994 },     Complex { re: -0.707106781, im: 0.707106781 },
    Complex { re: 0.983254908, im: 0.182235525 },     Complex { re: 0.933580426, im: 0.358367950 },     Complex { re: 0.852640164, im: 0.522498565 },     Complex { re: 0.743144825, im: 0.669130606 },     Complex { re: 0.608761429, im: 0.793353340 },     Complex { re: 0.453990500, im: 0.891006524 },     Complex { re: 0.284015345, im: 0.958819735 },     Complex { re: 0.104528463, im: 0.994521895 },     Complex { re: -0.078459096, im: 0.996917334 },     Complex { re: -0.258819045, im: 0.965925826 },     Complex { re: -0.430511097, im: 0.902585284 },     Complex { re: -0.587785252, im: 0.809016994 },     Complex { re: -0.725374371, im: 0.688354576 },     Complex { re: -0.838670568, im: 0.544639035 },     Complex { re: -0.923879533, im: 0.382683432 },
    Complex { re: 0.978147601, im: 0.207911691 },     Complex { re: 0.913545458, im: 0.406736643 },     Complex { re: 0.809016994, im: 0.587785252 },     Complex { re: 0.669130606, im: 0.743144825 },     Complex { re: 0.500000000, im: 0.866025404 },     Complex { re: 0.309016994, im: 0.951056516 },     Complex { re: 0.104528463, im: 0.994521895 },     Complex { re: -0.104528463, im: 0.994521895 },     Complex { re: -0.309016994, im: 0.951056516 },     Complex { re: -0.500000000, im: 0.866025404 },     Complex { re: -0.669130606, im: 0.743144825 },     Complex { re: -0.809016994, im: 0.587785252 },     Complex { re: -0.913545458, im: 0.406736643 },     Complex { re: -0.978147601, im: 0.207911691 },     Complex { re: -1.000000000, im: 0.000000000 },
    Complex { re: 0.972369920, im: 0.233445364 },     Complex { re: 0.891006524, im: 0.453990500 },     Complex { re: 0.760405966, im: 0.649448048 },     Complex { re: 0.587785252, im: 0.809016994 },     Complex { re: 0.382683432, im: 0.923879533 },     Complex { re: 0.156434465, im: 0.987688341 },     Complex { re: -0.078459096, im: 0.996917334 },     Complex { re: -0.309016994, im: 0.951056516 },     Complex { re: -0.522498565, im: 0.852640164 },     Complex { re: -0.707106781, im: 0.707106781 },     Complex { re: -0.852640164, im: 0.522498565 },     Complex { re: -0.951056516, im: 0.309016994 },     Complex { re: -0.996917334, im: 0.078459096 },     Complex { re: -0.987688341, im: -0.156434465 },     Complex { re: -0.923879533, im: -0.382683432 },
    Complex { re: 0.965925826, im: 0.258819045 },     Complex { re: 0.866025404, im: 0.500000000 },     Complex { re: 0.707106781, im: 0.707106781 },     Complex { re: 0.500000000, im: 0.866025404 },     Complex { re: 0.258819045, im: 0.965925826 },     Complex { re: 0.000000000, im: 1.000000000 },     Complex { re: -0.258819045, im: 0.965925826 },     Complex { re: -0.500000000, im: 0.866025404 },     Complex { re: -0.707106781, im: 0.707106781 },     Complex { re: -0.866025404, im: 0.500000000 },     Complex { re: -0.965925826, im: 0.258819045 },     Complex { re: -1.000000000, im: 0.000000000 },     Complex { re: -0.965925826, im: -0.258819045 },     Complex { re: -0.866025404, im: -0.500000000 },     Complex { re: -0.707106781, im: -0.707106781 },
    Complex { re: 0.958819735, im: 0.284015345 },     Complex { re: 0.838670568, im: 0.544639035 },     Complex { re: 0.649448048, im: 0.760405966 },     Complex { re: 0.406736643, im: 0.913545458 },     Complex { re: 0.130526192, im: 0.991444861 },     Complex { re: -0.156434465, im: 0.987688341 },     Complex { re: -0.430511097, im: 0.902585284 },     Complex { re: -0.669130606, im: 0.743144825 },     Complex { re: -0.852640164, im: 0.522498565 },     Complex { re: -0.965925826, im: 0.258819045 },     Complex { re: -0.999657325, im: -0.026176948 },     Complex { re: -0.951056516, im: -0.309016994 },     Complex { re: -0.824126189, im: -0.566406237 },     Complex { re: -0.629320391, im: -0.777145961 },     Complex { re: -0.382683432, im: -0.923879533 },
    Complex { re: 0.951056516, im: 0.309016994 },     Complex { re: 0.809016994, im: 0.587785252 },     Complex { re: 0.587785252, im: 0.809016994 },     Complex { re: 0.309016994, im: 0.951056516 },     Complex { re: 0.000000000, im: 1.000000000 },     Complex { re: -0.309016994, im: 0.951056516 },     Complex { re: -0.587785252, im: 0.809016994 },     Complex { re: -0.809016994, im: 0.587785252 },     Complex { re: -0.951056516, im: 0.309016994 },     Complex { re: -1.000000000, im: 0.000000000 },     Complex { re: -0.951056516, im: -0.309016994 },     Complex { re: -0.809016994, im: -0.587785252 },     Complex { re: -0.587785252, im: -0.809016994 },     Complex { re: -0.309016994, im: -0.951056516 },     Complex { re: -0.000000000, im: -1.000000000 },
    Complex { re: 0.942641491, im: 0.333806859 },     Complex { re: 0.777145961, im: 0.629320391 },     Complex { re: 0.522498565, im: 0.852640164 },     Complex { re: 0.207911691, im: 0.978147601 },     Complex { re: -0.130526192, im: 0.991444861 },     Complex { re: -0.453990500, im: 0.891006524 },     Complex { re: -0.725374371, im: 0.688354576 },     Complex { re: -0.913545458, im: 0.406736643 },     Complex { re: -0.996917334, im: 0.078459096 },     Complex { re: -0.965925826, im: -0.258819045 },     Complex { re: -0.824126189, im: -0.566406237 },     Complex { re: -0.587785252, im: -0.809016994 },     Complex { re: -0.284015345, im: -0.958819735 },     Complex { re: 0.052335956, im: -0.998629535 },     Complex { re: 0.382683432, im: -0.923879533 },
    Complex { re: 0.933580426, im: 0.358367950 },     Complex { re: 0.743144825, im: 0.669130606 },     Complex { re: 0.453990500, im: 0.891006524 },     Complex { re: 0.104528463, im: 0.994521895 },     Complex { re: -0.258819045, im: 0.965925826 },     Complex { re: -0.587785252, im: 0.809016994 },     Complex { re: -0.838670568, im: 0.544639035 },     Complex { re: -0.978147601, im: 0.207911691 },     Complex { re: -0.987688341, im: -0.156434465 },     Complex { re: -0.866025404, im: -0.500000000 },     Complex { re: -0.629320391, im: -0.777145961 },     Complex { re: -0.309016994, im: -0.951056516 },     Complex { re: 0.052335956, im: -0.998629535 },     Complex { re: 0.406736643, im: -0.913545458 },     Complex { re: 0.707106781, im: -0.707106781 },
];

