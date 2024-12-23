
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(unused_parens)]
use crate::extras::*;
use crate::mons::*;
use crate::moves::{Effect, PokeMove, Target};
use crate::natures::Nature;
use crate::types::*;

use rand::prelude::*;
use std::io::Write;
use std::process::Command;
use std::sync::mpsc::*;
use std::thread::{self, sleep, JoinHandle};

use std::{
    io,
    time::{Duration, Instant},
};

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
        SetBackgroundColor,
        Print
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
        LeaveAlternateScreen, size
    },
    ExecutableCommand,
};
use std::result::Result;

use crate::menu::Options;
use crate::*;

pub fn calc_text_rate(text_speed: u8) -> u64 {
    // returns how much time to wait between printing each scrolling character in milliseconds
    1000/((text_speed as u64 + 5u64)*2u64)
}

// TODO: Implement global tick. For now / testing, just use a timeout
// TODO: Implement proper Border Set processing to avoid unnecessary overhead
pub fn dialog_box(stdout: &mut std::io::Stdout, global_options: &Options, text: &str, speaker: &str) {

    let (_termwidth, termheight) = get_term_size();
    let mut text = text;
    let mut speaker = speaker;
    let timeout = calc_text_rate(global_options.text_speed());
    
    let max_line_length = TERM_WIDTH-4;
    let max_lines = 3;
    let mut char_idx = 0usize;
    let mut in_line_position = 0;
    let mut which_line = 0;

    let x_left = 3 as u16;
//    let x_right = (termwidth-3) as u16;
    let y_top = termheight-max_lines-1;
    let y_bottom = termheight-1;

    let mut text_chars = text.chars().collect::<Vec<char>>();

//    let mut top_line: Vec<char> = Vec::new();
//    let mut middle_line: Vec<char> = Vec::new();
//    let mut bottom_line: Vec<char> = Vec::new();

    let speaker_name_offset = 2+speaker.len();

    let mut in_msg = true;
    let mut msg_start = Instant::now();
    while in_msg {

        // DRAW BORDER HERE

        {
            let border_idx = global_options.frame_style_index;
            let border_color_idx = global_options.frame_color_index;
        }

        if which_line == 0 && speaker.len() > 0 && in_line_position == 0 {
           execute!(stdout, MoveTo(x_left, y_top));
           print!("{}",speaker.to_string() + ": ");
           in_line_position += speaker_name_offset;
           stdout.flush();
        }
        
        while (msg_start.elapsed().as_millis() as u64) < (timeout as u64 * char_idx as u64) {
            std::thread::sleep(Duration::from_millis( (timeout/4).into() ));
        }

        execute!(stdout, MoveTo((x_left+in_line_position as u16), y_top+which_line));
        print!("{}", text_chars[char_idx]);
        stdout.flush(); 


        // break up text by spaces
        'check_line_break: {
            let mut next_word_length = 0;
            if char_idx >= text_chars.len() {
                        break 'check_line_break;
                    }
            if text_chars[char_idx] == ' ' {
                for check_char_idx in char_idx+1..text_chars.len() { 
                    if text_chars[check_char_idx] != ' ' {
                        next_word_length += 1;
                    }
                    else {
                        break;
                    }
                }
            }
            
            in_line_position += 1;
            if next_word_length > 0 && in_line_position+next_word_length > max_line_length {
    //            execute!(stdout, MoveTo(x_left, termheight-max_lines-1+which_line as u16));
                execute!(stdout, MoveTo(x_left, y_top+which_line as u16));
                stdout.flush();
                //char_idx += 1;
                in_line_position = 0;
                which_line += 1;
                break 'check_line_break;
            }

            if in_line_position > max_line_length {
    //            execute!(stdout, MoveTo(x_left, termheight-max_lines-1+which_line as u16));
                execute!(stdout, MoveTo(x_left, y_top+which_line as u16));
                let _ = stdout.flush();
                in_line_position = 0;
                which_line += 1;
            }
        }

        if which_line == max_lines-1 && in_line_position == max_line_length {
            
            //Sleep instead of interaction (test)
            sleep(Duration::from_millis(1300));
            msg_start = Instant::now();
            //
            for i in y_top-1..=y_bottom+1 {
                execute!(stdout, MoveTo(x_left, i), Clear(ClearType::CurrentLine));
            }
            which_line = 0;
            in_line_position = 0;
            speaker = "";
            text = &text[char_idx..];
            text_chars = text.chars().collect::<Vec<char>>();
            char_idx = 0;

        }

        char_idx += 1;
        if char_idx >= text.len() {
            in_msg = false;
        }
    }


}






