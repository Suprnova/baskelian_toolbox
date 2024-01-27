pub mod dat {
    use core::fmt;
    use std::{
        fs::File as ioFile,
        io::{Error, Read, Seek},
        string,
    };

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
        STATS(Stats), // Todo: create a Stats struct that contains the info for a Stats file, and store it in the enum as a value
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
                            Self::STATS(Stats::from_data(data))
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
                Self::STATS(Stats) => write!(f, ".stats"),
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
                    entries.push(entry);
                    line.clear();
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
        pub team: u8,
        pub point_guard: u8,
        pub shooting_guard: u8,
        pub small_forward: u8,
        pub power_forward: u8,
        pub center: u8,
        pub height: u8,
        pub weight: u8,
        pub shoot_max: u8,
        pub shoot_min: u8,
        pub pass_max: u8,
        pub pass_min: u8,
        pub dribble_max: u8,
        pub dribble_min: u8,
        pub power_max: u8,
        pub power_min: u8,
        pub speed_max: u8,
        pub speed_min: u8,
        pub quickness_max: u8,
        pub quickness_min: u8,
        pub jump_max: u8,
        pub jump_min: u8,
        pub stamina_max: u8,
        pub stamina_min: u8,
        pub unknown_1: u8,
        pub unknown_2: u8,
        pub price: u16,
        pub unknown_3: u8,
        pub unknown_4: u8,
        pub unknown_5: u8,
        pub unknown_6: Vec<u8>, // this part is composed of numbers separated by commas; number of numbers is inconsistent and their purpose is unknown
        pub unknown_7: u8,
    }

    impl StatsEntry {
        fn from_data(data: &Vec<u8>) -> Self {
            let mut current_index: usize = 0;
            let mut stats_string: String = String::new();
            let mut stats_vec: Vec<String> = Vec::new();
            let mut unknown_6_string: String = String::new();
            let mut unknown_6_vec: Vec<u8> = Vec::new();
            let mut unknown_6_index: usize = 0;
            while current_index < data.len() {
                if data[current_index] != 0x20 && data[current_index] != 0x2D && data[current_index] != 0x0A {
                    let stats_char: char = data[current_index] as char;
                    stats_string.push(stats_char);
                } else if data[current_index] != 0x0A {
                    stats_vec.push(stats_string.clone());
                    stats_string.clear();
                } else {
                    stats_string = stats_vec[30].clone();
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
                    Self {
                        name: stats_vec[0].clone(),
                        team: stats_vec[1].parse().unwrap(),
                        point_guard: stats_vec[2].parse().unwrap(),
                        shooting_guard: stats_vec[3].parse().unwrap(),
                        small_forward: stats_vec[4].parse().unwrap(),
                        power_forward: stats_vec[5].parse().unwrap(),
                        center: stats_vec[6].parse().unwrap(),
                        height: stats_vec[7].parse().unwrap(),
                        weight: stats_vec[8].parse().unwrap(),
                        shoot_max: stats_vec[9].parse().unwrap(),
                        shoot_min: stats_vec[10].parse().unwrap(),
                        pass_max: stats_vec[11].parse().unwrap(),
                        pass_min: stats_vec[12].parse().unwrap(),
                        dribble_max: stats_vec[13].parse().unwrap(),
                        dribble_min: stats_vec[14].parse().unwrap(),
                        power_max: stats_vec[15].parse().unwrap(),
                        power_min: stats_vec[16].parse().unwrap(),
                        speed_max: stats_vec[17].parse().unwrap(),
                        speed_min: stats_vec[18].parse().unwrap(),
                        quickness_max: stats_vec[19].parse().unwrap(),
                        quickness_min: stats_vec[20].parse().unwrap(),
                        jump_max: stats_vec[21].parse().unwrap(),
                        jump_min: stats_vec[22].parse().unwrap(),
                        stamina_max: stats_vec[23].parse().unwrap(),
                        stamina_min: stats_vec[24].parse().unwrap(),
                        unknown_1: stats_vec[25].parse().unwrap(),
                        unknown_2: stats_vec[26].parse().unwrap(),
                        price: stats_vec[27].parse().unwrap(),
                        unknown_3: stats_vec[28].parse().unwrap(),
                        unknown_4: stats_vec[29].parse().unwrap(),
                        unknown_5: stats_vec[30].parse().unwrap(),
                        unknown_6: unknown_6_vec.clone(),
                        unknown_7: stats_vec[32].parse().unwrap()
                    };
                    stats_vec.clear();
                }
                current_index += 1;
            }
            Default::default()
        }
    }
}
