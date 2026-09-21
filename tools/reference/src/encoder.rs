//! AAC-ELD without SBR from the FDK AAC encoder, as raw access units.
//!
//! Through the `-sys` crate rather than the safe wrapper: the wrapper initialises the
//! encoder before the granule length and the SBR mode can be set, and both decide the
//! AudioSpecificConfig.

use std::ptr;

pub struct Encoded {
    pub config: Vec<u8>,
    pub units: Vec<Vec<u8>>,
}

fn check(e: sys::AACENC_ERROR, what: &str) {
    assert_eq!(e, 0, "{what} failed: {e:#x}");
}

/// Encode interleaved `pcm`; a last partial frame is dropped.
pub fn encode(pcm: &[i16], rate: u32, channels: usize, bitrate: u32, granule: u32) -> Encoded {
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
        let config = info.confBuf[..info.confSize as usize].to_vec();

        let mut outbuf = vec![0u8; info.maxOutBufBytes as usize];
        let mut units = Vec::new();
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
                units.push(outbuf[..n].to_vec());
            }
        }
        sys::aacEncClose(&mut h);
        Encoded { config, units }
    }
}
