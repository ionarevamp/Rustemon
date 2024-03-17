
// define descriptions, stat generation, etc.

#[derive(Debug, Clone, Copy)]
enum BattleType {
	None,
    Normal,
    Fighting,
	Psychic,
	Dark,
	Ghost,
	Bug,
	Grass,
	Fire,
	Water,
	Electric,
	Ground,
	Rock,
	Dragon,
	Fairy,
	Flying,
	Steel,
	Poison,
	Ice
}

impl BattleType {
    fn weaknesses(&self) -> Vec<&Self> {
        match self {
            Self::Normal => vec![&Self::Fighting],
            Self::Fighting => vec![&Self::Psychic, &Self::Flying],
			Self::Psychic => vec![&Self::Dark, &Self::Bug],
			Self::Dark => vec![&Self::Fighting, &Self::Fairy],
			Self::Ghost => vec![self, &Self::Dark],
			Self::Bug => vec![&Self::Flying, &Self::Fire, &Self::Rock],
			Self::Grass => vec![&Self::Fire, &Self::Flying, &Self::Bug, &Self::Poison, &Self::Ice, &Self::Poison],
			Self::Fire => vec![&Self::Water, &Self::Ground, &Self::Rock],
			Self::Water => vec![&Self::Electric, &Self::Grass],
			Self::Electric => vec![&Self::Ground],
			Self::Ground => vec![&Self::Water, &Self::Grass, &Self::Ice],
			Self::Rock => vec![&Self::Water, &Self::Fighting, &Self::Grass, &Self::Ground, &Self::Steel],
			Self::Dragon => vec![self, &Self::Fairy],
			Self::Fairy => vec![&Self::Poison, &Self::Steel],
			Self::Steel => vec![&Self::Fire, &Self::Fighting, &Self::Ground],
			Self::Poison => vec![&Self::Psychic, &Self::Ground],
			Self::Ice => vec![&Self::Fighting, &Self::Steel, &Self::Rock, &Self::Fire],
			Self::None | _ => vec![],
		
        }
    }
	fn resists(&self) -> Vec<&Self> {
		match self {
			Self::Fighting => vec![&Self::Rock, &Self::Bug, &Self::Dark],
			Self::Psychic => vec![self, &Self::Fighting],
			Self::Dark => vec![self, &Self::Ghost],
			Self::Ghost => vec![&Self::Poison, &Self::Bug],
			Self::Bug => vec![&Self::Fighting, &Self::Ground, &Self::Grass],
			Self::Grass => vec![self, &Self::Water, &Self::Ground, &Self::Electric],
			Self::Fire => vec![self, &Self::Steel, &Self::Grass, &Self::Bug, &Self::Ice, &Self::Fairy],
			Self::Water => vec![self, &Self::Ice, &Self::Steel, &Self::Fire],
			Self::Electric => vec![self, &Self::Steel, &Self::Flying],
			Self::Ground => vec![&Self::Rock, &Self::Poison],
			Self::Rock => vec![self, &Self::Fire, &Self::Normal, &Self::Flying, &Self::Poison],
			Self::Dragon => vec![&Self::Water, &Self::Fire, &Self::Electric, &Self::Grass],
			Self::Fairy => vec![&Self::Fighting, &Self::Bug, &Self::Dark],
			Self::Steel => vec![self, &Self::Rock, &Self::Normal, 
						&Self::Flying, &Self::Bug, &Self::Fairy,
						&Self::Grass, &Self::Psychic, &Self::Ice, &Self::Dragon],
			Self::Poison => vec![self, &Self::Fighting, &Self::Bug, &Self::Grass, &Self::Fairy],
			_ => vec![],
		}
	}
    fn immune_against(&self) -> Vec<&Self> {
        match self {
            Self::Normal => vec![&Self::Ghost],
			Self::Dark => vec![&Self::Psychic],
			Self::Ghost => vec![&Self::Normal, &Self::Fighting],
			Self::Normal => vec![&Self::Ghost],
			Self::Fairy => vec![&Self::Dragon],
			Self::Flying => vec![&Self::Ground],
			Self::Ground => vec![&Self::Electric],
			Self::Steel => vec![&Self::Poison],
			Self::Bug => vec![&Self::Dark],
			_ => vec![],
        }
    }
}

#[derive(Debug,Clone,Copy)]
enum Stat {
	Health,
	Attack,
	Defense,
	SpAttack,
	SpDefense,
	Speed
	
}

impl Stat {
	fn battle_stat<'a>(&self, mon: &'a mut Pokemon) -> &'a mut u16 {
		match self {
			Self::Health => &mut mon.curstats[0],
			Self::Attack => &mut mon.curstats[1],
			Self::Defense => &mut mon.curstats[2],
			Self::SpAttack => &mut mon.curstats[3],
			Self::SpDefense => &mut mon.curstats[4],
			Self::Speed => &mut mon.curstats[5],
		}
	}
}



#[derive(Debug, Clone, Copy)]
enum PokeMove {
	Scratch,
	Growl,
	Ember,
	Confusion
	
}

impl PokeMove {
	fn go(&self, actor: &mut Pokemon, mut target: &mut Pokemon) {
		match self {
			Self::Growl => {
				Effect::ChangeAttack.apply(&mut target, -20i32);
				
			},
			_ => todo!(),
		}
	}
}



#[derive(Debug, Clone, Copy)]
enum Effect {
	Null,
	ChangeAttack,
	Damage,
	Heal,
	Confused
}
 
impl Effect {
	fn apply(&self, target: &mut Pokemon, amount: i32) {
		match self {
			Self::ChangeAttack => {
				if amount < 0 {
					*Stat::Attack.battle_stat(target) -= (-1i32*amount) as u16;
				} else {
					*Stat::Attack.battle_stat(target) += amount as u16;
				}

			},
			_ => todo!(),
		}
	}
}


#[derive(Clone,Debug)]
struct Pokemon {
	battle_type: &'static [BattleType],
	iv: [u8; 6],
	ev: [u8; 6],
	stats: [u8; 6],
	curstats: [u16; 6],
	lvl: u8,
	status: Effect,
	moves: &'static [PokeMove],
	desc: String
}
impl Pokemon {
	fn new() -> Pokemon {
		Pokemon {
			battle_type: &[BattleType::Normal],
			iv: [0; 6],
			ev: [0; 6],
			stats: [0; 6],
			curstats: [0; 6],
			lvl: 0,
			status: Effect::Null,
			moves: &[PokeMove::Growl],
			desc: String::new()
		}
	}
	fn from_stats(
		types: &'static [BattleType],
		health: u8, 
		attack: u8, 
		defense: u8, 
		spattack: u8, 
		spdefense: u8, 
		speed: u8) -> Pokemon
	{
		Pokemon {
			battle_type: types,
			iv: [0; 6],
			ev: [0; 6],
			stats: [health,attack,defense,spattack,spdefense,speed],
			curstats: [health.into(),attack.into(),defense.into(),spattack.into(),spdefense.into(),speed.into()],
			lvl: 1,
			status: Effect::Null,
			moves: &[PokeMove::Growl],
			desc: String::new()
		}
	}
}


fn main() {
    
	let mut testmon = Pokemon::new();
	println!("{:?}",testmon);
	println!("{:?}",BattleType::Psychic.weaknesses());
	println!("{}",format!("{:?}", BattleType::Fire));
}
