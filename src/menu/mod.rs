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
        KeyEvent, KeyEventKind, KeyModifiers, KeyEventState,
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

use crate::Player;
use crate::extras;
use crate::Sound;
use crate::saves::*;
use crate::RIGHT_POINTING_TRIANGLE;
use crate::interface::*;

pub const CUSTOM_BORDERS: [Set; 2] = 
[
    Set {
        top_left: r"\",
        top_right: r"/",
        bottom_left: r"/",
        bottom_right: r"\",
        vertical_left: r"│",
        vertical_right: r"│",
        horizontal_top: r"─",
        horizontal_bottom: r"─",
    },
    Set {
        top_left: r"o",
        top_right: r"o",
        bottom_left: r"°",
        bottom_right: r"°",
        vertical_left: r"│",
        vertical_right: r"│",
        horizontal_top: r"─",
        horizontal_bottom: r"─", 
    }
];

pub const BORDERS: [ratatui::symbols::border::Set; 8] = [

        BorderType::Plain.to_border_set(),
        BorderType::Rounded.to_border_set(),
        BorderType::Double.to_border_set(),
        BorderType::Thick.to_border_set(),
        BorderType::QuadrantInside.to_border_set(),
        BorderType::QuadrantOutside.to_border_set(),
        CUSTOM_BORDERS[0],
        CUSTOM_BORDERS[1],

];

pub const BORDER_NAMES: [&'static str; 8] = [
    "Plain",
    "Rounded",
    "Double",
    "Thick",
    "QuadrantInside",
    "QuadrantOutside",
    "Pokéborder 1",
    "Pokéborder 2"
];


pub const CUSTOM_COLORS: [Color; 19] = [
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
    Rgb(240, 176, 206), //Light Pink
    Rgb(255, 109, 178), //Pink
    Rgb(0, 255, 142), //Sea Green
];

pub const COLOR_NAMES: [&'static str; 19] = [
    "Black",
    "Dark Gray",
    "Red",
    "Light Red",
    "Green",
    "Light Green",
    "Yellow",
    "Light Yellow",
    "Blue",
    "Light Blue",
    "Magenta",
    "Light Magenta",
    "Cyan",
    "Light Cyan",
    "White",
    "Gray",
    "Light Pink",
    "Pink",
    "Sea Green"
];

pub const BORDER_COLORS: [Color; 19] = [
    CUSTOM_COLORS[0],
    CUSTOM_COLORS[1],
    CUSTOM_COLORS[2],
    CUSTOM_COLORS[3],
    CUSTOM_COLORS[4],
    CUSTOM_COLORS[5],
    CUSTOM_COLORS[6],
    CUSTOM_COLORS[7],
    CUSTOM_COLORS[8],
    CUSTOM_COLORS[9],
    CUSTOM_COLORS[10],
    CUSTOM_COLORS[11],
    CUSTOM_COLORS[12],
    CUSTOM_COLORS[13],
    CUSTOM_COLORS[14],
    CUSTOM_COLORS[15],
    CUSTOM_COLORS[16],
    CUSTOM_COLORS[17],
    CUSTOM_COLORS[18]
];

#[derive(Debug, Clone)]
pub struct Slider {
    pub length: u16,
    pub position: f64,
}

impl Slider {
    pub fn new(length: u16, position: f64) -> Self {
        Self {
            length,
            position,
        }
    }
}

impl std::fmt::Display for Slider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        let position = self.position.clamp(0.0, 100.0);

        let mut placed_bar = false;

        for i in 0..=self.length {
            let portion = 100.0 / self.length as f64;
            let j = portion * i as f64;
            f.write_char(
                if j >= position && !placed_bar {
                    placed_bar = true;
                    print!("\x1b[s\x1b[100;1H portion = {}, j = {}                      \x1b[u", portion, j);
                    '|'
                } else {
                    '―'
                    //'='
                }
            )?;
        }

        Ok(())
        //write!(f, "{}", slider)

    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub frame_color_index: usize,
    pub frame_style_index: usize,
    pub text_speed: u8,
    pub master_volume: f32,
}

impl Options {
    pub fn default() -> Options {
        Options {
            frame_color_index: 14,
            frame_style_index: 0,
            text_speed: 3,
            master_volume: 1.0,
        }
    }

    pub fn frame_color_index(&self) -> usize {
        self.frame_color_index
    }
    pub fn frame_style_index(&self) -> usize {
        self.frame_style_index
    }
    pub fn text_speed(&self) -> u8 {
        self.text_speed
    }

    pub fn set_frame_color(&mut self, frame_color_index: usize) {
        self.frame_color_index = frame_color_index;
    }
    pub fn set_frame_style(&mut self, frame_style_index: usize) {
        self.frame_style_index = frame_style_index;
    }
    pub fn set_text_speed(&mut self, text_speed: u8) {
        self.text_speed = text_speed;
    }
}


// TODO: Refactor menu functions to use a helper function which generates a menu based on the
// number of items and an optional size parameter

use crate::extras::*;

