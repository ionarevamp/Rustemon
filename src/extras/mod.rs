extern crate sdl2;
use sdl2::audio::{
    AudioCVT, AudioCallback, AudioDevice, AudioFormat::{self, *}, AudioFormatNum, AudioSpec, AudioSpecDesired, AudioSpecWAV, AudioQueue
};
use sdl2::AudioSubsystem;
use sdl2::{sys, get_error};
use sdl2::mixer::{self, *};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::thread::{self, *};
use std::time::{Duration, Instant};


// TODO: change `data` to be of type `Vec<(u8,f32)>` (preferably aligned)
// TODO: ^^ also change fade funcs
#[repr(Rust, align(128))]
pub struct Sound {
    data: Vec<(u8, f32)>,
    //volume: f32,
    pos: usize,
}
pub struct OneWaySound {
    data: Vec<(u8, f32)>
}

//TODO: add proper read-ahead feature to fade functions to ensure smooth volume control
// NOTE: Look-ahead must exceed provided sample size given when generating an audio device with a
// buffer defined by a sound file 
impl Sound {
    pub fn volume(&self) -> f32 {
        self.data[self.pos].1.clone()
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
    pub fn len(&self) -> usize {
        self.data.len() - 1
    }
    pub fn fade_in(&mut self, rate: f32, max: f32) {
        for p in self.pos..(self.pos+u16::MAX as usize) {
            if p >= self.len() { break; }
            if self.data[p].1 < max {
                self.data[p].1 += rate;
            }
            if self.data[p].1 > max {
                self.data[p].1 = max;
            }
        }
    }

    pub fn fade_out(&mut self, rate: f32, min: f32) {
        for p in self.pos..(self.pos+u16::MAX as usize) {
            if p >= self.len() { break; }
            if self.data[p].1 > min {
                self.data[p].1 -= rate;
            }
            if self.data[p].1 < min {
                self.data[p].1 = min;
            }
        }
    }
    // fade_in and fade_out are linear, while fade_percent is logarithmic

    pub fn fade_percent(&mut self, rate: f32, limit: f32) {
        // rate can be negative, and determines whether limit
        // is an upper bound or lower bound
        //
        let rate = rate / 100.0;
        for p in self.pos..(self.pos+u16::MAX as usize) {
            if p >= self.len() { break; }
            if rate < 0.0 {
                if self.data[p].1 > limit {
                    self.data[p].1 *= (1.0 + rate);
                }
                if self.data[p].1 < limit {
                    self.data[p].1 = limit;
                }
            } else if rate >= 0.0 {
                if self.data[p].1 < 0.1 {
                    self.data[p].1 = 0.1;
                }
                if self.data[p].1 < limit {
                    self.data[p].1 *= (1.0 + rate);
                }
                if self.data[p].1 > limit {
                    self.data[p].1 = limit;
                }
            }
        }
    }
    pub fn restart(&mut self) {
        self.pos = 0;
    }
    pub fn seek(&mut self, move_pos: i32) {
        if move_pos < 0i32 {
            if move_pos.abs() - self.pos as i32 <= 0i32 {
                self.pos = 0;
            }
            else {
                self.pos -= move_pos.abs() as usize;
            }
        }
        else {
            self.pos += move_pos as usize;
        }
    }
    //Updates the volume for the rest of the buffer.
    pub fn set_volume(&mut self, volume: f32) {
        let len = self.data.len()-1;
        for p in self.pos..=len {
            self.data[p].1 = volume;
        }
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

    //TODO: Change to account for `self.data` being a `Vec<(u8, f32)>`
    fn callback(&mut self, out: &mut [Self::Channel]) {
        for dst in out.iter_mut() {
            // With channel type u8 the "silence" value is 128 (middle of the 0-2^8 range) so we need
            // to both fill in the silence and scale the wav data accordingly. Filling the silence                       // once the wav is finished is trivial, applying the volume is more tricky. We need to:                      // * Change the range of the values from [0, 255] to [-128, 127] so we can multiply                          // * Apply the volume by multiplying, this gives us range [-128*volume, 127*volume]
            // * Move the resulting range to a range centered around the value 128, the final range
            //   is [128 - 128*volume, 128 + 127*volume] – scaled and correctly positioned
            //
            // Using value 0 instead of 128 would result in clicking. Scaling by simply multiplying                      // would not give correct results.
            //
            let pre_scale = (self
                .data //tuple vector
                .get(self.pos)
                .unwrap_or(&(128,0.0))
                );
            let scaled_signed_float =
                (pre_scale.0 as f32 - 128.0) * pre_scale.1;
            let mut scaled =
                (scaled_signed_float + 128.0) as Self::Channel;


            *dst = scaled;
            self.pos += 1;

        }
    }
}

// TODO: include_bytes! on necessary files and parse the data into 
//   a Sound instance
/*
pub fn include_wav(src: Cow<'static, Path>) -> Result<AudioSpecWAV, String> {

    use std::mem::MaybeUninit;
    use std::ptr::null_mut;

    let data = include_bytes!(src);

    let mut desired = MaybeUninit::uninit();
    let mut audio_buf: *mut u8 = null_mut();
    let mut audio_len: u32 = 0;
    unsafe {
        let ret = sys::SDL_LoadWAV_RW(
            data,
            0,
            desired.as_mut_ptr(),
            &mut audio_buf,
            &mut audio_len,
        );
        if ret.is_null() {
            Err(get_error())
        } else {
            let desired = desired.assume_init();
            Ok(AudioSpecWAV {
                freq: desired.freq,
                format: AudioFormat::from_ll(desired.format).unwrap(),
                channels: desired.channels,
                audio_buf,
                audio_len,
            })
        }
    }
}
*/


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

pub trait CustomQueue {
    fn queue(&mut self, file: &Cow<'static, Path>, volume: f32);
}

//TODO: Flush already played positions from buffer when playing, only appending new data
//TODO: Define volume settings per audiofile and refer to it during playback; otherwise, normalize or limit audio amplitude
//  - If audio is defined per file, each position in the buffer must be a vector of tuples: one for
//  the sound data and one for the volume
impl CustomQueue for Sound {
    fn queue(&mut self, file: &Cow<'static, Path>, volume: f32) {
        let desired_spec = AudioSpecDesired {
            freq: Some(44_100), //Default should be 44_100 
            channels: Some(2),
            samples: None,
        };
        
        let wav_append = AudioSpecWAV::load_wav(file.clone()).expect("Could not load wav file");
        let mut wav_append = wav_spec(&wav_append,
                                AudioSpec {
                                    freq: desired_spec.freq.unwrap(),
                                    format: AudioFormat::U8,
                                    channels: desired_spec.channels.unwrap(),
                                    silence: 128,
                                    samples: 256u16,
                                    size: 44_100,
                                }
                                ).convert(wav_append.buffer().to_vec())
            .into_iter()
            .map(|x| (x, volume))
            .collect::<Vec<(u8,f32)>>();
        
        self.data.append(&mut wav_append);

    }
}


// OBSOLETE 
pub fn generate_sound(
    subsystem: &AudioSubsystem,
    sound_path: &Cow<'static, Path>,
    volume: f32, //0.0 to 1.0 , 0% to 100% and beyond
    pos: usize,
) -> Result<AudioDevice<Sound>> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44_100), //Default should be 44_100 
        channels: Some(2),
        samples: None,
    };
    let wav = AudioSpecWAV::load_wav(sound_path.clone()).expect("Could not load wav file");

