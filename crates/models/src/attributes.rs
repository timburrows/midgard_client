use super::*;

#[derive(Component, Debug, Reflect, Copy, Clone)]
#[reflect(Component)]
pub struct Attributes {
    strength: i32,
    dexterity: i32,
    intelligence: i32,
    vitality: i32,
    luck: i32,
}
impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            strength: 1,
            dexterity: 1,
            intelligence: 1,
            vitality: 1,
            luck: 1,
        }
    }
}

#[derive(Component, Debug, Reflect, Copy, Clone)]
#[reflect(Component)]
pub struct ComputedAttributes {
    // todo: maybe these shouldn't be pub if there are multiple
    // sources of what affects their value.
    // Example: `attack` might be a combination of strength and dexterity
    pub attack: f32,
    pub defence: f32,
    pub magic_defence: f32,
    pub magic_attack: f32,
    pub hit_rate: f32,
    pub dodge_rate: f32,
    pub crit_rate: f32,
    pub attack_rate: f32,
    pub attack_range: f32,
    pub move_speed: f32,

    pub health: Health,
    pub mana: Mana,
}

impl Default for ComputedAttributes {
    fn default() -> Self {
        ComputedAttributes {
            attack: 1.0,
            defence: 0.0,
            magic_defence: 0.0,
            magic_attack: 1.0,
            hit_rate: 1.0,
            dodge_rate: 0.0,
            crit_rate: 0.0,
            attack_rate: 1.0,
            attack_range: 1.0,
            move_speed: 10.0,

            health: Health::new(1.0),
            mana: Mana::new(1.0),
        }
    }
}

#[derive(Debug, Reflect, Default, Copy, Clone)]
pub struct Health {
    pub max_hp: f32,
    pub hp: f32,
}

impl Health {
    pub fn new(max_hp: f32) -> Self {
        Self {
            max_hp,
            hp: max_hp
        }
    }
}

#[derive(Debug, Reflect, Default, Copy, Clone)]
pub struct Mana {
    pub max_mp: f32,
    pub mp: f32,
}

impl Mana {
    pub fn new(max_mp: f32) -> Self {
        Self {
            max_mp,
            mp: max_mp
        }
    }
}