pub fn main_menu(
    player_data: &mut Player,
    audio_subsystem: &AudioSubsystem,
    beep_wav: &Cow<'static, Path>,
    select_wav: &Cow<'static, Path>,
    current_options: Options,
) -> io::Result<Options> {

    let entries = vec!["New Game", "Continue", "Options", "Save Options"];

    let mut in_menu = true;
    let mut menu_select = 0;
    let mut item_count = 0;

    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut options_state = current_options;

    //execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    
    let (select_volume, beep_volume) = (1.5, 2.0);

    let mut beep_sounds = Vec::new();
    let mut select_sound = extras::generate_sound(audio_subsystem, select_wav, select_volume * options_state.master_volume, 0)
        .unwrap();
    for _ in 0..2 {
        beep_sounds.push(
            extras::generate_sound(audio_subsystem, beep_wav, beep_volume * options_state.master_volume, 0)
                .unwrap(),
        );
    }

    while in_menu {
        let input_wait = Instant::now();
        
        let mut event_list = vec![Event::Key(KeyEvent::new_with_kind_and_state(
            KeyCode::Null,
            KeyModifiers::NONE,
            KeyEventKind::Release,
            KeyEventState::NONE
        ))];


        while input_wait.elapsed().as_millis() <= 100 {            
            if poll(Duration::from_millis(1)).unwrap() {
                let next_event = read().unwrap();
                #[allow(clippy::single_match)]
                match next_event {
                    Event::Key(event) => {
                        if event.kind != KeyEventKind::Release && event.code != KeyCode::Null {
                            event_list.push(Event::Key(event));
                        }
                    },
                    _ => {},
                }
            }
        }
        if event_list.len() > 1 {
            for _ in 0..event_list.len()-1 {
                event_list.remove(0);
            }
        }
        
        #[allow(clippy::single_match)]
        match event_list[0] {
            Event::Key(event) => {
                if event.kind != KeyEventKind::Release && event.code != KeyCode::Null { // Only act on press or repeat
                    
                    
                    let swap = beep_sounds.remove(0);
                    beep_sounds.push(swap);

                    for j in 1..beep_sounds.len() {
                        //beep_sounds[j].lock().set_volume(0.8);
                        beep_sounds[j].pause();
                    }
                    let len = beep_sounds.len() - 1;
                    
                    if event.code != KeyCode::Enter {
                        beep_sounds[len].lock().restart();
                    }
                    match event.code {
                        KeyCode::Up => {
                            //beep_sounds[0].lock().set_volume(2.0);
                            beep_sounds[0].resume();
                            if menu_select > 0 {
                                menu_select -= 1;
                            } else {
                                menu_select = entries.len()-1;
                            }
                        }
                        KeyCode::Down => {
                            //beep_sounds[0].lock().set_volume(2.0);
                            beep_sounds[0].resume();
                            if menu_select < entries.len()-1 {
                                menu_select += 1;
                            } else {
                                menu_select = 0;
                            }
                        }
                        KeyCode::Enter => {
                            select_sound.lock().restart();
                            select_sound.resume();
                            match menu_select {
                                0 => {
                                    //enter_game(&mut *player_data);
                                    dialog_box(&mut stdout, &options_state, "You have selected \"New Game\".", "???");
                                }, //New Game
                                1 => unimplemented!(), //Continue
                                2 => {
                                    options_state = options_menu(
                                        &mut beep_sounds,
                                        beep_volume,
                                        &mut select_sound,
                                        select_volume,
                                        options_state.clone(),
                                    ); //Options
                                    for track in beep_sounds.iter_mut() {
                                        track.lock().set_volume(beep_volume * options_state.master_volume);
                                    }
                                    select_sound.lock().set_volume(select_volume * options_state.master_volume);
                                }
                                3 => {
                                    write_save(&options_state)?;
                                }
                                _ => panic!("`menu_select` outside of expected parameters\n"),
                            }
                        }
                        KeyCode::End | Char('c') | KeyCode::Esc => in_menu = false,
                        _ => {},
                    }
                }
            },
            _ => {}, // add some kind of feedback (preferably a short sound effect)
        } 
            // Render menu based on which item is selected
        let menu = List::new(
            entries
                .clone()
                .into_iter()
                .map(|item| {
                    if menu_select == item_count {
                        item_count += 1;
                        ListItem::new(RIGHT_POINTING_TRIANGLE.to_owned() + item)
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
                    .border_set(BORDERS[options_state.frame_style_index].clone())
                    .style(Style::default().fg(BORDER_COLORS[options_state.frame_color_index].clone())),
            );

            f.render_widget(tabs, chunks[0]);
        })?;

        item_count = 0;
    }

    //execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(options_state)
}

pub fn options_menu(
    beep_sounds: &mut Vec<AudioDevice<Sound>>,
    beep_volume: f32,
    select_sound: &mut AudioDevice<Sound>,
    select_volume: f32,
    options_state: Options,
) -> Options {


    execute!(io::stdout(), EnterAlternateScreen).unwrap();

    let mut options = options_state.clone();
    
    let entries = vec!["Frame color", "Frame style", "Text speed", "Master Volume"];
    let slider_length = 20;
    let mut master_volume_slider = Slider::new(slider_length, options.master_volume as f64 * 100.0);

    let mut in_menu = true;
    let mut confirm = false;

    let mut menu_select = 0;
    let mut item_count = 0;

    let mut color_index = options.frame_color_index;
    let mut border_index = options.frame_style_index;

    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    execute!(io::stdout(), EnterAlternateScreen).unwrap();

    while in_menu {
        let input_wait = Instant::now();
        
        let mut event_list = vec![Event::Key(KeyEvent::new_with_kind_and_state(
            KeyCode::Null,
            KeyModifiers::NONE,
            KeyEventKind::Release,
            KeyEventState::NONE
        ))];


        while input_wait.elapsed().as_millis() <= 100 {            
            if poll(Duration::from_millis(1)).unwrap() {
                let next_event = read().unwrap();
                #[allow(clippy::single_match)]
                match next_event {
                    Event::Key(event) => {
                        if event.kind != KeyEventKind::Release && event.code != KeyCode::Null {
                            event_list.push(Event::Key(event));
                        }
                    },
                    _ => {},
                }
            }
        }
        if event_list.len() > 1 {
            for _ in 0..event_list.len()-1 {
                event_list.remove(0);
            }
        }
        
        #[allow(clippy::single_match)]
        match event_list[0] {
            Event::Key(event) => {
            if event.kind != KeyEventKind::Release && event.code != KeyCode::Null { // Only act on press or repeat
                let swap = beep_sounds.remove(0);
                beep_sounds.push(swap);

                for j in 1..beep_sounds.len() {
                    //beep_sounds[j].lock().set_volume(0.8);
                    beep_sounds[j].pause();
                }
                let len = beep_sounds.len() - 1;
                beep_sounds[len].lock().restart();

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
                                    color_index = BORDER_COLORS.len() - 1;
                                } else {
                                    color_index -= 1;
                                }
                            }
                            1 => {
                                if border_index == 0 {
                                    border_index = BORDERS.len() - 1;
                                } else { border_index -= 1; }
                            }
                            2 => {
                                if options.text_speed > 1 {
                                    options.text_speed -= 1;
                                }
                            }
                            3 => {
                                if options.master_volume > 0.0 {
                                    options.master_volume = 
                                    if options.master_volume > 1.0 / slider_length as f32 {
                                        options.master_volume - 0.05
                                    } else {
                                        0.0
                                    };
                                }
                                for track in beep_sounds.iter_mut() {
                                    track.lock().set_volume(beep_volume * options.master_volume);
                                }
                                select_sound.lock().set_volume(select_volume * options.master_volume);
                                beep_sounds[0].resume();
                            }
                            _ => {}
                        },
                        KeyCode::Right => match menu_select {
                            0 => {
                                if color_index == BORDER_COLORS.len() - 1 {
                                    color_index = 0;
                                } else {
                                    color_index += 1;
                                }
                            }
                            1 => {
                                if border_index == BORDERS.len() - 1 {
                                    border_index = 0;
                                } else { border_index += 1; }
                            }
                            2 => {
                                if options.text_speed < 5 {
                                    options.text_speed += 1;
                                }
                            }
                            3 => {
                                if options.master_volume < 1.0 {
                                    options.master_volume =
                                    if 1.0 - options.master_volume < 0.05 {
                                        1.0
                                    } else {
                                        options.master_volume + 0.05
                                    };
                                }
                                for track in beep_sounds.iter_mut() {
                                    track.lock().set_volume(beep_volume * options.master_volume);
                                }
                                select_sound.lock().set_volume(select_volume * options.master_volume);
                                beep_sounds[0].resume();

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
            }
            _ => {}
        }
        // Render menu based on which item is selected
        master_volume_slider.position = options.master_volume as f64 * 100.0;
        let menu = List::new(
            entries
                .clone()
                .into_iter()
                .map(|item| {
                    let state = match item_count {
                        0 => COLOR_NAMES[color_index].to_string(),
                        1 => BORDER_NAMES[border_index].to_string(),
                        2 => format!("{:?}", options.text_speed),
                        3 => format!("{} ({}%)", master_volume_slider, (options.master_volume * 100.0).round()),
                        _ => "Error: `item_count` out of bounds.".to_string(),
                    };

                    if menu_select == item_count {
                        item_count += 1;
                        ListItem::new(RIGHT_POINTING_TRIANGLE.to_owned() + item + " : " + state.as_str())
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
                        .border_set(BORDERS[border_index])
                        .style(Style::default().fg(BORDER_COLORS[color_index])),
                );

                f.render_widget(tabs, chunks[0]);
            })
            .unwrap();

        item_count = 0;
    }

    execute!(io::stdout(), LeaveAlternateScreen).unwrap();

    if confirm {
        options = Options {
            frame_color_index: color_index,
            frame_style_index: border_index,
            text_speed: options.text_speed,
            master_volume: options.master_volume,
        };
    }
    options
}


