use std::str::FromStr;
use num_derive::FromPrimitive;

use crate::errors::ValidationError;

#[derive(Debug, Default)]
pub struct Stats {
    pub entries: Vec<StatsEntry>,
}

impl Stats {
    pub(crate) fn from_data(data: &Vec<u8>) -> Self {
        let mut current_index: usize = 0;
        let mut line: Vec<u8> = Vec::new();
        let mut entries: Vec<StatsEntry> = Vec::new();
        while current_index < data.len() {
            if data[current_index] != 0x0A {
                // all stats entries, including the final entry, end in a new line char (0x0A)
                line.push(data[current_index]);
            } else {
                let entry = StatsEntry::from_data(&line);
                match entry {
                    Ok(entry) => {
                        entries.push(entry);
                        line.clear();
                    }
                    Err(e) => {
                        println!(
                            "error parsing entry at index {current_index}: \n{e}\nskipping..."
                        );
                    }
                }
            }
            current_index += 1;
        }
        Self { entries }
    }
}

#[derive(Debug, Default)]
pub struct StatsEntry {
    pub name: String,
    pub team: Team,
    pub grades: PositionGrades,
    pub height: u16,
    pub weight: u16,
    pub shoot: SkillRange,
    pub pass: SkillRange,
    pub dribble: SkillRange,
    pub power: SkillRange,
    pub speed: SkillRange,
    pub quickness: SkillRange,
    pub jump: SkillRange,
    pub stamina: SkillRange,
    pub unknown_1: u8,
    pub unknown_2: u8,
    pub price: u32,
    pub unknown_3: u8,
    pub unknown_4: u8,
    pub unknown_5: u8,
    pub unknown_6: Vec<u8>, // this part is composed of numbers separated by commas; number of numbers is inconsistent and their purpose is unknown
    pub unknown_6_len: u8,
}

impl StatsEntry {
    pub(crate) fn from_data(data: &Vec<u8>) -> Result<Self, ValidationError> {
        let mut current_index: usize = 0;
        let mut stats_string: String = String::new();
        let mut stats_vec: Vec<String> = Vec::new();
        let mut unknown_6_string: String = String::new();
        let mut unknown_6_vec: Vec<u8> = Vec::new();
        let mut unknown_6_index: usize;
        while current_index < data.len() {
            // TODO: this can be implemented as a split now
            if data[current_index] != 0x20 {
                let stats_char: char = data[current_index] as char;
                stats_string.push(stats_char);
            } else {
                stats_vec.push(stats_string.clone());
                stats_string.clear();
            }
            current_index += 1;
        }
        stats_vec.push(stats_string.clone());
        //if stats_vec.len() < 31 { // some entries have no max/min, resulting in errors with the current StatsEntry implementation. these are being defaulted to blank entries for the time being
        //    return Default::default()
        //}
        let unknown_6_location: usize = stats_vec.len() - 2;
        stats_string = stats_vec[unknown_6_location].clone();
        unknown_6_index = 0;
        while unknown_6_index < stats_string.len() {
            if stats_string.as_bytes()[unknown_6_index] != 0x2C {
                unknown_6_string.push(stats_string.as_bytes()[unknown_6_index] as char);
            } else {
                unknown_6_vec.push(unknown_6_string.parse().unwrap());
                unknown_6_string.clear();
            }
            unknown_6_index += 1;
        }
        Ok(Self {
            name: stats_vec[0].clone(),
            team: Team::from_id(stats_vec[1].parse().unwrap())?,
            // TODO: implement handling for this
            grades: stats_vec[2].parse()?,
            height: stats_vec[3].parse().unwrap(),
            weight: stats_vec[4].parse().unwrap(),
            shoot: stats_vec[5].parse()?,
            pass: stats_vec[6].parse()?,
            dribble: stats_vec[7].parse()?,
            power: stats_vec[8].parse()?,
            speed: stats_vec[9].parse()?,
            quickness: stats_vec[10].parse()?,
            jump: stats_vec[11].parse()?,
            stamina: stats_vec[12].parse()?,
            unknown_1: stats_vec[13].parse().unwrap(),
            unknown_2: stats_vec[14].parse().unwrap(),
            price: stats_vec[15].parse().unwrap(),
            unknown_3: stats_vec[16].parse().unwrap(),
            unknown_4: stats_vec[17].parse().unwrap(),
            unknown_5: stats_vec[18].parse().unwrap(),
            unknown_6: unknown_6_vec.clone(),
            unknown_6_len: stats_vec[20].parse().unwrap(),
        })
    }
}

#[derive(Debug, Default)]
pub struct SkillRange {
    pub initial_value: u8,
    pub max_value: u8,
}

impl FromStr for SkillRange {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let range: Vec<u8> = s.split('-').map(|s| s.parse::<u8>().unwrap()).collect();

        if range.len() > 2 || range.is_empty() {
            return Err(ValidationError::IncorrectFormat("skill range".to_string()));
        }
        Ok(Self {
            initial_value: range[range.len() - 1],
            max_value: range[0],
        })
    }
}

#[derive(Debug, Default)]
pub struct PositionGrades {
    pub point_guard: u8,
    pub shooting_guard: u8,
    pub small_forward: u8,
    pub power_forward: u8,
    pub center: u8,
}

impl FromStr for PositionGrades {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let grades: Vec<u8> = s.split('-').map(|s| s.parse::<u8>().unwrap()).collect();

        if grades.len() != 5 {
            return Err(ValidationError::IncorrectFormat(
                "position grades".to_string(),
            ));
        }

        Ok(Self {
            point_guard: grades[0],
            shooting_guard: grades[1],
            small_forward: grades[2],
            power_forward: grades[3],
            center: grades[4],
        })
    }
}

#[derive(Debug, Default, FromPrimitive)]
pub enum Team {
    Gerbils = 0,
    Dwolf = 1,
    Roosters = 2,
    Puppies = 3,
    Durock = 4,
    Lambs = 5,
    Trotters = 6,
    Linx = 7,
    Apes = 8,
    Boas = 9,
    // ??????
    CattlesMutilates = 10,
    BSKTiamats = 11,
    #[default]
    FreeAgent = 12,
    Shadow = 13,
}

impl Team {
    pub fn from_id(id: u8) -> Result<Self, ValidationError> {
        let team = num::FromPrimitive::from_u8(id);
        match team {
            Some(validteam) => Ok(validteam),
            None => Err(ValidationError::OutOfRange("team".to_string(), id)),
        }
    }
}
