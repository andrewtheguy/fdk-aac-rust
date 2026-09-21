//! Encode synthetic signals to raw AAC-ELD access units, in the `.rawpkts` layout
//! (i32 length + AudioSpecificConfig, then i32 length + access unit, repeated).
use std::f64::consts::TAU;
use std::io::Write;
use std::ptr;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

fn signal(kind: &str, rate: u32, channels: usize, seconds: f64) -> Vec<i16> {
    let n = (rate as f64 * seconds) as usize;
    let mut rng = Rng(0x9e3779b97f4a7c15);
    let mut out = Vec::with_capacity(n * channels);
    let mut env = 0.0f64;
    for i in 0..n {
        let t = i as f64 / rate as f64;
        let (l, r) = match kind {
            // Harmonic, strongly correlated channels with a slow pan: M/S territory.
            "music" => {
                let f0 = 220.0 * (1.0 + 0.01 * (TAU * 5.0 * t).sin());
                let mut s = 0.0;
                for h in 1..=12 {
                    s += (TAU * f0 * h as f64 * t).sin() / h as f64;
                }
                let chord = (TAU * 329.63 * t).sin() * 0.5 + (TAU * 1318.5 * t).sin() * 0.2;
                let pan = 0.5 + 0.4 * (TAU * 0.3 * t).sin();
                let m = 0.25 * (s + chord);
                (m * pan + 0.02 * rng.next(), m * (1.0 - pan) + 0.02 * rng.next())
            }
            // Sharp clicks with decaying noise bursts: TNS territory.
            "transients" => {
                if i % (rate as usize / 7) == 0 {
                    env = 1.0;
                }
                env *= 0.9992;
                let burst = rng.next() * env * 0.8;
                let tone = (TAU * 3000.0 * t).sin() * env * 0.3;
                let off = if (i / (rate as usize / 7)) % 2 == 0 { 1.0 } else { 0.2 };
                ((burst + tone) * off, (burst - tone) * (1.2 - off))
            }
            // Uncorrelated wideband noise: PNS and intensity territory at low rates.
            "noise" => {
                let a = 0.3 * (0.6 + 0.4 * (TAU * 0.7 * t).sin());
                (rng.next() * a, rng.next() * a)
            }
            // Sweep into silence and back, hard-panned halves.
            "sweep" => {
                let f = 50.0 * (400.0f64).powf(t / seconds);
                let gate = if (t * 2.0) as usize % 3 == 2 { 0.0 } else { 0.7 };
                let s = (TAU * f * t).sin() * gate;
                if t < seconds / 2.0 { (s, 0.0) } else { (0.0, s) }
            }
            _ => unreachable!(),
        };
        let q = |x: f64| (x.clamp(-1.0, 1.0) * 32767.0) as i16;
        out.push(q(l));
        if channels == 2 {
            out.push(q(r));
        }
    }
    out
}

fn check(e: sys::AACENC_ERROR, what: &str) {
    assert_eq!(e, 0, "{what} failed: {e:#x}");
}

