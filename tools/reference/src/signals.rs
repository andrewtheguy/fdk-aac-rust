//! The synthetic signals the fixtures are encoded from.

use std::f64::consts::TAU;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
    }
}

pub fn signal(kind: &str, rate: u32, channels: usize, seconds: f64) -> Vec<i16> {
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
                let off = if (i / (rate as usize / 7)).is_multiple_of(2) { 1.0 } else { 0.2 };
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
