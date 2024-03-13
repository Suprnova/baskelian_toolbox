#![allow(dead_code)]

use encoding_rs::SHIFT_JIS;
use std::str::FromStr;

use crate::errors::ValidationError;

#[derive(Debug)]
pub struct Put2D {
    pub id: u16,
    pub unknown: u8,
    pub txd_path: String,
    pub txt_path: String,
    pub entry_count: usize,
    pub entries: Vec<Put2DEntry>,
}

impl Put2D {
    pub fn from_data(data: &[u8]) -> Result<Self, ValidationError> {
        let (res, _, errors) = SHIFT_JIS.decode(data);
        if errors {
            return Err(ValidationError::IncorrectFormat(
                "Invalid SHIFT_JIS encoding!".to_string(),
            ));
        }
        let data = res.into_owned();
        let split: Vec<&str> = data.split('\n').collect();

        let entry_count: usize = split[5].parse().unwrap();
        let mut entries: Vec<Put2DEntry> = vec![];
        let mut i: usize = 0;

        while i < entry_count {
            entries.push(split[6 + i].parse()?);
            i += 1;
        }

        Ok(Put2D {
            id: split[1].parse().unwrap(),
            unknown: split[2].parse().unwrap(),
            txd_path: split[3].to_string(),
            txt_path: split[4].to_string(),
            entry_count,
            entries,
        })
    }
}

#[derive(Debug)]
pub struct Put2DEntry {
    unknown: u8,
    entry_type: Put2DEntryType,
}

impl FromStr for Put2DEntry {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(' ').collect();
        Ok(Put2DEntry {
            unknown: split[0].parse().unwrap(),
            entry_type: split[1..split.len()].join(" ").parse()?,
        })
    }
}

#[derive(Debug)]
pub enum Put2DEntryType {
    TYPE0 { entry: Type0 },
    TYPE1 { entry: Type1 },
    TYPE2 { entry: Type2 },
    TYPE3 { entry: Type3 },
    TYPE4 { entry: Type4 },
    UNKNOWN,
}

