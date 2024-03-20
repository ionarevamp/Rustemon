extern crate rand;

use std::cell::RefCell;

// define descriptions, stat generation, etc.
// consider controller support --
//	 - Bluetooth resource: https://dev.to/lcsfelix/using-rust-blurz-to-read-from-a-ble-device-gmb
//	 - Capturing other input likely requires something like libx11 (window manager) as 
//		terminals do not support keydown/keyup events by themselves

mod mons;
mod moves;
mod types;

use crate::mons::*;
use crate::types::*;
use crate::moves::{Effect, PokeMove, Target};
use std::borrow::BorrowMut;

#[derive(Clone, Debug)]
struct Monster<'a> {
	monster_id: Pokemon,
	battle_type: [BattleType; 2],
	iv: [u8; 6],
	ev: [u8; 6],
	stats: [u8; 6],
	curstats: [u16; 6],
	lvl: u8,
	status: Effect<'a>,
	moves: &'static [PokeMove],
	owned: bool,
	name: String
}
impl<'a> Monster<'a> {
	fn new(dex: usize) -> RefCell<Monster<'a>> {
		RefCell::new( Monster {
			monster_id: mon_at_dex(dex),
			battle_type: mon_at_dex(dex).types(),
			iv: [0; 6],
			ev: [0; 6],
			stats: mon_at_dex(dex).base_stats(),
			curstats: [0; 6],
			lvl: 1,
			status: Effect::Null,
			moves: &[PokeMove::Growl],
			owned: false,
			name: mon_at_dex(dex).name()
		} )
	}
	fn clear_effect(&mut self) {
		self.status = Effect::Null;
	}
	fn health(&mut self) -> &mut u16 {
		return self.curstats[0].borrow_mut();
	}
	fn attack(&mut self) -> &mut u16 {
		return self.curstats[1].borrow_mut();
	}
	fn defense(&mut self) -> &mut u16 {
		return self.curstats[2].borrow_mut();
	}
	fn spattack(&mut self) -> &mut u16 {
		return self.curstats[3].borrow_mut();
	}
	fn spdefense(&mut self) -> &mut u16 {
		return self.curstats[4].borrow_mut();
	}
	fn speed(&mut self) -> &mut u16 {
		return self.curstats[5].borrow_mut();
	}
}


fn main() {
    
	let mut testmon = Monster::new(1);
	println!("{:?}",testmon);
	println!("{:?}",BattleType::Psychic.weaknesses());
	println!("{}",format!("{:?}", BattleType::Fire));
}
