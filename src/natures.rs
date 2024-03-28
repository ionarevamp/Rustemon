use crate::Monster;
use rand::prelude::*;

/*
 * Nature	Increases	Decreases
Adamant	Attack	Sp. Atk
Bashful	Sp. Atk	Sp. Atk
Bold	Defense	Attack
Brave	Attack	Speed
Calm	Sp. Def	Attack
Careful	Sp. Def	Sp. Atk
Docile	Defense	Defense
Gentle	Sp. Def	Defense
Hardy	Attack	Attack
Hasty	Speed	Defense
Impish	Defense	Sp. Atk
Jolly	Speed	Sp. Atk
Lax 	Defense	Sp. Def
Lonely	Attack	Defense
Mild	Sp. Atk	Defense
Modest	Sp. Atk	Attack
Naive	Speed	Sp. Def
Naughty	Attack	Sp. Def
Quiet	Sp. Atk	Speed
Quirky	Sp. Def	Sp. Def
Rash	Sp. Atk	Sp. Def
Relaxed	Defense	Speed
Sassy	Sp. Def	Speed
Serious	Speed	Speed
Timid	Speed	Attack
*/

#[derive(Debug, Clone)]
pub enum Nature {
   Adamant,
   Bashful,
   Bold,
   Brave,
   Calm,
   Careful,
   Docile,
   Gentle,
   Hardy,
   Hasty,
   Impish,
   Jolly,
   Lax,
   Lonely,
   Mild,
   Modest,
   Naive,
   Naughty,
   Quiet,
   Quirky,
   Rash,
   Relaxed,
   Sassy,
   Serious,
   Timid,
}

impl Nature {
    pub fn new() -> Nature {
        let mut rng = rand::thread_rng();
        let which: u8 = rng.gen_range(0..25);
        match which {
            0u8 => Self::Adamant,
            1u8 => Self::Bashful,
            2u8 => Self::Bold,
            3u8 => Self::Brave,
            4u8 => Self::Calm,
            5u8 => Self::Careful,
            6u8 => Self::Docile,
            7u8 => Self::Gentle,
            8u8 => Self::Hardy,
            9u8 => Self::Hasty,
            10u8 => Self::Impish,
            11u8 => Self::Jolly,
            12u8 => Self::Lax,
            13u8 => Self::Lonely,
            14u8 => Self::Mild,
            15u8 => Self::Modest,
            16u8 => Self::Naive,
            17u8 => Self::Naughty,
            18u8 => Self::Quiet, 
            19u8 => Self::Quirky,
            20u8 => Self::Rash,
            21u8 => Self::Relaxed,
            22u8 => Self::Sassy,
            23u8 => Self::Serious,
            _ => Self::Timid,
        }                
    }
}
