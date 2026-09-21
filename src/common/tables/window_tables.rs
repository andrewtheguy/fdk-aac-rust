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
//! Sine window tables

use num_complex::Complex;

/// Returns the sine window slope of the given length: that of the 480 or of the 512 sample frame.
pub fn get_table(length: u16) -> Option<&'static [Complex<f32>]> {
    match length {
        480 => Some(&SINE_WINDOW_480),
        512 => Some(&SINE_WINDOW_512),
        _ => None,
    }
}

#[rustfmt::skip]
pub static SINE_WINDOW_480: [Complex<f32>; 240] = [

  Complex { re: 0.999998661, im: 0.001636245 }, Complex { re: 0.999987952, im: 0.004908719 }, Complex { re: 0.999966534, im: 0.008181140 }, Complex { re: 0.999934407, im: 0.011453473 },
  Complex { re: 0.999891571, im: 0.014725683 }, Complex { re: 0.999838028, im: 0.017997736 }, Complex { re: 0.999773777, im: 0.021269596 }, Complex { re: 0.999698819, im: 0.024541229 },
  Complex { re: 0.999613155, im: 0.027812598 }, Complex { re: 0.999516786, im: 0.031083670 }, Complex { re: 0.999409713, im: 0.034354408 }, Complex { re: 0.999291937, im: 0.037624779 },
  Complex { re: 0.999163460, im: 0.040894747 }, Complex { re: 0.999024282, im: 0.044164277 }, Complex { re: 0.998874406, im: 0.047433334 }, Complex { re: 0.998713832, im: 0.050701883 },
  Complex { re: 0.998542563, im: 0.053969889 }, Complex { re: 0.998360601, im: 0.057237317 }, Complex { re: 0.998167947, im: 0.060504132 }, Complex { re: 0.997964603, im: 0.063770300 },
  Complex { re: 0.997750572, im: 0.067035784 }, Complex { re: 0.997525856, im: 0.070300550 }, Complex { re: 0.997290457, im: 0.073564564 }, Complex { re: 0.997044378, im: 0.076827789 },
  Complex { re: 0.996787621, im: 0.080090192 }, Complex { re: 0.996520189, im: 0.083351737 }, Complex { re: 0.996242086, im: 0.086612390 }, Complex { re: 0.995953314, im: 0.089872115 },
  Complex { re: 0.995653875, im: 0.093130877 }, Complex { re: 0.995343775, im: 0.096388643 }, Complex { re: 0.995023014, im: 0.099645376 }, Complex { re: 0.994691598, im: 0.102901041 },
  Complex { re: 0.994349530, im: 0.106155605 }, Complex { re: 0.993996813, im: 0.109409032 }, Complex { re: 0.993633451, im: 0.112661288 }, Complex { re: 0.993259448, im: 0.115912336 },
  Complex { re: 0.992874808, im: 0.119162144 }, Complex { re: 0.992479535, im: 0.122410675 }, Complex { re: 0.992073633, im: 0.125657896 }, Complex { re: 0.991657107, im: 0.128903770 },
  Complex { re: 0.991229961, im: 0.132148265 }, Complex { re: 0.990792200, im: 0.135391344 }, Complex { re: 0.990343829, im: 0.138632973 }, Complex { re: 0.989884851, im: 0.141873117 },
  Complex { re: 0.989415273, im: 0.145111742 }, Complex { re: 0.988935099, im: 0.148348814 }, Complex { re: 0.988444334, im: 0.151584296 }, Complex { re: 0.987942984, im: 0.154818155 },
  Complex { re: 0.987431053, im: 0.158050356 }, Complex { re: 0.986908548, im: 0.161280865 }, Complex { re: 0.986375474, im: 0.164509646 }, Complex { re: 0.985831837, im: 0.167736666 },
  Complex { re: 0.985277642, im: 0.170961889 }, Complex { re: 0.984712896, im: 0.174185281 }, Complex { re: 0.984137604, im: 0.177406808 }, Complex { re: 0.983551773, im: 0.180626435 },
  Complex { re: 0.982955409, im: 0.183844128 }, Complex { re: 0.982348519, im: 0.187059852 }, Complex { re: 0.981731108, im: 0.190273572 }, Complex { re: 0.981103183, im: 0.193485255 },
  Complex { re: 0.980464752, im: 0.196694866 }, Complex { re: 0.979815821, im: 0.199902371 }, Complex { re: 0.979156396, im: 0.203107734 }, Complex { re: 0.978486486, im: 0.206310923 },
  Complex { re: 0.977806097, im: 0.209511902 }, Complex { re: 0.977115236, im: 0.212710637 }, Complex { re: 0.976413911, im: 0.215907095 }, Complex { re: 0.975702130, im: 0.219101240 },
  Complex { re: 0.974979900, im: 0.222293039 }, Complex { re: 0.974247228, im: 0.225482457 }, Complex { re: 0.973504123, im: 0.228669461 }, Complex { re: 0.972750593, im: 0.231854016 },
  Complex { re: 0.971986645, im: 0.235036087 }, Complex { re: 0.971212288, im: 0.238215642 }, Complex { re: 0.970427530, im: 0.241392645 }, Complex { re: 0.969632379, im: 0.244567064 },
  Complex { re: 0.968826845, im: 0.247738863 }, Complex { re: 0.968010935, im: 0.250908009 }, Complex { re: 0.967184659, im: 0.254074469 }, Complex { re: 0.966348025, im: 0.257238207 },
  Complex { re: 0.965501042, im: 0.260399190 }, Complex { re: 0.964643719, im: 0.263557385 }, Complex { re: 0.963776066, im: 0.266712757 }, Complex { re: 0.962898091, im: 0.269865274 },
  Complex { re: 0.962009805, im: 0.273014899 }, Complex { re: 0.961111216, im: 0.276161602 }, Complex { re: 0.960202335, im: 0.279305346 }, Complex { re: 0.959283170, im: 0.282446100 },
  Complex { re: 0.958353733, im: 0.285583829 }, Complex { re: 0.957414032, im: 0.288718499 }, Complex { re: 0.956464078, im: 0.291850078 }, Complex { re: 0.955503881, im: 0.294978531 },
  Complex { re: 0.954533451, im: 0.298103825 }, Complex { re: 0.953552799, im: 0.301225927 }, Complex { re: 0.952561936, im: 0.304344802 }, Complex { re: 0.951560871, im: 0.307460419 },
  Complex { re: 0.950549616, im: 0.310572743 }, Complex { re: 0.949528181, im: 0.313681740 }, Complex { re: 0.948496577, im: 0.316787379 }, Complex { re: 0.947454816, im: 0.319889625 },
  Complex { re: 0.946402908, im: 0.322988445 }, Complex { re: 0.945340865, im: 0.326083806 }, Complex { re: 0.944268698, im: 0.329175676 }, Complex { re: 0.943186419, im: 0.332264020 },
  Complex { re: 0.942094039, im: 0.335348805 }, Complex { re: 0.940991570, im: 0.338430000 }, Complex { re: 0.939879024, im: 0.341507570 }, Complex { re: 0.938756412, im: 0.344581482 },
  Complex { re: 0.937623748, im: 0.347651705 }, Complex { re: 0.936481041, im: 0.350718205 }, Complex { re: 0.935328306, im: 0.353780948 }, Complex { re: 0.934165555, im: 0.356839903 },
  Complex { re: 0.932992799, im: 0.359895037 }, Complex { re: 0.931810051, im: 0.362946316 }, Complex { re: 0.930617325, im: 0.365993708 }, Complex { re: 0.929414632, im: 0.369037181 },
  Complex { re: 0.928201987, im: 0.372076702 }, Complex { re: 0.926979400, im: 0.375112238 }, Complex { re: 0.925746887, im: 0.378143757 }, Complex { re: 0.924504460, im: 0.381171226 },
  Complex { re: 0.923252132, im: 0.384194614 }, Complex { re: 0.921989916, im: 0.387213887 }, Complex { re: 0.920717827, im: 0.390229013 }, Complex { re: 0.919435878, im: 0.393239960 },
  Complex { re: 0.918144082, im: 0.396246696 }, Complex { re: 0.916842454, im: 0.399249188 }, Complex { re: 0.915531007, im: 0.402247405 }, Complex { re: 0.914209756, im: 0.405241314 },
  Complex { re: 0.912878714, im: 0.408230883 }, Complex { re: 0.911537896, im: 0.411216081 }, Complex { re: 0.910187316, im: 0.414196874 }, Complex { re: 0.908826988, im: 0.417173232 },
  Complex { re: 0.907456928, im: 0.420145122 }, Complex { re: 0.906077150, im: 0.423112513 }, Complex { re: 0.904687668, im: 0.426075373 }, Complex { re: 0.903288498, im: 0.429033669 },
  Complex { re: 0.901879654, im: 0.431987372 }, Complex { re: 0.900461152, im: 0.434936447 }, Complex { re: 0.899033007, im: 0.437880866 }, Complex { re: 0.897595234, im: 0.440820594 },
  Complex { re: 0.896147848, im: 0.443755602 }, Complex { re: 0.894690865, im: 0.446685858 }, Complex { re: 0.893224301, im: 0.449611330 }, Complex { re: 0.891748171, im: 0.452531987 },
  Complex { re: 0.890262492, im: 0.455447797 }, Complex { re: 0.888767278, im: 0.458358731 }, Complex { re: 0.887262546, im: 0.461264755 }, Complex { re: 0.885748312, im: 0.464165840 },
  Complex { re: 0.884224593, im: 0.467061954 }, Complex { re: 0.882691405, im: 0.469953066 }, Complex { re: 0.881148763, im: 0.472839145 }, Complex { re: 0.879596685, im: 0.475720161 },
  Complex { re: 0.878035187, im: 0.478596082 }, Complex { re: 0.876464287, im: 0.481466878 }, Complex { re: 0.874884000, im: 0.484332517 }, Complex { re: 0.873294343, im: 0.487192970 },
  Complex { re: 0.871695335, im: 0.490048205 }, Complex { re: 0.870086991, im: 0.492898192 }, Complex { re: 0.868469329, im: 0.495742901 }, Complex { re: 0.866842367, im: 0.498582301 },
  Complex { re: 0.865206122, im: 0.501416361 }, Complex { re: 0.863560611, im: 0.504245051 }, Complex { re: 0.861905852, im: 0.507068342 }, Complex { re: 0.860241862, im: 0.509886202 },
  Complex { re: 0.858568660, im: 0.512698601 }, Complex { re: 0.856886264, im: 0.515505511 }, Complex { re: 0.855194690, im: 0.518306899 }, Complex { re: 0.853493959, im: 0.521102737 },
  Complex { re: 0.851784087, im: 0.523892994 }, Complex { re: 0.850065093, im: 0.526677641 }, Complex { re: 0.848336996, im: 0.529456647 }, Complex { re: 0.846599814, im: 0.532229983 },
  Complex { re: 0.844853565, im: 0.534997620 }, Complex { re: 0.843098269, im: 0.537759527 }, Complex { re: 0.841333944, im: 0.540515675 }, Complex { re: 0.839560608, im: 0.543266035 },
  Complex { re: 0.837778282, im: 0.546010577 }, Complex { re: 0.835986984, im: 0.548749271 }, Complex { re: 0.834186733, im: 0.551482089 }, Complex { re: 0.832377549, im: 0.554209001 },
  Complex { re: 0.830559450, im: 0.556929978 }, Complex { re: 0.828732457, im: 0.559644990 }, Complex { re: 0.826896589, im: 0.562354009 }, Complex { re: 0.825051865, im: 0.565057006 },
  Complex { re: 0.823198306, im: 0.567753951 }, Complex { re: 0.821335931, im: 0.570444817 }, Complex { re: 0.819464760, im: 0.573129573 }, Complex { re: 0.817584813, im: 0.575808191 },
  Complex { re: 0.815696111, im: 0.578480643 }, Complex { re: 0.813798673, im: 0.581146900 }, Complex { re: 0.811892520, im: 0.583806934 }, Complex { re: 0.809977672, im: 0.586460715 },
  Complex { re: 0.808054150, im: 0.589108216 }, Complex { re: 0.806121975, im: 0.591749408 }, Complex { re: 0.804181167, im: 0.594384262 }, Complex { re: 0.802231746, im: 0.597012752 },
  Complex { re: 0.800273734, im: 0.599634848 }, Complex { re: 0.798307152, im: 0.602250522 }, Complex { re: 0.796332021, im: 0.604859746 }, Complex { re: 0.794348361, im: 0.607462493 },
  Complex { re: 0.792356195, im: 0.610058735 }, Complex { re: 0.790355543, im: 0.612648443 }, Complex { re: 0.788346428, im: 0.615231591 }, Complex { re: 0.786328869, im: 0.617808149 },
  Complex { re: 0.784302890, im: 0.620378092 }, Complex { re: 0.782268511, im: 0.622941391 }, Complex { re: 0.780225755, im: 0.625498018 }, Complex { re: 0.778174644, im: 0.628047947 },
  Complex { re: 0.776115199, im: 0.630591150 }, Complex { re: 0.774047442, im: 0.633127600 }, Complex { re: 0.771971395, im: 0.635657270 }, Complex { re: 0.769887082, im: 0.638180132 },
  Complex { re: 0.767794524, im: 0.640696160 }, Complex { re: 0.765693743, im: 0.643205326 }, Complex { re: 0.763584762, im: 0.645707605 }, Complex { re: 0.761467604, im: 0.648202968 },
  Complex { re: 0.759342291, im: 0.650691390 }, Complex { re: 0.757208847, im: 0.653172843 }, Complex { re: 0.755067293, im: 0.655647301 }, Complex { re: 0.752917653, im: 0.658114738 },
  Complex { re: 0.750759949, im: 0.660575127 }, Complex { re: 0.748594206, im: 0.663028442 }, Complex { re: 0.746420446, im: 0.665474656 }, Complex { re: 0.744238693, im: 0.667913743 },
  Complex { re: 0.742048969, im: 0.670345678 }, Complex { re: 0.739851298, im: 0.672770434 }, Complex { re: 0.737645704, im: 0.675187985 }, Complex { re: 0.735432211, im: 0.677598305 },
  Complex { re: 0.733210842, im: 0.680001369 }, Complex { re: 0.730981620, im: 0.682397150 }, Complex { re: 0.728744571, im: 0.684785624 }, Complex { re: 0.726499717, im: 0.687166764 },
  Complex { re: 0.724247083, im: 0.689540545 }, Complex { re: 0.721986693, im: 0.691906941 }, Complex { re: 0.719718571, im: 0.694265928 }, Complex { re: 0.717442741, im: 0.696617480 },
  Complex { re: 0.715159228, im: 0.698961572 }, Complex { re: 0.712868056, im: 0.701298178 }, Complex { re: 0.710569250, im: 0.703627274 }, Complex { re: 0.708262835, im: 0.705948834 },
];

