#[derive(Debug, Clone, Copy)]
pub enum BattleType {
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
    Ice,
}

impl BattleType {
    pub fn weaknesses(&self) -> Vec<&Self> {
        match self {
            Self::Normal => vec![&Self::Fighting],
            Self::Fighting => vec![&Self::Psychic, &Self::Flying],
            Self::Psychic => vec![&Self::Dark, &Self::Bug],
            Self::Dark => vec![&Self::Fighting, &Self::Fairy],
            Self::Ghost => vec![self, &Self::Dark],
            Self::Bug => vec![&Self::Flying, &Self::Fire, &Self::Rock],
            Self::Grass => vec![
                &Self::Fire,
                &Self::Flying,
                &Self::Bug,
                &Self::Poison,
                &Self::Ice,
                &Self::Poison,
            ],
            Self::Fire => vec![&Self::Water, &Self::Ground, &Self::Rock],
            Self::Water => vec![&Self::Electric, &Self::Grass],
            Self::Electric => vec![&Self::Ground],
            Self::Ground => vec![&Self::Water, &Self::Grass, &Self::Ice],
            Self::Rock => vec![
                &Self::Water,
                &Self::Fighting,
                &Self::Grass,
                &Self::Ground,
                &Self::Steel,
            ],
            Self::Dragon => vec![self, &Self::Fairy],
            Self::Fairy => vec![&Self::Poison, &Self::Steel],
            Self::Steel => vec![&Self::Fire, &Self::Fighting, &Self::Ground],
            Self::Poison => vec![&Self::Psychic, &Self::Ground],
            Self::Ice => vec![&Self::Fighting, &Self::Steel, &Self::Rock, &Self::Fire],
            Self::None | _ => vec![],
        }
    }
    pub fn resists(&self) -> Vec<&Self> {
        match self {
            Self::Fighting => vec![&Self::Rock, &Self::Bug, &Self::Dark],
            Self::Psychic => vec![self, &Self::Fighting],
            Self::Dark => vec![self, &Self::Ghost],
            Self::Ghost => vec![&Self::Poison, &Self::Bug],
            Self::Bug => vec![&Self::Fighting, &Self::Ground, &Self::Grass],
            Self::Grass => vec![self, &Self::Water, &Self::Ground, &Self::Electric],
            Self::Fire => vec![
                self,
                &Self::Steel,
                &Self::Grass,
                &Self::Bug,
                &Self::Ice,
                &Self::Fairy,
            ],
            Self::Water => vec![self, &Self::Ice, &Self::Steel, &Self::Fire],
            Self::Electric => vec![self, &Self::Steel, &Self::Flying],
            Self::Ground => vec![&Self::Rock, &Self::Poison],
            Self::Rock => vec![
                self,
                &Self::Fire,
                &Self::Normal,
                &Self::Flying,
                &Self::Poison,
            ],
            Self::Dragon => vec![&Self::Water, &Self::Fire, &Self::Electric, &Self::Grass],
            Self::Fairy => vec![&Self::Fighting, &Self::Bug, &Self::Dark],
            Self::Steel => vec![
                self,
                &Self::Rock,
                &Self::Normal,
                &Self::Flying,
                &Self::Bug,
                &Self::Fairy,
                &Self::Grass,
                &Self::Psychic,
                &Self::Ice,
                &Self::Dragon,
            ],
            Self::Poison => vec![
                self,
                &Self::Fighting,
                &Self::Bug,
                &Self::Grass,
                &Self::Fairy,
            ],
            _ => vec![],
        }
    }
    pub fn immune_against(&self) -> Vec<&Self> {
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