    return Ok(subsystem
        .open_playback(None, &desired_spec, |spec| Sound {
            data: wav_spec(&wav, spec).convert(wav.buffer().to_vec())
                .into_iter()
                .map(|x| (x, volume))
                .collect::<Vec<(u8,f32)>>(),
            pos: pos,
        })
        .unwrap());
}

#[allow(dead_code)]
#[test]
pub fn loop_test() -> Result<()> {

    let (mut loop_count, loop_limit) = (0, 2);

    dbg!(format!("{:?}",&sdl2::audio::drivers().collect::<Vec<&str>>()));

    println!("Loading...");
    let sdl_context = sdl2::init().expect("Unable to initialize SDL2");
    println!("SDL2 initialized.");

    let audio_subsystem = sdl_context
        .audio()
        .expect("Unable to initialize audio subsystem");
    println!("Audio subsystem initialized.");

    /*
    let wav = Cow::from(Path::new("Ominous.wav"));
    let mut device = generate_sound(&audio_subsystem, &wav, 0.5, 0).unwrap();

    device.resume();

    
    loop {
        device.lock().fade_percent(0.05, 1.0);
        if device.lock().pos() >= 1154609 * 2 {
            device.lock().restart();
            device.lock().set_volume(0.5);
            loop_count += 1;
            if loop_count >= loop_limit {
                break;
            }
        }
    }

    let wav = Cow::from(Path::new("Mysterious_Cyborg.wav"));
    device = generate_sound(&audio_subsystem, &wav, 0.3, 0).unwrap();

    device.resume();
    loop_count = 0;
    loop {
        device.lock().fade_percent(0.02, 1.0);
        if device.lock().pos() >= (44100 as f64 * 21.33) as usize * 2 {
            device.lock().restart();
            device.lock().set_volume(0.5);
            loop_count += 1;
            if loop_count >= loop_limit {
                break;
            }
        }
    }
    */


    let path1 = Cow::from(Path::new("Purpose_prelude.wav"));
    let path2 = Cow::from(Path::new("Purpose_noprelude.wav"));
    let path3 = Cow::from(Path::new("Epic_Theme.wav"));
    let path4 = Cow::from(Path::new("Mysterious_Cyborg.wav"));
    let path5 = Cow::from(Path::new("Ominous.wav"));

    

    let mut skip = false;
    match std::env::args().collect::<Vec<String>>().iter().nth(1) {
        Some(x) => { println!("Argument received. Changing test behavior.");
                     skip = true; },
        _ => {},
    }

    if skip {

        
        let mut queue_device = generate_sound(&audio_subsystem, &path4, 0.0, 0).unwrap();
        for _ in 0..1 {
            queue_device.lock().queue(&path4, 0.0);
        }

        queue_device.resume();
        
        let audio_max_pos = queue_device.lock().len() - 1;
        while queue_device.lock().pos() < audio_max_pos {
            use std::io::Write;
            queue_device.lock().fade_percent(10.0, 0.7);
            print!("\x1b[s\x1b[2K Current volume (1.0 = max/normal): {:.4}\x1b[u", queue_device.lock().volume());
            std::io::stdout().flush();
            //queue_device.lock().fade_in(0.08, 0.7);
            sleep(Duration::from_millis(99));
        }
        

    }

    
    Ok(())
}