#[rustfmt::skip]
pub static SINE_WINDOW_512: [Complex<f32>; 256] = [

  Complex { re: 0.999998823, im: 0.001533980 }, Complex { re: 0.999989411, im: 0.004601926 }, Complex { re: 0.999970586, im: 0.007669829 }, Complex { re: 0.999942350, im: 0.010737659 },
  Complex { re: 0.999904701, im: 0.013805389 }, Complex { re: 0.999857641, im: 0.016872988 }, Complex { re: 0.999801170, im: 0.019940429 }, Complex { re: 0.999735288, im: 0.023007681 },
  Complex { re: 0.999659997, im: 0.026074718 }, Complex { re: 0.999575296, im: 0.029141509 }, Complex { re: 0.999481187, im: 0.032208025 }, Complex { re: 0.999377670, im: 0.035274239 },
  Complex { re: 0.999264747, im: 0.038340120 }, Complex { re: 0.999142419, im: 0.041405641 }, Complex { re: 0.999010686, im: 0.044470772 }, Complex { re: 0.998869550, im: 0.047535484 },
  Complex { re: 0.998719012, im: 0.050599749 }, Complex { re: 0.998559074, im: 0.053663538 }, Complex { re: 0.998389737, im: 0.056726821 }, Complex { re: 0.998211003, im: 0.059789571 },
  Complex { re: 0.998022874, im: 0.062851758 }, Complex { re: 0.997825350, im: 0.065913353 }, Complex { re: 0.997618435, im: 0.068974328 }, Complex { re: 0.997402130, im: 0.072034653 },
  Complex { re: 0.997176437, im: 0.075094301 }, Complex { re: 0.996941358, im: 0.078153242 }, Complex { re: 0.996696895, im: 0.081211447 }, Complex { re: 0.996443051, im: 0.084268888 },
  Complex { re: 0.996179829, im: 0.087325535 }, Complex { re: 0.995907229, im: 0.090381361 }, Complex { re: 0.995625256, im: 0.093436336 }, Complex { re: 0.995333912, im: 0.096490431 },
  Complex { re: 0.995033199, im: 0.099543619 }, Complex { re: 0.994723121, im: 0.102595869 }, Complex { re: 0.994403680, im: 0.105647154 }, Complex { re: 0.994074879, im: 0.108697444 },
  Complex { re: 0.993736722, im: 0.111746711 }, Complex { re: 0.993389211, im: 0.114794927 }, Complex { re: 0.993032350, im: 0.117842062 }, Complex { re: 0.992666142, im: 0.120888087 },
  Complex { re: 0.992290591, im: 0.123932975 }, Complex { re: 0.991905700, im: 0.126976696 }, Complex { re: 0.991511473, im: 0.130019223 }, Complex { re: 0.991107914, im: 0.133060525 },
  Complex { re: 0.990695025, im: 0.136100575 }, Complex { re: 0.990272812, im: 0.139139344 }, Complex { re: 0.989841278, im: 0.142176804 }, Complex { re: 0.989400428, im: 0.145212925 },
  Complex { re: 0.988950265, im: 0.148247679 }, Complex { re: 0.988490793, im: 0.151281038 }, Complex { re: 0.988022017, im: 0.154312973 }, Complex { re: 0.987543942, im: 0.157343456 },
  Complex { re: 0.987056571, im: 0.160372457 }, Complex { re: 0.986559910, im: 0.163399949 }, Complex { re: 0.986053963, im: 0.166425904 }, Complex { re: 0.985538735, im: 0.169450291 },
  Complex { re: 0.985014231, im: 0.172473084 }, Complex { re: 0.984480455, im: 0.175494253 }, Complex { re: 0.983937413, im: 0.178513771 }, Complex { re: 0.983385110, im: 0.181531608 },
  Complex { re: 0.982823551, im: 0.184547737 }, Complex { re: 0.982252741, im: 0.187562129 }, Complex { re: 0.981672686, im: 0.190574755 }, Complex { re: 0.981083391, im: 0.193585587 },
  Complex { re: 0.980484862, im: 0.196594598 }, Complex { re: 0.979877104, im: 0.199601758 }, Complex { re: 0.979260123, im: 0.202607039 }, Complex { re: 0.978633924, im: 0.205610413 },
  Complex { re: 0.977998515, im: 0.208611852 }, Complex { re: 0.977353900, im: 0.211611327 }, Complex { re: 0.976700086, im: 0.214608811 }, Complex { re: 0.976037079, im: 0.217604275 },
  Complex { re: 0.975364885, im: 0.220597690 }, Complex { re: 0.974683511, im: 0.223589029 }, Complex { re: 0.973992962, im: 0.226578264 }, Complex { re: 0.973293246, im: 0.229565366 },
  Complex { re: 0.972584369, im: 0.232550307 }, Complex { re: 0.971866337, im: 0.235533059 }, Complex { re: 0.971139158, im: 0.238513595 }, Complex { re: 0.970402839, im: 0.241491885 },
  Complex { re: 0.969657385, im: 0.244467903 }, Complex { re: 0.968902805, im: 0.247441619 }, Complex { re: 0.968139105, im: 0.250413007 }, Complex { re: 0.967366292, im: 0.253382037 },
  Complex { re: 0.966584374, im: 0.256348682 }, Complex { re: 0.965793359, im: 0.259312915 }, Complex { re: 0.964993253, im: 0.262274707 }, Complex { re: 0.964184064, im: 0.265234030 },
  Complex { re: 0.963365800, im: 0.268190857 }, Complex { re: 0.962538468, im: 0.271145160 }, Complex { re: 0.961702077, im: 0.274096910 }, Complex { re: 0.960856633, im: 0.277046080 },
  Complex { re: 0.960002146, im: 0.279992643 }, Complex { re: 0.959138622, im: 0.282936570 }, Complex { re: 0.958266071, im: 0.285877835 }, Complex { re: 0.957384501, im: 0.288816408 },
  Complex { re: 0.956493919, im: 0.291752263 }, Complex { re: 0.955594334, im: 0.294685372 }, Complex { re: 0.954685755, im: 0.297615707 }, Complex { re: 0.953768190, im: 0.300543241 },
  Complex { re: 0.952841648, im: 0.303467947 }, Complex { re: 0.951906137, im: 0.306389795 }, Complex { re: 0.950961666, im: 0.309308760 }, Complex { re: 0.950008245, im: 0.312224814 },
  Complex { re: 0.949045882, im: 0.315137929 }, Complex { re: 0.948074586, im: 0.318048077 }, Complex { re: 0.947094366, im: 0.320955232 }, Complex { re: 0.946105232, im: 0.323859367 },
  Complex { re: 0.945107193, im: 0.326760452 }, Complex { re: 0.944100258, im: 0.329658463 }, Complex { re: 0.943084437, im: 0.332553370 }, Complex { re: 0.942059740, im: 0.335445147 },
  Complex { re: 0.941026175, im: 0.338333767 }, Complex { re: 0.939983753, im: 0.341219202 }, Complex { re: 0.938932484, im: 0.344101426 }, Complex { re: 0.937872376, im: 0.346980411 },
  Complex { re: 0.936803442, im: 0.349856130 }, Complex { re: 0.935725689, im: 0.352728556 }, Complex { re: 0.934639130, im: 0.355597662 }, Complex { re: 0.933543773, im: 0.358463421 },
  Complex { re: 0.932439629, im: 0.361325806 }, Complex { re: 0.931326709, im: 0.364184790 }, Complex { re: 0.930205023, im: 0.367040346 }, Complex { re: 0.929074581, im: 0.369892447 },
  Complex { re: 0.927935395, im: 0.372741067 }, Complex { re: 0.926787474, im: 0.375586178 }, Complex { re: 0.925630831, im: 0.378427755 }, Complex { re: 0.924465474, im: 0.381265769 },
  Complex { re: 0.923291417, im: 0.384100195 }, Complex { re: 0.922108669, im: 0.386931006 }, Complex { re: 0.920917242, im: 0.389758174 }, Complex { re: 0.919717146, im: 0.392581674 },
  Complex { re: 0.918508394, im: 0.395401479 }, Complex { re: 0.917290997, im: 0.398217562 }, Complex { re: 0.916064966, im: 0.401029897 }, Complex { re: 0.914830312, im: 0.403838458 },
  Complex { re: 0.913587048, im: 0.406643217 }, Complex { re: 0.912335185, im: 0.409444149 }, Complex { re: 0.911074734, im: 0.412241227 }, Complex { re: 0.909805708, im: 0.415034424 },
  Complex { re: 0.908528119, im: 0.417823716 }, Complex { re: 0.907241978, im: 0.420609074 }, Complex { re: 0.905947298, im: 0.423390474 }, Complex { re: 0.904644091, im: 0.426167889 },
  Complex { re: 0.903332368, im: 0.428941292 }, Complex { re: 0.902012144, im: 0.431710658 }, Complex { re: 0.900683429, im: 0.434475961 }, Complex { re: 0.899346237, im: 0.437237174 },
  Complex { re: 0.898000580, im: 0.439994271 }, Complex { re: 0.896646470, im: 0.442747228 }, Complex { re: 0.895283921, im: 0.445496017 }, Complex { re: 0.893912945, im: 0.448240612 },
  Complex { re: 0.892533555, im: 0.450980989 }, Complex { re: 0.891145765, im: 0.453717121 }, Complex { re: 0.889749586, im: 0.456448982 }, Complex { re: 0.888345033, im: 0.459176548 },
  Complex { re: 0.886932119, im: 0.461899791 }, Complex { re: 0.885510856, im: 0.464618686 }, Complex { re: 0.884081259, im: 0.467333209 }, Complex { re: 0.882643340, im: 0.470043332 },
  Complex { re: 0.881197113, im: 0.472749032 }, Complex { re: 0.879742593, im: 0.475450282 }, Complex { re: 0.878279792, im: 0.478147056 }, Complex { re: 0.876808724, im: 0.480839331 },
  Complex { re: 0.875329403, im: 0.483527079 }, Complex { re: 0.873841843, im: 0.486210276 }, Complex { re: 0.872346059, im: 0.488888897 }, Complex { re: 0.870842063, im: 0.491562916 },
  Complex { re: 0.869329871, im: 0.494232309 }, Complex { re: 0.867809497, im: 0.496897049 }, Complex { re: 0.866280954, im: 0.499557113 }, Complex { re: 0.864744258, im: 0.502212474 },
  Complex { re: 0.863199422, im: 0.504863109 }, Complex { re: 0.861646461, im: 0.507508991 }, Complex { re: 0.860085390, im: 0.510150097 }, Complex { re: 0.858516224, im: 0.512786401 },
  Complex { re: 0.856938977, im: 0.515417878 }, Complex { re: 0.855353665, im: 0.518044504 }, Complex { re: 0.853760301, im: 0.520666254 }, Complex { re: 0.852158902, im: 0.523283103 },
  Complex { re: 0.850549481, im: 0.525895027 }, Complex { re: 0.848932055, im: 0.528502002 }, Complex { re: 0.847306639, im: 0.531104001 }, Complex { re: 0.845673247, im: 0.533701002 },
  Complex { re: 0.844031895, im: 0.536292979 }, Complex { re: 0.842382600, im: 0.538879909 }, Complex { re: 0.840725375, im: 0.541461766 }, Complex { re: 0.839060237, im: 0.544038527 },
  Complex { re: 0.837387202, im: 0.546610167 }, Complex { re: 0.835706284, im: 0.549176662 }, Complex { re: 0.834017501, im: 0.551737988 }, Complex { re: 0.832320868, im: 0.554294121 },
  Complex { re: 0.830616400, im: 0.556845037 }, Complex { re: 0.828904115, im: 0.559390712 }, Complex { re: 0.827184027, im: 0.561931121 }, Complex { re: 0.825456154, im: 0.564466242 },
  Complex { re: 0.823720511, im: 0.566996049 }, Complex { re: 0.821977115, im: 0.569520519 }, Complex { re: 0.820225983, im: 0.572039629 }, Complex { re: 0.818467130, im: 0.574553355 },
  Complex { re: 0.816700573, im: 0.577061673 }, Complex { re: 0.814926329, im: 0.579564559 }, Complex { re: 0.813144415, im: 0.582061990 }, Complex { re: 0.811354847, im: 0.584553943 },
  Complex { re: 0.809557642, im: 0.587040394 }, Complex { re: 0.807752818, im: 0.589521319 }, Complex { re: 0.805940391, im: 0.591996695 }, Complex { re: 0.804120377, im: 0.594466499 },
  Complex { re: 0.802292796, im: 0.596930708 }, Complex { re: 0.800457662, im: 0.599389298 }, Complex { re: 0.798614995, im: 0.601842247 }, Complex { re: 0.796764810, im: 0.604289531 },
  Complex { re: 0.794907126, im: 0.606731127 }, Complex { re: 0.793041960, im: 0.609167012 }, Complex { re: 0.791169330, im: 0.611597164 }, Complex { re: 0.789289253, im: 0.614021559 },
  Complex { re: 0.787401747, im: 0.616440175 }, Complex { re: 0.785506830, im: 0.618852988 }, Complex { re: 0.783604519, im: 0.621259977 }, Complex { re: 0.781694832, im: 0.623661118 },
  Complex { re: 0.779777788, im: 0.626056388 }, Complex { re: 0.777853404, im: 0.628445767 }, Complex { re: 0.775921699, im: 0.630829230 }, Complex { re: 0.773982691, im: 0.633206755 },
  Complex { re: 0.772036397, im: 0.635578320 }, Complex { re: 0.770082837, im: 0.637943904 }, Complex { re: 0.768122029, im: 0.640303482 }, Complex { re: 0.766153990, im: 0.642657034 },
  Complex { re: 0.764178741, im: 0.645004537 }, Complex { re: 0.762196298, im: 0.647345969 }, Complex { re: 0.760206682, im: 0.649681307 }, Complex { re: 0.758209910, im: 0.652010531 },
  Complex { re: 0.756206001, im: 0.654333618 }, Complex { re: 0.754194975, im: 0.656650546 }, Complex { re: 0.752176850, im: 0.658961293 }, Complex { re: 0.750151646, im: 0.661265838 },
  Complex { re: 0.748119380, im: 0.663564159 }, Complex { re: 0.746080074, im: 0.665856234 }, Complex { re: 0.744033744, im: 0.668142041 }, Complex { re: 0.741980412, im: 0.670421560 },
  Complex { re: 0.739920095, im: 0.672694769 }, Complex { re: 0.737852815, im: 0.674961646 }, Complex { re: 0.735778589, im: 0.677222170 }, Complex { re: 0.733697438, im: 0.679476320 },
  Complex { re: 0.731609381, im: 0.681724074 }, Complex { re: 0.729514438, im: 0.683965412 }, Complex { re: 0.727412629, im: 0.686200312 }, Complex { re: 0.725303972, im: 0.688428753 },
  Complex { re: 0.723188489, im: 0.690650714 }, Complex { re: 0.721066199, im: 0.692866175 }, Complex { re: 0.718937122, im: 0.695075114 }, Complex { re: 0.716801279, im: 0.697277511 },
  Complex { re: 0.714658688, im: 0.699473345 }, Complex { re: 0.712509371, im: 0.701662595 }, Complex { re: 0.710353347, im: 0.703845241 }, Complex { re: 0.708190637, im: 0.706021261 },
];

