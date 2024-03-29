extern crate rand;

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

mod mons;
mod natures;
mod moves;
mod types;
mod animation_data;

use crate::mons::*;
use crate::types::*;
use crate::natures::Nature;
use crate::moves::{Effect, PokeMove, Target};

use rand::prelude::*;
use std::process::Command;
use std::thread::{self, sleep, JoinHandle};
use std::io::Write;
use std::sync::mpsc::*;

use std::{io, time::{
    Instant,
    Duration}
};
use tui::{
    text::{Span, Spans},
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders, Tabs, List, ListItem},
    layout::{Layout, Constraint, Direction, Rect},
    Terminal,
    style::{Style, Color},
};
use crossterm::{
    event::{self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyEvent, KeyEventKind, KeyCode::{self, *}, KeyModifiers},
    ExecutableCommand, execute,
    cursor::{DisableBlinking, EnableBlinking, MoveUp, MoveDown, MoveTo, RestorePosition, SavePosition, Hide, Show},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::result::Result;



// ! TEMPORARY ! : TODO: Place audio playing functions into an enum (maybe via code generation)
// Hard-coding strings is a work-around for dealing with string movement semantics (for now)
fn play_intro() {
    
    let arguments = ["-nodisp".to_string(), "Rustemon_Intro_Sound.mp3".to_string(), 
                  "-autoexit".to_string(), "-loglevel".to_string(), "quiet".to_string(),
                    ];
    let handle = move |args| {
        let mut command = Command::new(&String::from("ffplay")); 
        command.args(&args)
            .spawn().expect("command failed.");
        
        let _ = execute!(io::stdout(), MoveUp(1));
    };
    thread::spawn(move || { handle(arguments.clone());  } );
}

#[derive(Debug, Clone)]
struct Party<'a> {
    mons: [RefCell<Monster<'a>>; 6],
    owned: bool,
}

impl<'a> Party<'a> {
    fn new() -> Party<'a> {
        Party {
            mons: [ // Initialized this way because Copy is not implemented
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
    party: Party<'a>
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
	name: String
}

impl<'a> Monster<'a> {
    
	pub fn new(dex: usize, level: u8) -> RefCell<Monster<'a>> { //Implemeneted as a refcell
                                                                   //because multiple sources may
                                                                   //end up needing to alter a
                                                                   //monster in a single pass
		RefCell::new( Monster {
			monster_id: mon_at_dex(dex),
			battle_type: mon_at_dex(dex).types(),
			iv: [0; 6],
			ev: [0; 6],
			stats: mon_at_dex(dex).base_stats(),
			curstats: [0; 6],
            accuracy: 100.0f64,
            crit_level: 0u8,
			lvl: level,
			status: Effect::Null,
			moves: &[PokeMove::Null; 4],
			owned: false,
            affection: 0i32,
            nature: Nature::new(),
			name: mon_at_dex(dex).name()
		} )
	}
    
    pub fn nature_modify(&mut self) -> [(usize, f64); 2] {
        let (modadd, modsub) = (0.2f64,-0.2f64);
        match self.nature {
            Nature::Adamant => [(1,modadd),(3,modsub)],
            Nature::Bashful => [(3,modadd),(3,modsub)],
            Nature::Bold => [(2,modadd),(1,modsub)],
            Nature::Brave => [(1,modadd),(5,modsub)],
            Nature::Calm => [(4,modadd),(1,modsub)],
            Nature::Careful => [(4,modadd),(2,modsub)],
            Nature::Docile => [(2,modadd),(2,modsub)],
            Nature::Gentle => [(4,modadd),(2,modsub)],
            Nature::Hardy => [(1,modadd),(2,modsub)],
            Nature::Hasty =>[(5,modadd),(2,modsub)], 
            Nature::Impish => [(2,modadd),(3,modsub)], 
            Nature::Jolly => [(5,modadd),(2,modsub)],
            Nature::Lax => [(3,modadd),(4,modsub)],
            Nature::Lonely => [(1,modadd),(2,modsub)],
            Nature::Mild  => [(3,modadd),(2,modsub)],
            Nature::Modest => [(3,modadd),(2,modsub)],
            Nature::Naive => [(5,modadd),(4,modsub)],
            Nature::Naughty => [(1,modadd),(4,modsub)],
            Nature::Quiet => [(3,modadd),(5,modsub)],
            Nature::Quirky => [(4,modadd),(5,modsub)],
            Nature::Rash => [(3,modadd),(4,modsub)],
            Nature::Relaxed => [(2,modadd),(5,modsub)],
            Nature::Sassy => [(4,modadd),(5,modsub)],
            Nature::Serious => [(5,modadd),(5,modsub)],
            Nature::Timid => [(5,modadd),(1,modsub)],
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
            if i == 0 { // health

                mon.curstats[i] = 
                    (0.01 * ( (2.0*(base + iv) + ( 0.25 * ev ).floor()) * level ) + level + 10.0) as u16;

            } else { // other stats

                let mut modifier: f64 = 1.0;
                let nature_mod = mon.nature_modify();
                if i == nature_mod[0].0 {
                    modifier += nature_mod[0].1;
                }
                if i == nature_mod[1].0 {
                    modifier += nature_mod[1].1;
                }
                mon.curstats[i] =
                    ((0.01 * ((2.0*(base + iv) + (0.25 * ev).floor())* level) + 5.0) * modifier) as u16;

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


pub fn main_menu(player_data: &mut Player) -> io::Result<()> {
    
    let entries = vec![
        "New Game",
        "Continue",
        "Options"
    ];

    let mut in_menu = true;
    let mut menu_select = 0;
    let mut item_count = 0;

    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
       
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        
       while in_menu { 
            if poll(Duration::from_millis(100)).expect("IO error in menu.") {
                match read() {
                    Ok(Event::Key(event)) => match event.code {
                        KeyCode::Up => { if menu_select > 0 {
                                            menu_select -= 1;
                                        } else {
                                            print!("");
                                        }
                                    },
                        KeyCode::Down => { if menu_select <= 2 {
                                            menu_select += 1;
                                        } else {
                                            print!("");
                                        }
                                    },
                        Char('c') => in_menu = false,
                        _ => print!(""),
                    },
                    _ => print!(""), // add some kind of feedback (preferably a short sound effect)
                }
            }
           // Render menu based on which item is selected
               let menu = List::new(entries.clone()
                   .into_iter()
                   .map(|item| {
                       if menu_select == item_count {
                           item_count += 1;
                           ListItem::new("> ".to_owned() + item)
                               .style(Style::default().fg(Color::Red).bg(Color::White))
                       } else {
                           item_count += 1;
                           ListItem::new(item)
                       }
                   })
                   .collect::<Vec<_>>()
               );
        
    terminal.draw(|f| {

       let chunks = Layout::default()
           .direction(Direction::Horizontal)
           .margin(1)
           .constraints(
               [
                    Constraint::Percentage(40),
                    Constraint::Percentage(50)
               ].as_ref()
           )
           .split(
               Rect {
                   x: 0,
                   y: 0,
                   width: f.size().width,
                   height: (entries.len()+3 +2) as u16 
                       // plus 3 to account for margins, then another 2 to add whitespace
               }
            );
    
               let tabs = menu
                   .block(
                       Block::default()
                       .title("MENU")
                       .borders(Borders::ALL)
                   );

        f.render_widget(tabs, chunks[0]);

    })?;


        item_count = 0;
        
       }
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    Ok(())

}


// COPYRIGHT MESSAGE
const COPYRIGHT_MESSAGE: [&str; 4] = ["Inspired by and adapted from ",
                         "the works of GAME FREAK Inc. , by whom ",
                         "copyright (\u{00a9}) is reserved. This project is ",
                         "published as an open-source, non-profit endeavor."];


fn main() -> Result<(), io::Error> {

    let mut stdout = io::stdout();
    execute!(stdout, Hide);

    enable_raw_mode()?;

    //let backend = CrosstermBackend::new(io::stdout());
    //let mut terminal = Terminal::new(backend)?;

    //
    // KEEP THIS HERE FOR LEGAL REASONS!!!
    // ( Copyright message, stylized )
    
    // Make some room
    for _ in 0..40 {
        println!();
    }
    execute!(stdout, MoveTo(0,19));

    let mut intro_time = 2440; //milliseconds
    for i in 0..100 {
        let mut i = i;
        if poll(Duration::from_millis(20))? {
            execute!(stdout, Clear(ClearType::All), MoveTo(0,22))?;
            match read()? {
                _ => break,
            }// if user does anything, skip the rest
        } else {
            if i == 50 {
                play_intro();
                if poll(Duration::from_millis(3000))? {
                    let timer = Instant::now();
                    execute!(stdout, Clear(ClearType::All), MoveTo(0,22));
                    match read()? {
                        _ => { while (timer.elapsed().as_millis() as usize) < intro_time {
                            for line_num in 0..4 {                                                                                           execute!(stdout, MoveTo(0,19+line_num))?;                                                                    print!("{}", &COPYRIGHT_MESSAGE[line_num as usize]);                                                     } //Display while waiting
                            };
                            break}, // allow interrupt during sound effect
                    }
                }
            } else if i > 50 {
                i = 100-i;
            }
            let shade = format!("\x1B[38;2;{0};{0};{0}m", (i as f32*5.222) as usize );
            print!("{}", &shade);
            for line_num in 0..4 {
                execute!(stdout, MoveTo(0,19+line_num))?;
                print!("{}", &COPYRIGHT_MESSAGE[line_num as usize]);
            }
 
        }
        if intro_time >= 20 {
            intro_time -= 20;
        }

    }

    execute!(stdout, Clear(ClearType::All))?;

    //Reset text style
    print!("\x1B[0m");
    
    // Play Intro Sequence ( TODO: implement skip functionality )

    let mut playing_intro = true;
    let mut intro_state = 0;
    let animation_speed = 100;
    let intro_ptr = &animation_data::intro::FRAME_DATA;
    let intro_start = std::time::Instant::now();

    while playing_intro {
        
        if intro_start.elapsed().as_millis() as usize >= (animation_speed * intro_state) {
            
            let frame = intro_ptr[intro_state as usize].lines()
                            .collect::<Vec<&str>>();
            for i in 0..frame.len() {
                
                print!("\x1B[{};1H", 20+i);
                let _ = io::stdout().write(frame[i].as_bytes());

            }

            let _ = io::stdout().flush();

            intro_state += 1;
        }
        if poll(Duration::from_millis((animation_speed * intro_state - intro_start.elapsed().as_millis() as usize).try_into().unwrap()))? {
            match read()? {
                _ => break,
            }
        }
        if intro_state >= intro_ptr.len().try_into().unwrap() {
            playing_intro = false;
        }
    }


    // MAIN LOGIC

    //Initialize player data
    let mut player = Player { name: "DEV".to_string(), party: Party::new() };
    player.party[0] = Monster::new(150, 0u8);
    
    // Menu
    
    { main_menu(&mut player);
      disable_raw_mode();
    }

    disable_raw_mode()?;

    execute!(io::stdout(), Show);

    // TESTING
    dbg!(&player.party[0]);
	println!("{:?}",BattleType::Fire.weaknesses());
	println!("{}",format!("{:?}", BattleType::Fire));
    println!("{:?}, {:?}", Pokemon[0], Pokemon[0].name());


    Ok(())
    
}


