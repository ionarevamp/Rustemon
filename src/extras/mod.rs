extern crate sdl2;
use sdl2::audio::{
    AudioCVT, AudioCallback, AudioDevice, AudioFormatNum, AudioSpec, AudioSpecDesired, AudioSpecWAV,
};
use sdl2::AudioSubsystem;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::thread::{self, *};
use std::time::{Duration, Instant};

pub struct Sound {
    data: Vec<u8>,
    volume: f32,
    pos: usize,
}

impl Sound {
    pub fn volume(&self) -> f32 {
        self.volume.clone()
    }
    pub fn fade_in(&mut self, rate: f32, max: f32) {
        if self.volume < max {
            self.volume += rate;
        }
        if self.volume > max {
            self.volume = max;
        }
    }

    pub fn fade_out(&mut self, rate: f32, min: f32) {
        if self.volume > min {
            self.volume -= rate;
        }
        if self.volume < min {
            self.volume = min;
        }
    }
    // fade_in and fade_out are linear, while fade_percent is logarithmic

    pub fn fade_percent(&mut self, rate: f32, limit: f32) {
        // rate can be negative, and determines whether limit                                                        // is an upper bound or lower bound
        let rate = rate / 100.0;
        if rate < 0.0 {
            if self.volume > limit {
                self.volume *= 1.0 + rate;
            }
            if self.volume < limit {
                self.volume = limit;
            }
        } else {
            if self.volume < 0.1 {
                self.volume = 0.1;
            }
            if self.volume < limit {
                self.volume *= 1.0 + rate;
            }
            if self.volume > limit {
                self.volume = limit;
            }
        }
    }
    pub fn restart(&mut self) {
        self.pos = 128;
    }
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}
pub trait GetElapsed {
    fn nanos(&self) -> u128;
    fn millis(&self) -> u128;
}
impl GetElapsed for Instant {
    fn nanos(&self) -> u128 {
        self.elapsed().as_nanos()
    }
    fn millis(&self) -> u128 {
        self.elapsed().as_millis()
    }
}
impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for dst in out.iter_mut() {
            // With channel type u8 the "silence" value is 128 (middle of the 0-2^8 range) so we need
            // to both fill in the silence and scale the wav data accordingly. Filling the silence                       // once the wav is finished is trivial, applying the volume is more tricky. We need to:                      // * Change the range of the values from [0, 255] to [-128, 127] so we can multiply                          // * Apply the volume by multiplying, this gives us range [-128*volume, 127*volume]
            // * Move the resulting range to a range centered around the value 128, the final range
            //   is [128 - 128*volume, 128 + 127*volume] – scaled and correctly positioned
            //
            // Using value 0 instead of 128 would result in clicking. Scaling by simply multiplying                      // would not give correct results.
            let pre_scale = *self
                .data
                .get(self.pos)
                .unwrap_or(&(Self::Channel::MAX / 2 + 1));
            let scaled_signed_float =
                (pre_scale as f32 - (Self::Channel::MAX / 2 + 1) as f32) * self.volume;
            let scaled =
                (scaled_signed_float + (Self::Channel::MAX / 2 + 1) as f32) as Self::Channel;
            *dst = scaled;
            self.pos += 1;
        }
    }
}
pub fn wav_spec(audio_spec: &AudioSpecWAV, desired_spec: AudioSpec) -> AudioCVT {
    AudioCVT::new(
        audio_spec.format,
        audio_spec.channels,
        audio_spec.freq,
        desired_spec.format,
        desired_spec.channels,
        desired_spec.freq,
    )
    .expect("Could not convert wav file to desired specifications.")
}

pub fn generate_sound(
    subsystem: &AudioSubsystem,
    sound_path: &Cow<'static, Path>,
    volume: f32, //0.0 to 1.0 , 0% to 100% and beyond
    pos: usize,
) -> Result<(AudioDevice<Sound>, AudioSpecWAV)> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100),
        channels: Some(2),
        samples: None,
    };
    let wav = AudioSpecWAV::load_wav(sound_path.clone()).expect(
        format!(
            "Unable to locate {} in working directory",
            sound_path.clone().display()
        )
        .as_str(),
    );
    // tuple starts here
    return Ok(
        (
            subsystem
                .open_playback(None, &desired_spec, |spec| Sound {
                    data: wav_spec(&wav, spec).convert(wav.buffer().to_vec()),
                    volume: volume,
                    pos: pos,
                })
                .unwrap(),
            wav,
        ), //tuple ends here
    );
}

// ^ This returns a device and an audio spec...
