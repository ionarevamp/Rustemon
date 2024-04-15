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

mod animation_data;
mod extras;
mod mons;
mod moves;
mod natures;
mod types;

use crate::extras::*;
use crate::mons::*;
use crate::moves::{Effect, PokeMove, Target};
use crate::natures::Nature;
use crate::types::*;

use crate::Color::Rgb;

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
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, BorderType, List, ListItem, Tabs, Widget},
    Terminal,
};

#[allow(unused_imports)]
use crossterm::{
    cursor::{
        DisableBlinking, EnableBlinking, Hide, MoveDown, MoveTo, MoveUp, RestorePosition,
        SavePosition, Show,
    },
    event::{
        self, poll, read, DisableMouseCapture, EnableMouseCapture, Event,
        KeyCode::{self, *},
        KeyEvent, KeyEventKind, KeyModifiers,
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

#[derive(Debug, Clone)]
struct Options {
    frame_color: Color,
    frame_style: BorderType,
    text_speed: u8,
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

struct Player<'a> {
    name: String,
    party: Party<'a>,
}

#[derive(Clone, Debug)]
struct Monster<'a> {
    monster_id: Pokemon,
    battle_type: [BattleType; 2],
    iv: [u8; 6],
    ev: [u8; 6],
    stats: [u8; 6],
    curstats: [u16; 6],
    crit_level: u8,
    accuracy: f64,
    lvl: u8,
    status: Effect<'a>,
    moves: &'static [PokeMove],
    owned: bool,
    affection: i32,
    nature: Nature,
    name: String,
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
        let mut mon = self.borrow_mut();
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

pub fn main_menu(
    player_data: &mut Player,
    audio_subsystem: &AudioSubsystem,
    beep_wav: &Cow<'static, Path>,
    select_wav: &Cow<'static, Path>,
) -> io::Result<()> {

    // READ OPTIONS FROM FILE

    let entries = vec!["New Game", "Continue", "Options"];

    let mut in_menu = true;
    let mut menu_select = 0;
    let mut item_selected = false;
    let mut item_count = 0;

    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut options_state = Options {
        frame_color: Color::White,
        frame_style: BorderType::Plain,
        text_speed: 0,
    };

    //execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let mut beep_sounds = Vec::new();
    let mut select_sound = extras::generate_sound(&audio_subsystem, &select_wav, 1.5, 0)
        .unwrap();
    for _ in 0..2 {
        beep_sounds.push(
            extras::generate_sound(&audio_subsystem, &beep_wav, 2.0, 0)
                .unwrap(),
        );
    }

    while in_menu {
        if poll(Duration::from_millis(100)).expect("IO error in menu.") {
            let swap = beep_sounds.remove(0);
            beep_sounds.push(swap);

            for j in 1..beep_sounds.len() {
                //beep_sounds[j].lock().set_volume(0.8);
                beep_sounds[j].pause();
            }
            let len = beep_sounds.len() - 1;
            beep_sounds[len].lock().restart();

            std::hint::spin_loop();
            match read().expect("Error reading event.") {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Up => {
                            //beep_sounds[0].lock().set_volume(2.0);
                            beep_sounds[0].resume();
                            if menu_select > 0 {
                                menu_select -= 1;
                            } else {
                                menu_select = 2;
                            }
                        }
                        KeyCode::Down => {
                            //beep_sounds[0].lock().set_volume(2.0);
                            beep_sounds[0].resume();
                            if menu_select < 2 {
                                menu_select += 1;
                            } else {
                                menu_select = 0;
                            }
                        }
                        KeyCode::Enter => {
                            select_sound.lock().restart();
                            select_sound.resume();
                            match menu_select {
                                0 => unimplemented!(), //New Game
                                1 => unimplemented!(), //Continue
                                2 => {
                                    options_state = options_menu(
                                        &audio_subsystem,
                                        &mut beep_sounds,
                                        &mut select_sound,
                                        options_state.clone(),
                                    ); //Options
                                }
                                _ => panic!("`menu_select` outside of expected parameters\n"),
                            }
                        }
                        KeyCode::End | Char('c') | KeyCode::Esc => in_menu = false,
                        _ => print!(""),
                    }
                }
                _ => print!(""), // add some kind of feedback (preferably a short sound effect)
            }
        } else {
            // Render menu based on which item is selected
            let menu = List::new(
                entries
                    .clone()
                    .into_iter()
                    .map(|item| {
                        if menu_select == item_count {
                            item_count += 1;
                            ListItem::new(">".to_owned() + item)
                                .style(Style::default().fg(Color::Gray))
                        } else {
                            item_count += 1;
                            ListItem::new(" ".to_owned() + item)
                                .style(Style::default().fg(Color::White))
                        }
                    })
                    .collect::<Vec<_>>(),
            );
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(50)].as_ref())
                    .split(Rect {
                        x: 0,
                        y: 0,
                        width: f.size().width,
                        height: (entries.len() + 3 + 2) as u16, // plus 3 to account for margins, then another 2 to add whitespace
                    });

                let tabs = menu.block(
                    Block::default()
                        .title("MENU (Press 'c'/End to exit)")
                        .title_bottom("--ENTER key to select--")
                        .borders(Borders::ALL)
                        .border_type(options_state.frame_style.clone())
                        .style(Style::default().fg(options_state.frame_color.clone())),
                );

                f.render_widget(tabs, chunks[0]);
            })?;
        }

        item_count = 0;
    }

    //execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    // SAVE OPTIONS TO FILE

    Ok(())
}

