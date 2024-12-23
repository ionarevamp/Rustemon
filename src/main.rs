extern crate rand;
extern crate sdl2;

#[allow(unused_imports)]
use std::cell::RefCell;
//use std::sync::Arc;
use std::borrow::BorrowMut;

// define descriptions, stat generation, etc.
// consider controller support --
//	 - Bluetooth resource: https://dev.to/lcsfelix/using-rust-blurz-to-read-from-a-ble-device-gmb
//	 - Capturing other input likely requires something like libx11 (window manager) as
//		terminals do not support keydown/keyup events by themselves
// implement capturing of general input for real-time movement and menu management.
//
//
// REGARDING TESTING/RUNNING: When testing on windows, running in a separate shell
// via `start` seems to send an input to the window, implicitly skipping
// the introductory message.

mod animation_data;
mod extras;
mod mons;
mod moves;
mod natures;
mod types;
mod menu;
mod interface;
mod saves;
mod area;
mod tile_to_ascii;

use crate::extras::*;
use crate::mons::*;
use crate::moves::{Effect, PokeMove, Target};
use crate::natures::Nature;
use crate::types::*;
use crate::interface::{dialog_box};
use crate::saves::*;
use crate::area::*;

use rand::prelude::*;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::*;
use std::thread::{self, sleep, JoinHandle};

use std::{
    io,
    time::{Duration, Instant},
};

#[allow(unused_imports)]
use sdl2::audio::{
    AudioCVT, AudioCallback, AudioDevice, AudioFormatNum, AudioSpec, AudioSpecDesired, AudioSpecWAV,
};
use sdl2::AudioSubsystem;
use sdl2::mixer::{self, *};

use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
use ratatui::{
    symbols::border::Set,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color::{self, *}, Style},
    text::Span,
    widgets::{Block, Borders, BorderType, List, ListItem, Tabs, Widget},
    Terminal,
};

#[allow(unused_imports)]
use crossterm::{
    style::{
        Color::{self as CrosstermColor, *},
        SetForegroundColor,
        SetBackgroundColor
    },
    cursor::{
        DisableBlinking, EnableBlinking, Hide, MoveDown, MoveTo, MoveUp, RestorePosition,
        SavePosition, Show,
    },
    event::{
        self, poll, read, DisableMouseCapture, EnableMouseCapture, Event,
        KeyCode::{self, *},
        KeyEvent, KeyEventKind, KeyModifiers,
        KeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags
    },
    execute,
    terminal::{
        self,
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use std::result::Result;

pub fn get_term_size() -> (u16, u16) {
    let termsize = terminal::size().unwrap();

    let width = termsize.0;
    let height = termsize.1;

    (width.clone(), height.clone())
}



#[derive(Debug, Clone)]
struct Party<'a> {
    mons: [RefCell<Monster<'a>>; 6],
    owned: bool,
}

impl<'a> Party<'a> {
    fn new() -> Party<'a> {
        Party {
            mons: [
                // Initialized this way because Copy is not implemented
                Monster::new(0, 0),
                Monster::new(0, 0),
                Monster::new(0, 0),
                Monster::new(0, 0),
                Monster::new(0, 0),
                Monster::new(0, 0),
            ],
            owned: false,
        }
    }

    fn with_mons(mons: &'a [RefCell<Monster<'a>>]) -> Party<'a> {
        let new_party = Party::new();
        for i in 0..mons.len() {
            *new_party.mons[i].borrow_mut() = mons[i].borrow().clone();
        }
        new_party
    }
}

impl<'a> std::ops::Index<usize> for Party<'a> {
    type Output = RefCell<Monster<'a>>;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.mons[idx]
    }
}

impl<'a> std::ops::IndexMut<usize> for Party<'a> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.mons[idx]
    }
}

// TODO: move this into items.rs to simplify main file
// mod items;
enum Item {
    Pokeball,
}

// TODO: maybe move this into its own file as well
// mod stance;
enum Stance {
    Default,
}

enum CharacterState {
    Running,
    Walking,
    Standing,
    Biking(u8), // which bike
    Surfing(u16), // which pokemon?
    Holding(Item, Stance), // self-explanatory
}

struct Player<'a> {
    name: String,
    party: Party<'a>,
    pub area: Area,
    pub position: (u16, u16, u16), // x, y, z
    pub animation_state: u16,
    pub movement_state: CharacterState,
}