fn encode(path: &str, kind: &str, rate: u32, channels: usize, bitrate: u32, granule: u32, seconds: f64) {
    unsafe {
        let mut h: sys::HANDLE_AACENCODER = ptr::null_mut();
        check(sys::aacEncOpen(&mut h, 0, channels as u32), "open");
        let set = |p, v| check(sys::aacEncoder_SetParam(h, p, v), "set");
        set(sys::AACENC_PARAM_AACENC_AOT, sys::AUDIO_OBJECT_TYPE_AOT_ER_AAC_ELD as u32);
        set(sys::AACENC_PARAM_AACENC_SAMPLERATE, rate);
        set(sys::AACENC_PARAM_AACENC_CHANNELMODE, channels as u32);
        set(sys::AACENC_PARAM_AACENC_BITRATE, bitrate);
        set(sys::AACENC_PARAM_AACENC_TRANSMUX, sys::TRANSPORT_TYPE_TT_MP4_RAW as u32);
        set(sys::AACENC_PARAM_AACENC_SBR_MODE, 0);
        set(sys::AACENC_PARAM_AACENC_GRANULE_LENGTH, granule);
        set(sys::AACENC_PARAM_AACENC_AFTERBURNER, 1);
        check(sys::aacEncEncode(h, ptr::null(), ptr::null(), ptr::null(), ptr::null_mut()), "init");
        let mut info = std::mem::zeroed::<sys::AACENC_InfoStruct>();
        check(sys::aacEncInfo(h, &mut info), "info");
        assert_eq!(info.frameLength, granule);
        let asc = &info.confBuf[..info.confSize as usize];

        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(&(asc.len() as i32).to_le_bytes()).unwrap();
        file.write_all(asc).unwrap();

        let pcm = signal(kind, rate, channels, seconds);
        let mut outbuf = vec![0u8; info.maxOutBufBytes as usize];
        let (mut units, mut bytes) = (0usize, 0usize);
        for frame in pcm.chunks_exact(granule as usize * channels) {
            let mut in_ptr = frame.as_ptr() as *mut std::ffi::c_void;
            let mut in_id = sys::AACENC_BufferIdentifier_IN_AUDIO_DATA as i32;
            let mut in_size = (frame.len() * 2) as i32;
            let mut in_el = 2i32;
            let inbuf = sys::AACENC_BufDesc { numBufs: 1, bufs: &mut in_ptr, bufferIdentifiers: &mut in_id, bufSizes: &mut in_size, bufElSizes: &mut in_el };
            let mut out_ptr = outbuf.as_mut_ptr() as *mut std::ffi::c_void;
            let mut out_id = sys::AACENC_BufferIdentifier_OUT_BITSTREAM_DATA as i32;
            let mut out_size = outbuf.len() as i32;
            let mut out_el = 1i32;
            let outdesc = sys::AACENC_BufDesc { numBufs: 1, bufs: &mut out_ptr, bufferIdentifiers: &mut out_id, bufSizes: &mut out_size, bufElSizes: &mut out_el };
            let inargs = sys::AACENC_InArgs { numInSamples: frame.len() as i32, numAncBytes: 0 };
            let mut outargs = std::mem::zeroed::<sys::AACENC_OutArgs>();
            check(sys::aacEncEncode(h, &inbuf, &outdesc, &inargs, &mut outargs), "encode");
            let n = outargs.numOutBytes as usize;
            if n > 0 {
                file.write_all(&(n as i32).to_le_bytes()).unwrap();
                file.write_all(&outbuf[..n]).unwrap();
                units += 1;
                bytes += n;
            }
        }
        sys::aacEncClose(&mut h);
        let hex: String = asc.iter().map(|b| format!("{b:02x}")).collect();
        println!("{path}: asc={hex} units={units} bytes={bytes}");
    }
}

fn main() {
    let dir = std::env::args().nth(1).expect("output directory");
    std::fs::create_dir_all(&dir).unwrap();
    // The Apple Screen Sharing shape: 48 kHz, stereo, 480-sample frames, no SBR.
    for (kind, bitrate) in [("music", 128_000), ("transients", 96_000), ("noise", 32_000), ("sweep", 64_000), ("music", 24_000), ("noise", 256_000)] {
        encode(&format!("{dir}/eld_48k_stereo_480_{kind}_{}k.rawpkts", bitrate / 1000), kind, 48_000, 2, bitrate, 480, 3.0);
    }
    // The rest of what plain AAC-ELD allows and the tables still carry.
    encode(&format!("{dir}/eld_48k_stereo_512_music_96k.rawpkts"), "music", 48_000, 2, 96_000, 512, 2.0);
    encode(&format!("{dir}/eld_44k_stereo_480_transients_64k.rawpkts"), "transients", 44_100, 2, 64_000, 480, 2.0);
    encode(&format!("{dir}/eld_48k_mono_480_music_48k.rawpkts"), "music", 48_000, 1, 48_000, 480, 2.0);
    encode(&format!("{dir}/eld_32k_mono_512_noise_24k.rawpkts"), "noise", 32_000, 1, 24_000, 512, 2.0);
}