impl FromStr for Put2DEntryType {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(' ').collect();
        match split[0] {
            "0" => Ok(Self::TYPE0 {
                entry: Type0 {
                    unknown_1: split[1].parse().unwrap(),
                    unknown_2: split[2].parse().unwrap(),
                    content: split[3].to_string(),
                    unknown_3: split[4].parse().unwrap(),
                    unknown_4: split[5].parse().unwrap(),
                    unknown_5: split[6].parse().unwrap(),
                    unknown_6: split[7].parse().unwrap(),
                    unknown_7: split[8].parse().unwrap(),
                    unknown_8: split[9].parse().unwrap(),
                    unknown_9: split[10].parse().unwrap_or_default(),
                },
            }), // to handle an undocumented error in cursor.put2d
            "1" => Ok(Self::TYPE1 {
                entry: Type1 {
                    unknown_1: split[1].parse().unwrap(),
                    file_path: split[2].to_string(),
                    unknown_2: split[3].parse().unwrap(),
                    unknown_3: split[4].parse().unwrap(),
                    unknown_4: split[5].parse().unwrap(),
                    unknown_5: split[6].parse().unwrap(),
                    unknown_6: split[7].parse().unwrap(),
                    unknown_7: split[8].parse().unwrap(),
                    unknown_8: split[9].parse().unwrap(),
                    unknown_9: split[10].parse().unwrap(),
                    id: split[11].parse().unwrap(),
                    file_name: split[12].to_string(),
                },
            }),
            "2" => Ok(Self::TYPE2 {
                entry: Type2 {
                    unknown_1: split[1].parse().unwrap(),
                    file_path: split[2].to_string(),
                    unknown_2: split[3].parse().unwrap(),
                    unknown_3: split[4].parse().unwrap(),
                    unknown_4: split[5].parse().unwrap(),
                    unknown_5: split[6].parse().unwrap(),
                    unknown_6: split[7].parse().unwrap(),
                    unknown_7: split[8].parse().unwrap(),
                    unknown_8: split[9].parse().unwrap(),
                    unknown_9: split[10].parse().unwrap(),
                    unknown_10: split[11].parse().unwrap(),
                    unknown_11: split[12].parse().unwrap(),
                    unknown_12: split[13].parse().unwrap(),
                    unknown_13: split[14].parse().unwrap(),
                    id: split[15].parse().unwrap(),
                    file_name: split[16].to_string(),
                },
            }),
            "3" => Ok(Self::TYPE3 {
                entry: Type3 {
                    unknown_1: split[1].parse().unwrap(),
                    unknown_2: split[2].parse().unwrap(),
                    unknown_3: split[3].parse().unwrap(),
                    unknown_4: split[4].parse().unwrap(),
                    unknown_5: split[5].parse().unwrap(),
                    unknown_6: split[6].parse().unwrap(),
                    unknown_7: split[7].parse().unwrap(),
                    unknown_8: split[8].parse().unwrap(),
                    unknown_9: split[9].parse().unwrap(),
                    unknown_10: split[10].parse().unwrap_or_default(), // to handle an undocumented error in soundtest.put2d
                    unknown_11: split[11].parse().unwrap(),
                    unknown_12: split[12].parse().unwrap(),
                },
            }),
            "4" => Ok(Self::TYPE4 {
                entry: Type4 {
                    unknown_1: split[1].parse().unwrap(),
                    unknown_2: split[2].parse().unwrap(),
                    unknown_3: split[3].parse().unwrap(),
                    unknown_4: split[4].parse().unwrap(),
                    unknown_5: split[5].parse().unwrap(),
                    unknown_6: split[6].parse().unwrap(),
                    unknown_7: split[7].parse().unwrap(),
                    unknown_8: split[8].parse().unwrap(),
                    unknown_9: split[9].parse().unwrap(),
                    unknown_10: split[10].parse().unwrap(),
                    unknown_11: split[11].parse().unwrap(),
                    unknown_12: split[12].parse().unwrap(),
                    unknown_13: split[13].parse().unwrap_or_default(), // to handle an undocumented error in sample_mix.put2d
                    unknown_14: split[14].parse().unwrap(),
                    unknown_15: split[15].parse().unwrap(),
                },
            }),
            _ => Ok(Self::UNKNOWN),
        }
    }
}

#[derive(Debug)]
pub struct Type0 {
    unknown_1: u8,
    unknown_2: u8,
    content: String,
    unknown_3: u16,
    unknown_4: u16,
    unknown_5: u8,
    unknown_6: u8,
    unknown_7: u8,
    unknown_8: u8,
    unknown_9: u8,
}

#[derive(Debug)]
pub struct Type1 {
    unknown_1: u8,
    file_path: String,
    unknown_2: u16,
    unknown_3: u16,
    unknown_4: u16,
    unknown_5: u8,
    unknown_6: u8,
    unknown_7: u8,
    unknown_8: u8,
    unknown_9: u8,
    id: u8,
    file_name: String,
}

#[derive(Debug)]
pub struct Type2 {
    unknown_1: u8,
    file_path: String,
    unknown_2: u16,
    unknown_3: u16,
    unknown_4: u16,
    unknown_5: u16,
    unknown_6: u16,
    unknown_7: u16,
    unknown_8: u16,
    unknown_9: u8,
    unknown_10: u8,
    unknown_11: u8,
    unknown_12: u8,
    unknown_13: u8,
    id: u8,
    file_name: String,
}

#[derive(Debug)]
pub struct Type3 {
    unknown_1: u8,
    unknown_2: u8,
    unknown_3: u8,
    unknown_4: u8,
    unknown_5: u16,
    unknown_6: u16,
    unknown_7: u8,
    unknown_8: u8,
    unknown_9: u8,
    unknown_10: u8,
    unknown_11: u8,
    unknown_12: u8,
}

#[derive(Debug)]
pub struct Type4 {
    unknown_1: u8,
    unknown_2: u16,
    unknown_3: u16,
    unknown_4: u16,
    unknown_5: u16,
    unknown_6: u16,
    unknown_7: u8,
    unknown_8: u8,
    unknown_9: u8,
    unknown_10: u8,
    unknown_11: u8,
    unknown_12: u16,
    unknown_13: u8,
    unknown_14: u8,
    unknown_15: u8,
}
