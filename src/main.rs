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
use std::thread;
use std::io::Write;

use std::result::Result;

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


// ! TEMPORARY ! : TODO: Place audio playing functions into an enum (maybe via code generation)
// Hard-coding strings is a work-around for dealing with string movement semantics (for now)

fn play_intro() {
    let handle = move || {
        Command::new(&String::from("ffplay"))
            .args(&["-nodisp".to_string(), "Rustemon_Intro_Sound.wav".to_string(), 
                  "-autoexit".to_string(), "-loglevel".to_string(), "quiet".to_string() ])
            .spawn().expect("command failed.");

        print!("\x1B[A");
        };
    thread::spawn(move || { handle();  } );
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
    
	pub fn new(dex: usize, level: usize) -> RefCell<Monster<'a>> { //Implemeneted as a refcell
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
			lvl: level as u8,
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



// COPYRIGHT MESSAGE
const COPYRIGHT: &str = "\tInspired by and adapted from \n\
                         \tthe works of GAME FREAK Inc. , by whom \n\
                         \tcopyright (\u{00a9}) is reserved. This project is \n\
                         \tpublished as an open-source, non-profit endeavor.";


fn main() {

    //
    // KEEP THIS HERE FOR LEGAL REASONS!!!
    // ( Copyright message, stylized )
    
    // Make some room and return to top of message (required for the spawned ffplay thread to not displace output)
    for _ in 0..40 {
        println!();
    }
    print!("\x1B[5A");

    for i in 0..100 {
        let mut i = i;
        if i == 50 {
            play_intro();
            std::thread::sleep(std::time::Duration::from_millis(3000)); //Must be longer than
                                                                        //how long it takes to
                                                                        //complete audio thread
        } else if i > 50 {
            i = 100-i;
        }
        let shade = format!("\x1B[38;2;{0};{0};{0}m", (i as f32*5.222) as usize );
        println!("{}{}", &shade, &COPYRIGHT);
        std::thread::sleep(std::time::Duration::from_millis(20)); //TODO: Tie animation to actual
                                                                  //time passed rather than
                                                                  //sleeping for a set amount of
                                                                  //time such as in the
                                                                  //intro animation
        
        print!("\x1B[4A");
        
    }
    //Reset text style
    print!("\x1B[0m");
    
    // Play Intro Sequence ( TODO: implement skip functionality )

    let mut playing_intro = true;
    let mut intro_state = 0;
    let animation_speed = 100;
    let intro_ptr = &animation_data::intro::FRAME_DATA;
    let intro_start = std::time::Instant::now();

    for _ in 0..14 {
        println!();
    }
    while playing_intro {
        if intro_start.elapsed().as_millis() as usize >= (animation_speed * intro_state) {
            
            println!("\x1B[14A");
            let _ = std::io::stdout().write(intro_ptr[intro_state as usize].as_bytes() );
            

            intro_state += 1;
        }
        if intro_state >= intro_ptr.len().try_into().unwrap() {
            playing_intro = false;
        }
    }


    // MAIN LOGIC

    //Initialize player data
    let mut player_party = Party::new();
    player_party[0] = Monster::new(0, 0);

    

    // TESTING
    dbg!(&player_party[0]);
	println!("{:?}",BattleType::Fire.weaknesses());
	println!("{}",format!("{:?}", BattleType::Fire));
    println!("{:?}, {:?}", Pokemon[0], Pokemon[0].name());
    
}


