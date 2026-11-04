use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rodio::mixer::Mixer;
use rodio::{
    ChannelCount, Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, SampleRate, Source,
};

use crate::config::{Config, Paths};

#[derive(Clone, Copy, Debug)]
pub enum Sfx {
    Navigate,
    Select,
    Back,
    Toggle,
    Launch,
    Notice,
    Slide,
    SlideOut,
}

impl Sfx {
    pub fn name(self) -> &'static str {
        match self {
            Self::Navigate => "navigate",
            Self::Select => "select",
            Self::Back => "back",
            Self::Toggle => "toggle",
            Self::Launch => "launch",
            Self::Notice => "notice",
            Self::Slide => "slide",
            Self::SlideOut => "slide_out",
        }
    }

    fn embedded(self) -> &'static [u8] {
        match self {
            Self::Navigate => include_bytes!("../assets/sounds/navigate.wav"),
            Self::Select => include_bytes!("../assets/sounds/select.wav"),
            Self::Back => include_bytes!("../assets/sounds/back.wav"),
            Self::Toggle => include_bytes!("../assets/sounds/toggle.wav"),
            Self::Launch => include_bytes!("../assets/sounds/launch.wav"),
            Self::Notice => include_bytes!("../assets/sounds/notice.wav"),
            Self::Slide => include_bytes!("../assets/sounds/launch.wav"),
            Self::SlideOut => include_bytes!("../assets/sounds/back.wav"),
        }
    }

    /// The 2016 shell mixed navigation ticks well below the confirm sounds.
    /// Keeping the source files untouched but balancing them here stops the
    /// click track from overpowering the menu music.
    fn gain(self) -> f32 {
        match self {
            Self::Navigate => 0.22,
            Self::Select => 0.42,
            Self::Back => 0.38,
            Self::Toggle => 0.40,
            Self::Launch => 0.52,
            Self::Notice => 0.28,
            Self::Slide => 0.45,
            Self::SlideOut => 0.45,
        }
    }
}

pub enum MusicStatus {
    NotInstalled,
    Playing,
    Muted,
    Unavailable(String),
    Disabled,
}

/// A gapless in-memory loop. The decoder is not involved at the loop point.
/// The first pass plays the original track; later passes start with a
/// crossfade of the tail into the head, so a recording that fades out at the
/// end does not restart quietly and then jump back up in volume.
struct LoopSource {
    intro: Arc<Vec<f32>>,
    loop_samples: Arc<Vec<f32>>,
    channels: ChannelCount,
    sample_rate: SampleRate,
    position: usize,
    first_pass: bool,
}

