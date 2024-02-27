pub mod dat {
    use core::fmt;
    use std::{
        fs::File as ioFile,
        io::{Error, Read, Seek},
        str::FromStr
    };

    use num_derive::FromPrimitive;
    use thiserror::Error;

    // this could be rewritten to have a DAT as the main file, and files within that DAT are also DATs, only actually extracting the files
    // in the subsequent DAT, but for now we will use InnerDAT
    // in general everything here is bad and should be rewritten this is just a bodge to match the file specifications
    #[derive(Default)]
    pub struct DAT {
        /// The file that the DAT originates from
        file: Option<ioFile>,
        /// The number of entries in this file
        entry_count: u32,
        /// The table of each file's info and offset within the file
        table_entries: Vec<TableEntry>,
        /// The inner DAT files of the main DAT
        pub inner_dats: Vec<InnerDAT>,
    }

    impl DAT {
        pub fn from_file(file: ioFile) -> Result<Self, Error> {
            let mut dat_struct: Self = Default::default();
            dat_struct.file = Some(file);
            let mut file = dat_struct.file.as_ref().unwrap();
            let mut count_buffer: [u8; 4] = [0; 4];
            file.read_exact(&mut count_buffer)?;
            dat_struct.entry_count = u32::from_le_bytes(count_buffer);
            let mut buffer: [u8; 12] = [255; 12];
            let mut i = 0;
            while i < dat_struct.entry_count {
                // this throws an error if even one table is messed up, we could be more lenient with something like that
                file.read_exact(&mut buffer)?;
                let entry = TableEntry::from_entry(buffer);
                dat_struct.table_entries.push(entry);
                i += 1;
            }
            for entry in dat_struct.table_entries.iter() {
                dat_struct
                    .inner_dats
                    .push(dat_struct.read_entry(&entry).unwrap());
            }
            Ok(dat_struct)
        }

        pub fn read_entry(&self, entry: &TableEntry) -> Result<InnerDAT, Error> {
            let mut inner_struct: InnerDAT = Default::default();
            inner_struct.offset = entry.address;
            let mut file = self.file.as_ref().unwrap();
            file.seek(std::io::SeekFrom::Start(entry.address.into()))?;
            let mut count_buffer: [u8; 4] = [0; 4];
            file.read_exact(&mut count_buffer)?;
            inner_struct.entry_count = u32::from_le_bytes(count_buffer);
            let mut buffer: [u8; 8] = [255; 8];
            let mut i = 0;
            while i < inner_struct.entry_count {
                file.read_exact(&mut buffer)?;
                inner_struct
                    .file_table_entries
                    .push(FileTableEntry::from_file_entry(buffer));
                i += 1;
            }
            Ok(inner_struct)
        }

        pub fn read_file(
            &self,
            inner_dat: &InnerDAT,
            entry: &FileTableEntry,
        ) -> Result<File, Error> {
            let mut file = self.file.as_ref().unwrap();
            file.seek(std::io::SeekFrom::Start(
                (inner_dat.offset + entry.address).into(),
            ))?;
            let mut buffer: Vec<u8> = vec![0; entry.size.try_into().unwrap()];
            file.read_exact(&mut buffer)?;
            Ok(File::new(buffer))
        }
    }

    #[derive(Default)]
    pub struct InnerDAT {
        offset: u32,
        entry_count: u32,
        pub file_table_entries: Vec<FileTableEntry>,
    }

    pub struct TableEntry {
        /// The memory address within the main DAT in which the file appears
        pub address: u32,
        /// The size of the file
        pub size: u32,
        /// The number of entries in the file
        pub entry_count: u32,
    }

    impl TableEntry {
        fn from_entry(entry: [u8; 12]) -> Self {
            Self {
                address: u32::from_le_bytes(
                    entry[0..4].try_into().expect("invalid table entry address"),
                ),
                size: u32::from_le_bytes(entry[4..8].try_into().expect("invalid table entry size")),
                entry_count: u32::from_le_bytes(
                    entry[8..12].try_into().expect("invalid table entry count"),
                ),
            }
        }
    }

    pub struct FileTableEntry {
        /// The memory address within the parent DAT in which the file appears
        pub address: u32,
        /// The size of the file
        pub size: u32,
    }

    impl FileTableEntry {
        fn from_file_entry(entry: [u8; 8]) -> Self {
            Self {
                address: u32::from_le_bytes(
                    entry[0..4].try_into().expect("invalid table entry address"),
                ),
                size: u32::from_le_bytes(entry[4..8].try_into().expect("invalid table entry size")),
            }
        }
    }

    // Todo: move all of this into a file mod, containing the structs for necessary file types (i.e. Stats) and associated functions
    pub struct File {
        pub data: Vec<u8>,
        pub file_type: FileType,
    }

    impl File {
        fn new(data: Vec<u8>) -> Self {
            Self {
                file_type: FileType::from_data(&data),
                data,
            }
        }
    }

    pub enum FileType {
        /// RenderWare Anim Animation File (0x1B)
        ANM,
        /// RenderWare Delta Morph Animation File (0x1E)
        DMA,
        /// RenderWare Model File (Clump) (0x10)
        DFF,
        /// Baskelian Map Info File
        MAPINFO,
        /// Baskelian Stats File
        STATS{stats_file: Stats}, // Todo: create a Stats struct that contains the info for a Stats file, and store it in the enum as a value
        /// RenderWare Texture Dictionary (0x16)
        TXD,
        UNKNOWN,
    }

    impl FileType {
        fn from_data(data: &Vec<u8>) -> Self {
            if data.len() == 0 {
                Self::UNKNOWN
            } else {
                match data[0] {
                    0x10 => Self::DFF,
                    0x16 => Self::TXD,
                    0x1B => Self::ANM,
                    0x1E => Self::DMA,
                    _ => {
                        let mut current_index: usize = 0;
                        let mut space_count: u8 = 0;
                        while current_index < 150
                            && current_index < data.len()
                            && data[current_index] != 0x0A
                        {
                            if data[current_index] == 0x20 {
                                space_count += 1;
                            }
                            current_index += 1;
                        }
                        if space_count == 20 {
                            Self::STATS{ stats_file: Stats::from_data(data)}
                        } else {
                            let mut current_index: usize = 12;
                            let mut buffer_count: u8 = 0;
                            while current_index < 32
                                && current_index < data.len()
                                && data[current_index] == 0xFF
                            {
                                buffer_count += 1;
                                current_index += 1;
                            }
                            if buffer_count == 12 && data.len() > 29 && data[8] == data[28] {
                                // data [8] and data[28] are expected to be the Map ID value of the map that the MAPINFO file is referencing
                                Self::MAPINFO
                            } else {
                                Self::UNKNOWN
                            }
                        }
                    }
                }
            }
        }
    }

    impl fmt::Display for FileType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Self::ANM => write!(f, ".anm"),
                Self::DMA => write!(f, ".dma"),
                Self::DFF => write!(f, ".dff"),
                Self::MAPINFO => write!(f, ".mapinfo"),
                Self::STATS{..} => write!(f, ".stats"),
                Self::TXD => write!(f, ".txd"),
                Self::UNKNOWN => write!(f, ""),
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct Stats {
        pub entries: Vec<StatsEntry>,
    }

    impl Stats {
        fn from_data(data: &Vec<u8>) -> Self {
            let mut current_index: usize = 0;
            let mut line: Vec<u8> = Vec::new();
            let mut entries: Vec<StatsEntry> = Vec::new();
            while current_index < data.len() {
                if data[current_index] != 0x0A { // all stats entries, including the final entry, end in a new line char (0x0A)
                    line.push(data[current_index]);
                } else {
                    let entry = StatsEntry::from_data(&line);
                    match entry {
                        Ok(entry) => {
                            entries.push(entry);
                            line.clear();
                        },
                        Err(e) => {
                            println!("error parsing entry at index {current_index}: \n{e}\nskipping...");
                        }
                    }
                }
                current_index += 1;
            }
            Self {
                entries
            }
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
        fn from_data(data: &Vec<u8>) -> Result<Self, ValidationError> {
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
                unknown_6_len: stats_vec[20].parse().unwrap()
            })
        }
    }

    #[derive(Debug, Default)]
    pub struct SkillRange {
        pub initial_value: u8,
        pub max_value: u8
    }

    impl FromStr for SkillRange {
        type Err = ValidationError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let range: Vec<u8> = s
                .split('-')
                .map(|s| s.parse::<u8>().unwrap())
                .collect();

            if range.len() > 2 || range.len() < 1 {
                return Err(ValidationError::IncorrectFormat("skill range".to_string()))
            }
            Ok(Self {
                initial_value: range[range.len()-1],
                max_value: range[0]
            })
        }
    }

    #[derive(Debug, Default)]
    pub struct PositionGrades {
        pub point_guard: u8,
        pub shooting_guard: u8,
        pub small_forward: u8,
        pub power_forward: u8,
        pub center: u8
    }

    impl FromStr for PositionGrades {
        type Err = ValidationError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let grades: Vec<u8> = s
                .split('-')
                .map(|s| s.parse::<u8>().unwrap())
                .collect();

            if grades.len() != 5 {
                return Err(ValidationError::IncorrectFormat("position grades".to_string()));
            }
            Ok(Self {
                point_guard: grades[0],
                shooting_guard: grades[1],
                small_forward: grades[2],
                power_forward: grades[3],
                center: grades[4]
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
        Shadow = 13
    }

    impl Team {
        pub fn from_id(id: u8) -> Result<Self, ValidationError> {
            let team = num::FromPrimitive::from_u8(id);
            match team {
                Some(validteam) => Ok(validteam),
                None => Err(ValidationError::OutOfRange("team".to_string(), id))
            }
        }
    }

    #[derive(Error, Debug)]
    pub enum ValidationError {
        #[error("field `{0}` is in an invalid format")]
        IncorrectFormat(String),
        #[error("field `{0}` is missing")]
        MissingField(String),
        #[error("field `{0}`'s value of `{1}` is out of range")]
        OutOfRange(String, u8),
        #[error("unknown parsing error")]
        ParseFailure
    }
}