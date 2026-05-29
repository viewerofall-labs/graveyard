use bevy::prelude::*;

#[derive(Resource)]
pub struct Sfx {
    pub punch: Handle<AudioSource>,
    pub smite: Handle<AudioSource>,
    pub fall: Handle<AudioSource>,
    pub trip: Handle<AudioSource>,
    pub door: Handle<AudioSource>,
    pub dismember: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
    pub bounce: Handle<AudioSource>,
    pub piano: Handle<AudioSource>,
    pub ascend: Handle<AudioSource>,
}

pub fn play_sfx(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((
        AudioPlayer::<AudioSource>(handle.clone()),
        PlaybackSettings::DESPAWN,
    ));
}

// --- Procedural SFX generation -----------------------------------------------

const SR: u32 = 22050;

pub fn ensure_sfx(cache_root: &std::path::Path) {
    let dir = cache_root.join("sfx");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    let files: &[(&str, fn() -> Vec<i16>)] = &[
        ("punch.wav", synth_punch),
        ("smite.wav", synth_smite),
        ("fall.wav", synth_fall),
        ("trip.wav", synth_trip),
        ("door.wav", synth_door),
        ("dismember.wav", synth_dismember),
        ("death.wav", synth_death),
        ("bounce.wav", synth_bounce),
        ("piano.wav", synth_piano_crash),
        ("ascend.wav", synth_ascend),
    ];
    for (name, f) in files {
        let p = dir.join(name);
        if !p.exists() {
            write_wav(&p, &f(), SR);
        }
    }
}

fn write_wav(path: &std::path::Path, samples: &[i16], sample_rate: u32) {
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;
    let mut bytes = Vec::with_capacity(44 + data_size as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&file_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for s in samples {
        bytes.extend_from_slice(&s.to_le_bytes());
    }
    let _ = std::fs::write(path, bytes);
}

fn n_samples(dur_s: f32) -> usize {
    (dur_s * SR as f32) as usize
}

struct Lcg(u32);
impl Lcg {
    fn next_unit(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.0 >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
    }
}

fn synth_punch() -> Vec<i16> {
    let n = n_samples(0.18);
    let mut out = Vec::with_capacity(n);
    let mut rng = Lcg(0xdeadbeef);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let env = (-t * 30.0).exp();
        let bass = (t * 70.0 * std::f32::consts::TAU).sin();
        let noise = rng.next_unit();
        let s = (bass * 0.7 + noise * 0.45) * env * i16::MAX as f32 * 0.55;
        out.push(s.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
    }
    out
}

fn synth_smite() -> Vec<i16> {
    let n = n_samples(0.55);
    let mut out = Vec::with_capacity(n);
    let mut rng = Lcg(0xc0deba5e);
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let freq = (1400.0 + (t * 6.0).sin() * 800.0 - t * 1600.0).max(80.0);
        phase += freq * std::f32::consts::TAU / SR as f32;
        let env = if t < 0.04 { t / 0.04 } else { ((0.55 - t) / 0.51).max(0.0) };
        let noise = rng.next_unit();
        let tone = phase.sin();
        let s = (tone * 0.45 + noise * 0.45) * env * i16::MAX as f32 * 0.5;
        out.push(s.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
    }
    out
}

fn synth_fall() -> Vec<i16> {
    let n = n_samples(0.26);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let env = (-t * 14.0).exp();
        let bass = (t * 45.0 * std::f32::consts::TAU).sin();
        let s = bass * env * i16::MAX as f32 * 0.6;
        out.push(s as i16);
    }
    out
}

fn synth_trip() -> Vec<i16> {
    let n = n_samples(0.18);
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let freq = (900.0 - t * 2200.0).max(120.0);
        phase += freq * std::f32::consts::TAU / SR as f32;
        let env = (1.0 - t / 0.18).max(0.0);
        let s = phase.sin() * env * i16::MAX as f32 * 0.45;
        out.push(s as i16);
    }
    out
}

fn synth_door() -> Vec<i16> {
    let n = n_samples(0.55);
    let mut out = Vec::with_capacity(n);
    let mut rng = Lcg(0xa55a5a5a);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let env = if t < 0.05 {
            t / 0.05
        } else if t > 0.50 {
            ((0.55 - t) / 0.05).max(0.0)
        } else {
            1.0
        };
        let mod_freq = (t * 10.0).sin() * 40.0;
        let freq = 170.0 + mod_freq;
        let tone = (t * freq * std::f32::consts::TAU).sin();
        let noise = rng.next_unit();
        let s = (tone * 0.4 + noise * 0.18) * env * i16::MAX as f32 * 0.4;
        out.push(s.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
    }
    out
}

fn synth_dismember() -> Vec<i16> {
    let n = n_samples(0.32);
    let mut out = Vec::with_capacity(n);
    let mut rng = Lcg(0xfeedface);
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let freq = 140.0 + (t * 25.0).sin() * 80.0;
        phase += freq * std::f32::consts::TAU / SR as f32;
        let env = (-t * 9.0).exp();
        let noise = rng.next_unit();
        let tone = phase.sin();
        let s = (tone * 0.4 + noise * 0.65) * env * i16::MAX as f32 * 0.55;
        out.push(s.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
    }
    out
}

fn synth_death() -> Vec<i16> {
    let n = n_samples(0.75);
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let freq = 440.0 * (1.0 - t / 0.75).max(0.2);
        phase += freq * std::f32::consts::TAU / SR as f32;
        let env = if t < 0.05 { t / 0.05 } else { ((0.75 - t) / 0.7).max(0.0) };
        let tone = phase.sin();
        let s = tone * env * i16::MAX as f32 * 0.42;
        out.push(s as i16);
    }
    out
}

fn synth_bounce() -> Vec<i16> {
    let n = n_samples(0.28);
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let freq = 240.0 + t * 900.0;
        phase += freq * std::f32::consts::TAU / SR as f32;
        let env = (-t * 7.0).exp();
        let s = phase.sin() * env * i16::MAX as f32 * 0.55;
        out.push(s as i16);
    }
    out
}

fn synth_piano_crash() -> Vec<i16> {
    let n = n_samples(0.85);
    let mut out = Vec::with_capacity(n);
    let mut rng = Lcg(0xda1aada1);
    // discordant low cluster: minor 2nds piled up
    let freqs = [55.0, 58.27, 65.41, 73.42, 87.31, 110.0, 138.59, 174.61];
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let env = (-t * 5.5).exp();
        let mut s = 0.0;
        for f in freqs {
            s += (t * f * std::f32::consts::TAU).sin();
        }
        s = s / freqs.len() as f32 * 0.75 + rng.next_unit() * 0.35;
        out.push(
            (s * env * i16::MAX as f32 * 0.6).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
        );
    }
    out
}

fn synth_ascend() -> Vec<i16> {
    let n = n_samples(1.4);
    let mut out = Vec::with_capacity(n);
    let mut phase = 0.0_f32;
    for i in 0..n {
        let t = i as f32 / SR as f32;
        // rising chime: sweep up over 1.4s
        let freq = 220.0 * (1.0 + t * 2.5);
        phase += freq * std::f32::consts::TAU / SR as f32;
        let f1 = phase.sin();
        let f2 = (phase * 1.5).sin() * 0.45;
        let f3 = (phase * 2.0).sin() * 0.30;
        let env = if t < 0.12 {
            t / 0.12
        } else {
            ((1.4 - t) / 1.28).max(0.0)
        };
        let s = (f1 + f2 + f3) * env * i16::MAX as f32 * 0.28;
        out.push(s.clamp(i16::MIN as f32, i16::MAX as f32) as i16);
    }
    out
}