impl<'a> Player<'a> {
    pub fn move_left(&mut self) {
        if self.position.0 > 0 {
            self.position.0 -= 1;
        }
    }
    pub fn move_right(&mut self) {
        if self.position.0 < u16::MAX {
            self.position.0 += 1;
        }
    }
    pub fn move_up(&mut self) {
        if self.position.1 < u16::MAX {
            self.position.1 += 1;
        }
    }
    pub fn move_down(&mut self) {
        if self.position.1 > 0 {
            self.position.1 -= 1;
        }
    }
    pub fn ascend(&mut self) {
        if self.position.2 < u16::MAX {
            self.position.2 += 1;
        }
    }
    pub fn descend(&mut self) {
        if self.position.2 > 0 {
            self.position.2 -= 1;
        }
    }

    pub fn swap_party(&mut self, first: usize, second: usize) {
        let swap = self.party[first].clone();
        self.party[first] = self.party[second].clone();
        self.party[second] = swap;
    }
}

#[derive(Clone, Debug)]
struct Monster<'a> {
    monster_id: Pokemon,
    battle_type: [BattleType; 2],
    iv: [u8; 6],
    ev: [u8; 6],
    pub stats: [u8; 6],
    pub curstats: [u16; 6],
    pub crit_level: u8,
    pub accuracy: f64,
    pub lvl: u8,
    pub status: Effect<'a>,
    pub moves: &'static [PokeMove],
    owned: bool,
    pub affection: i32,
    nature: Nature,
    pub name: String,
    overworld: Option<(u8, u16, u16, u16, u16)>, // direction, x, y, z, animation state
}

impl<'a> Monster<'a> {
    pub fn new(dex: usize, level: u8) -> RefCell<Monster<'a>> {
        //Implemented as a refcell
        //because multiple sources may
        //end up needing to alter a
        //monster in a single pass
        RefCell::new(Monster {
            monster_id: Pokemon[dex],
            battle_type: Pokemon[dex].types(),
            iv: [0; 6],
            ev: [0; 6],
            stats: Pokemon[dex].base_stats(),
            curstats: [0; 6],
            accuracy: 100.0f64,
            crit_level: 0u8,
            lvl: level,
            status: Effect::Null,
            moves: &[PokeMove::Null; 4],
            owned: false,
            affection: 0i32,
            nature: Nature::new(),
            name: Pokemon[dex].name(),
            overworld: None
        })
    }

    pub fn nature_modify(&mut self) -> [(usize, f64); 2] {
        let (modadd, modsub) = (0.2f64, -0.2f64);
        match self.nature {
            Nature::Adamant => [(1, modadd), (3, modsub)],
            Nature::Bashful => [(3, modadd), (3, modsub)],
            Nature::Bold => [(2, modadd), (1, modsub)],
            Nature::Brave => [(1, modadd), (5, modsub)],
            Nature::Calm => [(4, modadd), (1, modsub)],
            Nature::Careful => [(4, modadd), (2, modsub)],
            Nature::Docile => [(2, modadd), (2, modsub)],
            Nature::Gentle => [(4, modadd), (2, modsub)],
            Nature::Hardy => [(1, modadd), (2, modsub)],
            Nature::Hasty => [(5, modadd), (2, modsub)],
            Nature::Impish => [(2, modadd), (3, modsub)],
            Nature::Jolly => [(5, modadd), (2, modsub)],
            Nature::Lax => [(3, modadd), (4, modsub)],
            Nature::Lonely => [(1, modadd), (2, modsub)],
            Nature::Mild => [(3, modadd), (2, modsub)],
            Nature::Modest => [(3, modadd), (2, modsub)],
            Nature::Naive => [(5, modadd), (4, modsub)],
            Nature::Naughty => [(1, modadd), (4, modsub)],
            Nature::Quiet => [(3, modadd), (5, modsub)],
            Nature::Quirky => [(4, modadd), (5, modsub)],
            Nature::Rash => [(3, modadd), (4, modsub)],
            Nature::Relaxed => [(2, modadd), (5, modsub)],
            Nature::Sassy => [(4, modadd), (5, modsub)],
            Nature::Serious => [(5, modadd), (5, modsub)],
            Nature::Timid => [(5, modadd), (1, modsub)],
        }
    }

    /* Generation follows formula:
     *
     * HP = floor(0.01 x (2 x Base + IV + floor(0.25 x EV)) x Level) + Level + 10
     * Other Stats = (floor(0.01 x (2 x Base + IV + floor(0.25 x EV)) x Level) + 5) x Nature
     */
    pub fn init_stats(&mut self) {
        let mon = self.borrow_mut();
        mon.accuracy = 100.0f64;
        for i in 0..mon.stats.len() {
            let base = mon.stats[i] as f64;
            let iv = mon.iv[i] as f64;
            let ev = mon.ev[i] as f64;
            let level = mon.lvl as f64;
            if i == 0 {
                // health

                mon.curstats[i] = (0.01 * ((2.0 * (base + iv) + (0.25 * ev).floor()) * level)
                    + level
                    + 10.0) as u16;
            } else {
                // other stats

                let mut modifier: f64 = 1.0;
                let nature_mod = mon.nature_modify();
                if i == nature_mod[0].0 {
                    modifier += nature_mod[0].1;
                }
                if i == nature_mod[1].0 {
                    modifier += nature_mod[1].1;
                }
                mon.curstats[i] = ((0.01 * ((2.0 * (base + iv) + (0.25 * ev).floor()) * level)
                    + 5.0)
                    * modifier) as u16;
            }
        }
    }

    pub fn health(&mut self) -> &mut u16 {
        self.curstats[0].borrow_mut()
    }
    pub fn attack(&mut self) -> &mut u16 {
        self.curstats[1].borrow_mut()
    }
    pub fn defense(&mut self) -> &mut u16 {
        self.curstats[2].borrow_mut()
    }
    pub fn spattack(&mut self) -> &mut u16 {
        self.curstats[3].borrow_mut()
    }
    pub fn spdefense(&mut self) -> &mut u16 {
        self.curstats[4].borrow_mut()
    }
    pub fn speed(&mut self) -> &mut u16 {
        self.curstats[5].borrow_mut()
    }
}