impl Iterator for LoopSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let samples = if self.first_pass {
            &self.intro
        } else {
            &self.loop_samples
        };
        let sample = *samples.get(self.position)?;
        self.position += 1;
        if self.position >= samples.len() {
            self.position = 0;
            self.first_pass = false;
        }
        Some(sample)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl Source for LoopSource {
    // This source never ends, so it has no finite span. Returning a span here
    // makes rodio split the stream into Take blocks and rebuild the resampler
    // at every boundary, which produced the audible restart/volume drift.
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct Music {
    _device: Option<MixerDeviceSink>,
    player: Option<Player>,
    sfx_mixer: Option<Mixer>,
    active_sfx: Vec<(Instant, Player)>,
    path: Option<PathBuf>,
    status: MusicStatus,
    level: f32,
    fade: f32,
    fade_target: f32,
    duck: f32,
    duck_target: f32,
    focus_muted: bool,
    focus_gain: f32,
    game_running: bool,
}

impl Music {
    pub fn new(paths: &Paths, config: &Config, no_audio: bool) -> Self {
        if no_audio {
            return Self::without_audio(paths.music_file(), MusicStatus::Disabled);
        }

        let mut device = match DeviceSinkBuilder::open_default_sink() {
            Ok(device) => device,
            Err(error) => {
                return Self::without_audio(
                    paths.music_file(),
                    MusicStatus::Unavailable(error.to_string()),
                );
            }
        };
        device.log_on_drop(false);
        let sfx_mixer = Some(device.mixer().clone());

        let Some(path) = paths.music_file() else {
            return Self {
                _device: Some(device),
                player: None,
                sfx_mixer,
                active_sfx: Vec::new(),
                path: None,
                status: MusicStatus::NotInstalled,
                level: 0.0,
                fade: 0.0,
                fade_target: 0.0,
                duck: 1.0,
                duck_target: 1.0,
                focus_muted: false,
                focus_gain: 1.0,
                game_running: false,
            };
        };

        let decoded = File::open(&path)
            .map_err(|error| error.to_string())
            .and_then(|file| Decoder::new(BufReader::new(file)).map_err(|error| error.to_string()));

        match decoded {
            Ok(decoder) => {
                let channels = decoder.channels();
                let sample_rate = decoder.sample_rate();
                let samples: Vec<f32> = decoder.collect();
                let samples = trim_edge_silence(samples, channels.get() as usize);
                let samples = normalize_quiet_sections(
                    samples,
                    channels.get() as usize,
                    sample_rate.get() as usize,
                );
                if samples.is_empty() {
                    return Self {
                        _device: Some(device),
                        player: None,
                        sfx_mixer,
                        active_sfx: Vec::new(),
                        path: Some(path),
                        status: MusicStatus::Unavailable(
                            "decoded menu music contains no samples".to_owned(),
                        ),
                        level: 0.0,
                        fade: 0.0,
                        fade_target: 0.0,
                        duck: 1.0,
                        duck_target: 1.0,
                        focus_muted: false,
                        focus_gain: 1.0,
                        game_running: false,
                    };
                }

                let (intro, loop_samples) = build_loop_buffers(
                    samples,
                    channels.get() as usize,
                    sample_rate.get() as usize,
                );
                let frames = intro.len() / channels.get() as usize;
                let seconds = frames as f32 / sample_rate.get() as f32;
                log::info!(
                    "menu music decoded: {frames} frames, {:.3}s loop, {:.1}s crossfade, no decoder or resampler restart at the loop point",
                    seconds,
                    CROSSFADE_SECONDS
                );

                let level = effective_volume(config);
                let (player, queue) = Player::new();
                player.set_volume(level);
                player.append(LoopSource {
                    intro: Arc::new(intro),
                    loop_samples: Arc::new(loop_samples),
                    channels,
                    sample_rate,
                    position: 0,
                    first_pass: true,
                });
                device.mixer().add(queue);
                Self {
                    _device: Some(device),
                    player: Some(player),
                    sfx_mixer,
                    active_sfx: Vec::new(),
                    path: Some(path),
                    status: if config.muted {
                        MusicStatus::Muted
                    } else {
                        MusicStatus::Playing
                    },
                    level,
                    fade: 1.0,
                    fade_target: 1.0,
                    duck: 1.0,
                    duck_target: 1.0,
                    focus_muted: false,
                    focus_gain: 1.0,
                    game_running: false,
                }
            }
            Err(error) => Self {
                _device: Some(device),
                player: None,
                sfx_mixer,
                active_sfx: Vec::new(),
                path: Some(path),
                status: MusicStatus::Unavailable(error),
                level: 0.0,
                fade: 0.0,
                fade_target: 0.0,
                duck: 1.0,
                duck_target: 1.0,
                focus_muted: false,
                focus_gain: 1.0,
                game_running: false,
            },
        }
    }

    fn without_audio(path: Option<PathBuf>, status: MusicStatus) -> Self {
        Self {
            _device: None,
            player: None,
            sfx_mixer: None,
            active_sfx: Vec::new(),
            path,
            status,
            level: 0.0,
            fade: 0.0,
            fade_target: 0.0,
            duck: 1.0,
            duck_target: 1.0,
            focus_muted: false,
            focus_gain: 1.0,
            game_running: false,
        }
    }

    pub fn reload(&mut self, paths: &Paths, config: &Config) {
        let game_running = self.game_running;
        let duck = self.duck;
        let duck_target = self.duck_target;
        *self = Self::new(paths, config, false);
        self.game_running = game_running;
        self.duck = duck;
        self.duck_target = duck_target;
        if game_running {
            self.fade = 0.0;
            self.fade_target = 0.0;
            self.apply_volume();
        }
    }

    pub fn set_levels(&mut self, config: &Config) {
        self.level = effective_volume(config);
        if matches!(self.status, MusicStatus::Playing | MusicStatus::Muted) {
            self.status = if config.muted {
                MusicStatus::Muted
            } else {
                MusicStatus::Playing
            };
        }
        self.apply_volume();
    }

    pub fn set_focus_muted(&mut self, muted: bool) {
        self.focus_muted = muted;
        self.focus_gain = if muted { 0.0 } else { 1.0 };
        self.apply_volume();
    }

    pub fn set_launching(&mut self, launching: bool) {
        self.fade_target = if launching { 0.0 } else { 1.0 };
    }

    /// Keep the menu theme silent while a launched experience is running.
    /// The shell may still be visible behind Sober, but the game owns the
    /// audio session at that point.
    pub fn set_game_running(&mut self, running: bool) {
        self.game_running = running;
    }

    /// Lower the theme while the user is browsing game lists so the UI sound
    /// effects sit above it instead of being buried.
    pub fn set_browsing(&mut self, browsing: bool) {
        self.duck_target = if browsing { 0.55 } else { 1.0 };
    }

    pub fn update(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        let target = if self.game_running {
            0.0
        } else {
            self.fade_target
        };
        let mut changed = false;
        if (self.fade - target).abs() < 0.001 {
            if self.fade != target {
                self.fade = target;
                changed = true;
            }
        } else {
            let duration = if target > self.fade { 1.5 } else { 0.8 };
            let step = (dt / duration).clamp(0.0, 1.0);
            if target > self.fade {
                self.fade = (self.fade + step).min(target);
            } else {
                self.fade = (self.fade - step).max(target);
            }
            changed = true;
        }

        if (self.duck - self.duck_target).abs() < 0.001 {
            if self.duck != self.duck_target {
                self.duck = self.duck_target;
                changed = true;
            }
        } else {
            let step = (dt / 1.2).clamp(0.0, 1.0);
            self.duck += (self.duck_target - self.duck) * step;
            changed = true;
        }

        if changed {
            self.apply_volume();
        }
    }

    pub fn play_sfx(&mut self, effect: Sfx, paths: &Paths, config: &Config) {
        if !config.ui_sounds
            || config.muted
            || self.focus_muted
            || self.game_running
            || config.master_volume <= 0.001
        {
            return;
        }
        let Some(mixer) = &self.sfx_mixer else {
            return;
        };
        let volume = (config.master_volume * effect.gain()).clamp(0.0, 1.0);

        if let Some(path) = paths.sfx_file(effect.name())
            && let Ok(file) = File::open(&path)
            && let Ok(decoder) = Decoder::new(BufReader::new(file))
        {
            let (player, queue) = Player::new();
            player.set_volume(volume);
            player.append(decoder);
            mixer.add(queue);
            self.active_sfx.push((Instant::now(), player));
            self.cleanup_sfx();
            log::debug!("sfx {:?} from {}", effect, path.display());
            return;
        }

        if let Ok(decoder) = Decoder::new(Cursor::new(effect.embedded())) {
            let (player, queue) = Player::new();
            player.set_volume(volume);
            player.append(decoder);
            mixer.add(queue);
            self.active_sfx.push((Instant::now(), player));
            self.cleanup_sfx();
            log::debug!("sfx {:?} from embedded fallback", effect);
        }
    }

    fn cleanup_sfx(&mut self) {
        self.active_sfx.retain(|(started, player)| {
            !player.empty() && started.elapsed() < Duration::from_secs(5)
        });
    }

    fn apply_volume(&mut self) {
        if let Some(player) = &self.player {
            let volume = self.level * self.fade * self.duck * self.focus_gain;
            player.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    pub fn status_text(&self) -> String {
        if self.game_running {
            return "Menu music paused while the game runs".to_owned();
        }
        match &self.status {
            MusicStatus::NotInstalled => "Menu music not installed".to_owned(),
            MusicStatus::Playing => "Playing menu music".to_owned(),
            MusicStatus::Muted => "Menu music muted".to_owned(),
            MusicStatus::Unavailable(error) => format!("Audio unavailable: {error}"),
            MusicStatus::Disabled => "Audio disabled for this run".to_owned(),
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

// Keep the loop blend short. A long equal-power crossfade made the tail and
// head audible at the same time, which sounded like the song restarted twice.
// 20 ms is enough to remove the sample-level click without ever overlapping
// two musically distinct passages.
const CROSSFADE_SECONDS: f32 = 0.20;

/// Build the first-pass buffer and the crossfaded loop buffer. The intro is
/// the original track minus its fading tail. The tail is folded into the
/// start of the loop buffer, and the first pass transitions into it exactly
/// where the original tail begins.
fn build_loop_buffers(
    samples: Vec<f32>,
    channels: usize,
    sample_rate: usize,
) -> (Vec<f32>, Vec<f32>) {
    let channels = channels.max(1);
    let frames = samples.len() / channels;
    if frames == 0 {
        return (samples.clone(), samples);
    }
    let fade_frames = ((CROSSFADE_SECONDS * sample_rate as f32) as usize).min(frames / 4);
    if fade_frames == 0 {
        return (samples.clone(), samples);
    }
    let cut = frames - fade_frames;
    let intro = samples[..cut * channels].to_vec();
    let mut loop_samples = intro.clone();
    for frame in 0..fade_frames {
        let phase = (frame as f32 + 1.0) / fade_frames as f32 * std::f32::consts::FRAC_PI_2;
        let tail_gain = phase.cos();
        let head_gain = phase.sin();
        for channel in 0..channels {
            let tail = samples[(cut + frame) * channels + channel];
            let head = samples[frame * channels + channel];
            loop_samples[frame * channels + channel] = tail * tail_gain + head * head_gain;
        }
    }
    (intro, loop_samples)
}

/// Give quiet passages a modest amount of makeup gain so the menu theme does
/// not appear to change volume between loop iterations. Loud passages are
/// left alone; the gain envelope is smoothed to avoid pumping.
fn normalize_quiet_sections(samples: Vec<f32>, channels: usize, sample_rate: usize) -> Vec<f32> {
    let channels = channels.max(1);
    let frames = samples.len() / channels;
    if frames == 0 {
        return samples;
    }
    let window = (sample_rate / 2).max(1);
    let window_count = frames.div_ceil(window);
    if window_count == 0 {
        return samples;
    }
    let mut gains = Vec::with_capacity(window_count);
    for index in 0..window_count {
        let start = index * window;
        let end = (start + window).min(frames);
        let mut sum = 0.0;
        for frame in start..end {
            for channel in 0..channels {
                let sample = samples[frame * channels + channel];
                sum += sample * sample;
            }
        }
        let rms = (sum / ((end - start) * channels).max(1) as f32).sqrt();
        let gain = if rms > 0.01 && rms < 0.16 {
            (0.16 / rms).clamp(1.0, 2.5)
        } else {
            1.0
        };
        gains.push(gain);
    }
    let mut samples = samples;
    let attack = 1.0 - (-1.0 / (sample_rate as f32 * 0.08)).exp();
    let mut current_gain = 1.0f32;
    for frame in 0..frames {
        let target = gains[(frame / window).min(gains.len() - 1)];
        current_gain += (target - current_gain) * attack;
        for channel in 0..channels {
            samples[frame * channels + channel] =
                (samples[frame * channels + channel] * current_gain).clamp(-1.0, 1.0);
        }
    }
    samples
}

fn trim_edge_silence(samples: Vec<f32>, channels: usize) -> Vec<f32> {
    let channels = channels.max(1);
    let frames = samples.len() / channels;
    if frames == 0 {
        return samples;
    }
    let silent_frame = |frame: usize| {
        (0..channels).all(|channel| samples[frame * channels + channel].abs() <= 1.0e-6)
    };

    let mut start = 0;
    while start < frames && silent_frame(start) {
        start += 1;
    }
    let mut end = frames;
    while end > start && silent_frame(end - 1) {
        end -= 1;
    }
    if start >= end {
        return samples;
    }
    samples[start * channels..end * channels].to_vec()
}

fn effective_volume(config: &Config) -> f32 {
    if config.muted {
        0.0
    } else {
        (config.master_volume * config.music_volume).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_source_wraps_without_a_gap() {
        let data = Arc::new(vec![0.1, 0.2, 0.3, 0.4]);
        let mut source = LoopSource {
            intro: data.clone(),
            loop_samples: data,
            channels: std::num::NonZero::new(2).unwrap(),
            sample_rate: std::num::NonZero::new(48_000).unwrap(),
            position: 0,
            first_pass: true,
        };
        let samples: Vec<f32> = (0..10).map(|_| source.next().unwrap()).collect();
        assert_eq!(
            samples,
            vec![0.1, 0.2, 0.3, 0.4, 0.1, 0.2, 0.3, 0.4, 0.1, 0.2]
        );
    }

    #[test]
    fn infinite_loop_reports_no_finite_span() {
        let source = LoopSource {
            intro: Arc::new(vec![0.1, 0.2]),
            loop_samples: Arc::new(vec![0.1, 0.2]),
            channels: std::num::NonZero::new(1).unwrap(),
            sample_rate: std::num::NonZero::new(48_000).unwrap(),
            position: 0,
            first_pass: true,
        };
        assert!(
            source.current_span_len().is_none(),
            "an endless source must not advertise a finite span"
        );
    }

    #[test]
    fn uniform_source_reads_through_the_loop_boundary() {
        let source = LoopSource {
            intro: Arc::new(vec![0.1, 0.2, 0.3, 0.4]),
            loop_samples: Arc::new(vec![0.1, 0.2, 0.3, 0.4]),
            channels: std::num::NonZero::new(1).unwrap(),
            sample_rate: std::num::NonZero::new(48_000).unwrap(),
            position: 0,
            first_pass: true,
        };
        let mut uniform = rodio::source::UniformSourceIterator::new(
            source,
            std::num::NonZero::new(1).unwrap(),
            std::num::NonZero::new(48_000).unwrap(),
        );
        let samples: Vec<f32> = (0..12).map(|_| uniform.next().unwrap()).collect();
        assert_eq!(
            samples,
            vec![0.1, 0.2, 0.3, 0.4, 0.1, 0.2, 0.3, 0.4, 0.1, 0.2, 0.3, 0.4]
        );
    }

    #[test]
    fn short_crossfade_keeps_the_loop_level_continuous() {
        // A constant signal exposes a duplicated or missing block at the
        // boundary; the old long crossfade failed this by overlapping two
        // distinct musical passages.
        let samples = vec![0.25_f32; 4000];
        let (intro, loop_samples) = build_loop_buffers(samples, 1, 8000);
        assert_eq!(intro.len(), loop_samples.len());
        assert!((intro.first().copied().unwrap_or_default() - 0.25).abs() < 1.0e-6);
        assert!((intro.last().copied().unwrap_or_default() - 0.25).abs() < 1.0e-6);
        assert!((loop_samples.first().copied().unwrap_or_default() - 0.25).abs() < 0.01);
        assert!((loop_samples.last().copied().unwrap_or_default() - 0.25).abs() < 0.01);
    }

    #[test]
    fn edge_silence_is_trimmed_at_frame_boundaries() {
        let trimmed = trim_edge_silence(vec![0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.0, 0.0], 2);
        assert_eq!(trimmed, vec![0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn embedded_sfx_files_decode() {
        for effect in [
            Sfx::Navigate,
            Sfx::Select,
            Sfx::Back,
            Sfx::Toggle,
            Sfx::Launch,
            Sfx::Notice,
            Sfx::Slide,
            Sfx::SlideOut,
        ] {
            assert!(
                Decoder::new(Cursor::new(effect.embedded())).is_ok(),
                "{} should decode",
                effect.name()
            );
        }
    }
}
