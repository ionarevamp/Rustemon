
use crate::mons::*;
use crate::types::*;
use crate::Monster;

use rand::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Effect<'a> {
	Null,
	ChangeAttack,
	ChangeDefense,
	ChangeSpAttack,
	ChangeSpDefense,
	ChangeSpeed,
	Damage,
	Heal,
	Burn(&'a str, &'a str),
	Paralyzed(&'a str, &'a str),
	Infatuated(&'a str, &'a str),
	Asleep(&'a str, &'a str),
	Frozen(&'a str, &'a str),
	Enraged(&'a str, &'a str),
	Confused(&'a str, &'a str)
}
 
impl<'a> Effect<'a> {
	pub fn apply(&self, target: &mut Monster, amount: i32) {
		match self {
			Self::ChangeAttack => {
				if amount < 0 {
					*target.attack() -= (-1i32*amount) as u16;
				} else {
					*target.attack() += amount as u16;
				}
			},
			
			_ => todo!(),
		}
	}
}


#[derive(Debug, Clone, Copy)]
pub enum PokeMove {
	Null,
	Scratch,
	Growl,
	Ember,
	Confusion	
}

impl<'a> PokeMove {

    pub fn power(&self) -> f64 {
        match self {
            Self::Ember => 40.0,
            Self::Scratch => 40.0,
            _ => 0.0,
        }
    }

    pub fn get_type(&self) -> bool {
        match self {
            Self::Scratch => true,
            Self::Ember => false,
            _ => false,
        }
    }

    pub fn high_crit(&self) -> bool {
        match self {
            _ => false,
        }
    }

	pub fn go(&self, actor: &'a mut Monster, target: &'a mut Monster, accuracy: f64) -> Vec<String> {
		let mut rng = rand::thread_rng();
		let random_accuracy: f64 = rng.gen::<f64>() * 100.0 * accuracy;
		let random_power: f64 = rng.gen::<f64>() * 100.0;
		let attack = match self.get_type() {
            true => *actor.attack(),
            false => *actor.spattack(),
        } as f64;
        let defense = match self.get_type() {
            true => *target.defense(),
            false => *target.spdefense(),
        } as f64;
        
        let crit_rate: f64 = match actor.crit_level {
            0 => 1.0/24.0,
            1 => 1.0/8.0,
            2 => 0.5,
            _ => 1.0,
        };
        
        //TODO: add affection calculation
        

        let actor_spec = match actor.owned {
			true => "Foe ",
			false => "",
		};
		let mut message = Vec::new();
		message.push(format!("{}{} used {:?}!\n", actor_spec, actor.name, self));
		match self {
			Self::Growl => {
				if random_accuracy >= 100.0 * accuracy {
					Effect::ChangeAttack.apply(target, -20i32);
					message.push("Target was disarmed!\nAttack was lowered.\n".to_string());
				} else { message.push("Enemy avoided the attack!\n".to_string()); }
			},
			Self::Ember => {
				if random_accuracy >= 100.0 * accuracy { //TODO: determine damage and stats
					Effect::Damage.apply(target, 100i32);
				}
			},
			_ => todo!(),
		}
		message
	}
}

pub fn calculate_damage(
    attack: f64,
    defense: f64
) -> u16 {
    (attack-defense) as u16
}

pub enum Target {
	All,
	Foe,
	Foes,
	AtSelf,
	Team,
	Party,
	WholeParty	
}