// DIMENSIONS (PROGRAM NEEDS AT LEAST 80x24 characters)
pub const TERM_WIDTH: usize = 80;
pub const TERM_HEIGHT: usize = 24;
pub const RIGHT_POINTING_TRIANGLE: &str = "▶";

// COPYRIGHT MESSAGE
const COPYRIGHT_MESSAGE: [&str; 4] = [
    "Inspired by and adapted from ",
    "the works of GAME FREAK Inc. , by whom ",
    "copyright (\u{00a9}) is reserved. This project is ",
    "published as an open-source, non-profit endeavor.",
];

fn main() -> Result<(), io::Error> {
    
    let mut stdout = io::stdout();

    //INITIALIZE
    println!("Loading...");
    let sdl_context = sdl2::init().expect("Unable to initialize SDL2");
    println!("SDL2 initialized.");
    let audio_subsystem = sdl_context
        .audio()
        .expect("Unable to initialize audio subsystem");
    println!("Audio subsystem initialized.");

    // load options if present
    let mut global_options = saves::read_save().unwrap();
    
    /*
    // TEST
        execute!(&stdout, Clear(ClearType::All), MoveTo(1,1));
        dialog_box(&mut stdout, &global_options, "This is a test dialog box, with a looooooooooooooooooooooooooong sentence!! ... And this is a second sentence. Yeaaaaaaaah.", "Yo mama");
        sleep(Duration::from_millis(1000));

        execute!(&stdout, Clear(ClearType::All), MoveTo(1,1));
        dialog_box(&mut stdout, &global_options, "LONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORDLONGWORD", "DEV");
        sleep(Duration::from_millis(1000));
    //
    */

    let beep_wav = Cow::from(Path::new("shortbeep.wav"));
    let select_wav = Cow::from(Path::new("select.wav"));

    // If the first sound is able to be created, then the rest should be fine up to 16 total
    // devices
    //    let mut beep_sound = extras::generate_sound(&audio_subsystem, &beep_wav, 0.7, 0)
    //        .expect("Unable to create device.").0;

    execute!(&stdout, Hide)?;

    // KEEP THIS HERE FOR LEGAL REASONS!!!
    // ( Copyright message, stylized )

    // Make some room
    for _ in 0..terminal::size().expect("Error getting terminal size").1 {
        println!();
    }
    execute!(&stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    
    enable_raw_mode()?;

    { // keeps its own scope to properly load and drop sound data
        

        let message_wav = Cow::from(Path::new("Rustemon_Intro_Sound.wav"));
        // load sound ahead of time
        let message_sound =
            extras::generate_sound(&audio_subsystem, &message_wav, 3.5 * global_options.master_volume, 0)
                .unwrap();
        
        let timeout: u64 = 17; //60 fps
        let pause_time = 3000;
        let limit = (5000 - pause_time)/timeout;
        let mut msg_pause: u64 = 0;
        let start_time = Instant::now();

        let mut in_message = true;
        let mut step = 0;

        while in_message {
            if (start_time.elapsed().as_millis() as u64) >= (timeout * step) + msg_pause {
                
                let mut i = step;

                #[allow(clippy::comparison_chain)]
                if step == (limit/2) {
                    
                    msg_pause += pause_time;

                    message_sound.resume();
                    if poll(Duration::from_millis(0))? {
                        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                            in_message = false;
                            // allow interrupt during sound effect
                    }

                } else if step > (limit/2) {
                    i = limit - step;
                }

                let shade = (i as f32 * (limit as f32/ 2.0)/ 17.0) as u8;
                execute!(stdout, SetForegroundColor(CrosstermColor::Rgb { r: shade, g: shade, b: shade }))?;
                for line_num in 0..4 {
                    execute!(stdout, MoveTo(0, line_num))?;
                    print!("{}", &COPYRIGHT_MESSAGE[line_num as usize]);
                }
                
                stdout.flush()?;
                if step >= limit {
                    in_message = false;
                }

                step += 1;

            } else {
                #[allow(clippy::collapsible_if)]
                if poll(Duration::from_millis(0))? {
                    if let Ok(Event::Key(event)) = read() {
                        if event.kind == KeyEventKind::Press {
                            in_message = false;
                        } // any keypress skips
                    }
                }
            }
        }

    }

    execute!(&stdout, Clear(ClearType::All))?;

    //Reset text style
    print!("\x1B[0m");

    // Play Intro Sequence ( TODO: implement skip functionality )

    // has its own scope to avoid taking up SDL2 audio devices, and to ensure proper dropping of
    // audio stream
    {
        let mut skip = false;

        let mut device = generate_sound(&audio_subsystem, &Cow::from(Path::new("Epic_Theme.wav")), 0.8 * global_options.master_volume, 0).unwrap();
        device.resume();

        let song_length = device.lock().len();

        let mut playing_intro = true;
        let mut intro_state = 0;
        let animation_speed = 100; // Just for now; goal is to have a 60fps animation
        let intro_ptr = &animation_data::intro::FRAME_DATA;
        let intro_start = std::time::Instant::now();

        while playing_intro {
            if intro_start.elapsed().as_millis() as usize >= (animation_speed * intro_state) {
                let frame = intro_ptr[intro_state]
                    .lines()
                    .collect::<Vec<&str>>();
                for (i, cell) in frame.iter().enumerate() {
                    execute!(stdout, MoveTo(0,0 + i as u16))?;
                    let _ = io::stdout().write(cell.as_bytes());
                    stdout.flush()?;
                }

                let _ = io::stdout().flush();

                intro_state += 1;

                if skip {
                    if device.lock().volume() <= 0.015f32 {
                        playing_intro = false;
                    } else {
                        device.lock().fade_out(0.08, 0.0);
                    }
                }
            }

            if poll(Duration::from_millis(0))? {
                if let Ok(Event::Key(event)) = read() {
                    if event.kind == KeyEventKind::Press {
                        skip = true;
                    } // any keypress skips
                }
            }

            // DEVELOPER -- don't continue animation if there are no more frames
            if intro_state >= intro_ptr.len() {
                intro_state = intro_ptr.len() - 1;
            }

            // DEV CHECK -- Second evaluation checks if song is done playing
            if intro_state >= intro_ptr.len() - 1
                || device.lock().pos() >= song_length
            {
                playing_intro = false;
            }
        }

        let _ = execute!(&stdout, Clear(ClearType::All));

        // Intro sequence scope ends here
    }

    // MAIN LOGIC

    //Initialize player data
    let mut player = Player {
        name: "DEV".to_string(),
        party: Party::new(),
        area: Area::TestZone,
        animation_state: 0,
        position: (10, 10, 0),
        movement_state: CharacterState::Standing,
    };
    player.party[0] = Monster::new(150, 0u8);

    

    // Menu

    {
        global_options = menu::main_menu(&mut player, &audio_subsystem, &beep_wav, &select_wav, global_options).unwrap();
        disable_raw_mode()?;
    }


    disable_raw_mode()?;
    execute!(io::stdout(), Show)?;

    // TESTING
    //println!();
    //dbg!(&player.party[0]);
    //println!("{:?}", BattleType::Fire.weaknesses());
    //println!("{}", format!("{:?}", BattleType::Fire));
    //println!("{:?}, {:?}", Pokemon[0], Pokemon[0].name());

    Ok(())
}