pub fn options_menu(
    audio: &AudioSubsystem,
    beep_sounds: &mut Vec<AudioDevice<Sound>>,
    select_sound: &mut AudioDevice<Sound>,
    options_state: Options,
) -> Options {
    //TODO: Write option settings to file

    execute!(io::stdout(), EnterAlternateScreen).unwrap();

    let mut options = options_state.clone();
    
    let borders = vec![
        BorderType::Plain,
        BorderType::Rounded,
        BorderType::Double,
        BorderType::Thick,
        BorderType::QuadrantInside,
        BorderType::QuadrantOutside,
    ];

    let colors = vec![
        Color::Black,
        Color::DarkGray,
        Color::Red,
        Color::LightRed,
        Color::Green,
        Color::LightGreen,
        Color::Yellow,
        Color::LightYellow,
        Color::Blue,
        Color::LightBlue,
        Color::Magenta,
        Color::LightMagenta,
        Color::Cyan,
        Color::LightCyan,
        Color::White,
        Color::Gray,
    ];

    let custom_colors = vec![
        Rgb(240, 190, 190), //Pink
    ];

    let entries = vec!["Frame color", "Frame style", "Text speed"];

    let mut in_menu = true;
    let mut confirm = false;

    let mut menu_select = 0;
    let mut item_selected = false;
    let mut item_count = 0;

    let mut color_index = 14;
    let mut border_index = 0;

    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    execute!(io::stdout(), EnterAlternateScreen).unwrap();

    while in_menu {
        if poll(Duration::from_millis(100)).expect("IO error in menu.") {
            let swap = beep_sounds.remove(0);
            beep_sounds.push(swap);

            for j in 1..beep_sounds.len() {
                //beep_sounds[j].lock().set_volume(0.8);
                beep_sounds[j].pause();
            }
            let len = beep_sounds.len() - 1;
            beep_sounds[len].lock().restart();

            std::hint::spin_loop();
            match read() {
                Ok(Event::Key(event)) => {
                    match event.code {
                        KeyCode::Up => {
                            //beep_sounds[0].lock().set_volume(2.0);
                            beep_sounds[0].resume();
                            if menu_select > 0 {
                                menu_select -= 1;
                            } else {
                                menu_select = 2;
                            }
                        }
                        KeyCode::Down => {
                            //beep_sounds[0].lock().set_volume(2.0);
                            beep_sounds[0].resume();
                            if menu_select < entries.len() - 1 {
                                menu_select += 1;
                            } else {
                                menu_select = 0;
                            }
                        }
                        KeyCode::Left => match menu_select {
                            0 => {
                                if color_index == 0 {
                                    color_index = colors.len() - 1;
                                } else {
                                    color_index -= 1;
                                }
                            }
                            1 => {
                                if border_index == 0 {
                                    border_index = borders.len() - 1;
                                } else { border_index -= 1; }
                            }
                            2 => {
                                if options.text_speed > 0 {
                                    options.text_speed -= 1;
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Right => match menu_select {
                            0 => {
                                if color_index == colors.len() - 1 {
                                    color_index = 0;
                                } else {
                                    color_index += 1;
                                }
                            }
                            1 => {
                                if border_index == borders.len() - 1 {
                                    border_index = 0;
                                } else { border_index += 1; }
                            }
                            2 => {
                                if options.text_speed < 5 {
                                    options.text_speed += 1;
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Enter => {
                            select_sound.lock().restart();
                            select_sound.resume();
                            sleep(Duration::from_millis(500));
                            confirm = true;
                            in_menu = false;
                        }
                        KeyCode::End | Char('c') | KeyCode::Esc => {
                            options = options_state.clone();
                            in_menu = false;
                        }
                        _ => {}
                    }
                }
                _ => {} // add some kind of feedback (preferably a short sound effect)
            }
        }
        // Render menu based on which item is selected
        let menu = List::new(
            entries
                .clone()
                .into_iter()
                .map(|item| {
                    let state = match item_count {
                        0 => format!("{:?}", colors[color_index]),
                        1 => format!("{:?}", borders[border_index]),
                        2 => format!("{:?}", options.text_speed),
                        _ => "Error: `item_count` out of bounds.".to_string(),
                    };

                    if menu_select == item_count {
                        item_count += 1;
                        ListItem::new(">".to_owned() + item + " : " + state.as_str())
                            .style(Style::default().fg(Color::Gray))
                    } else {
                        item_count += 1;
                        ListItem::new(" ".to_owned() + item + " : " + state.as_str())
                            .style(Style::default().fg(Color::White))
                    }
                })
                .collect::<Vec<_>>(),
        );
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(Rect {
                        x: 0,
                        y: 0,
                        width: f.size().width,
                        height: (entries.len() + 3 + 2) as u16, // plus 3 to account for margins, then another 2 to add whitespace
                    });

                let tabs = menu.block(
                    Block::default()
                        .title("OPTIONS (Press 'c'/End to go back)")
                        .title_bottom("--ENTER key to confirm--")
                        .borders(Borders::ALL)
                        .border_type(borders[border_index])
                        .style(Style::default().fg(colors[color_index])),
                );

                f.render_widget(tabs, chunks[0]);
            })
            .unwrap();

        item_count = 0;
    }

    execute!(io::stdout(), LeaveAlternateScreen).unwrap();

    if confirm {
        options = Options {
            frame_color: colors[color_index],
            frame_style: borders[border_index],
            text_speed: options.text_speed,
        };
    }
    options
}





// COPYRIGHT MESSAGE
const COPYRIGHT_MESSAGE: [&str; 4] = [
    "Inspired by and adapted from ",
    "the works of GAME FREAK Inc. , by whom ",
    "copyright (\u{00a9}) is reserved. This project is ",
    "published as an open-source, non-profit endeavor.",
];

fn main() -> Result<(), io::Error> {
    //INITIALIZE
    println!("Loading...");
    let sdl_context = sdl2::init().expect("Unable to initialize SDL2");
    println!("SDL2 initialized.");
    let audio_subsystem = sdl_context
        .audio()
        .expect("Unable to initialize audio subsystem");
    println!("Audio subsystem initialized.");
    
    let beep_wav = Cow::from(Path::new("shortbeep.wav"));
    let select_wav = Cow::from(Path::new("select.wav"));

    // If the first sound is able to be created, then the rest should be fine up to 16 total
    // devices
    //    let mut beep_sound = extras::generate_sound(&audio_subsystem, &beep_wav, 0.7, 0)
    //        .expect("Unable to create device.").0;

    let mut stdout = io::stdout();
    execute!(stdout, Hide);

    // KEEP THIS HERE FOR LEGAL REASONS!!!
    // ( Copyright message, stylized )

    // Make some room
    for _ in 0..terminal::size().expect("Error getting terminal size").1 {
        println!();
    }
    execute!(stdout, MoveTo(0, 19), Clear(ClearType::All));
    
    enable_raw_mode()?;

    'message: { // keeps its own scope to properly load and drop sound data
        
        let message_wav = Cow::from(Path::new("Rustemon_Intro_Sound.wav"));
        // load sound ahead of time
        let mut message_sound =
            extras::generate_sound(&audio_subsystem, &message_wav, 3.5, 0)
                .unwrap();

        for i in 0..100 {
            let mut i = i;

            if poll(Duration::from_millis(20))? {
                match read()? {
                    _ => break,
                } // if user does anything, skip the rest
            } else {
                if i == 50 {

                    message_sound.resume();
                    if poll(Duration::from_millis(3000))? {
                        execute!(stdout, Clear(ClearType::All), MoveTo(0, 22));
                        match read()? {
                            _ => {
                                break 'message;
                            } // allow interrupt during sound effect
                        }
                    }
                } else if i > 50 {
                    i = 100 - i;
                }
                let shade = format!("\x1B[38;2;{0};{0};{0}m", (i as f32 * 5.222) as usize);
                print!("{}", &shade);
                for line_num in 0..4 {
                    execute!(stdout, MoveTo(0, 19 + line_num))?;
                    print!("{}", &COPYRIGHT_MESSAGE[line_num as usize]);
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

        let mut device = generate_sound(&audio_subsystem, &Cow::from(Path::new("Epic_Theme.wav")), 0.8, 0).unwrap();
        device.resume();

        let song_length = device.lock().len();

        let mut playing_intro = true;
        let mut intro_state = 0;
        let animation_speed = 100;
        let intro_ptr = &animation_data::intro::FRAME_DATA;
        let intro_start = std::time::Instant::now();

        while playing_intro {
            if intro_start.elapsed().as_millis() as usize >= (animation_speed * intro_state) {
                let frame = intro_ptr[intro_state as usize]
                    .lines()
                    .collect::<Vec<&str>>();
                for i in 0..frame.len() {
                    print!("\x1B[{};1H", 20 + i);
                    let _ = io::stdout().write(frame[i].as_bytes());
                }

                let _ = io::stdout().flush();

                intro_state += 1;

                if skip {
                    if device.lock().volume() <= 0.01f32 {
                        playing_intro = false;
                    } else {
                        device.lock().fade_out(0.08, 0.0);
                    }
                }
            }

            if poll(Duration::from_millis(0))? {
                match read()? {
                    _ => {
                        skip = true;
                    } //Any event skips
                }
            }

            // DEVELOPER -- don't continue animation if there are no more frames
            if intro_state >= intro_ptr.len() {
                intro_state = intro_ptr.len() - 1;
            }

            // DEV CHECK -- Second evaluation checks if song is done playing
            if intro_state >= intro_ptr.len().try_into().unwrap()
                && device.lock().pos() >= song_length
            {
                playing_intro = false;
            }
        }

        execute!(&stdout, Clear(ClearType::All));

        // Intro sequence scope ends here
    }

    // MAIN LOGIC

    //TODO: Load saved player data if present
    //Initialize player data
    let mut player = Player {
        name: "DEV".to_string(),
        party: Party::new(),
    };
    player.party[0] = Monster::new(150, 0u8);

    // Menu

    {
        main_menu(&mut player, &audio_subsystem, &beep_wav, &select_wav);
        disable_raw_mode();
    }

    disable_raw_mode()?;

    execute!(io::stdout(), Show);

    // TESTING
    //println!();
    //dbg!(&player.party[0]);
    //println!("{:?}", BattleType::Fire.weaknesses());
    //println!("{}", format!("{:?}", BattleType::Fire));
    //println!("{:?}, {:?}", Pokemon[0], Pokemon[0].name());

    Ok(())
}
