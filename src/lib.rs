pub mod errors;
pub mod put2d;
pub mod stats;

pub mod dat {
    use core::fmt;
    use std::{
        fs::File as ioFile,
        io::{Error, Read, Seek}
    };

    pub use crate::{errors, put2d, stats};

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
        pub file_name: Option<String>
    }

    impl File {
        fn new(data: Vec<u8>) -> Self {
            let file_type = FileType::from_data(&data);
            let file_name = match &file_type {
                FileType::PUT2D{put2d_script} => {
                    Some(put2d_script.txd_path.clone().split('/').last().unwrap().strip_suffix(".txd").unwrap().to_string())
                }
                _ => { None }
            };
            Self {
                file_type,
                data,
                file_name
            }
        }
    }

    pub enum FileType {
        /// RenderWare Anim Animation File (0x1B)
        ANM,
        /// Baskelian MIF Attached Data Script
        ATTACHED,
        /// Baskelian MIF Comid Data Script
        COMID,
        /// RenderWare Delta Morph Animation File (0x1E)
        DMA,
        /// RenderWare Model File (Clump) (0x10)
        DFF,
        /// Baskelian MIF Fixed Data Script
        FIXED,
        /// Baskelian Font Type Information
        FTI,
        /// Baskelian Map Info File
        MAPINFO,
        /// Baskelian Name Data Script
        NAME,
        /// Portable Network Graphics File
        PNG,
        /// Baskelian MIF PostBL Data Script
        POSTBL,
        /// Baskelian Put2D Script
        PUT2D{put2d_script: put2d::Put2D},
        /// Baskelian Stats File
        STATS{stats_file: stats::Stats},
        /// RenderWare Texture Dictionary (0x16)
        TXD,
        TXT,
        UNKNOWN,
    }

    impl FileType {
        fn from_data(data: &Vec<u8>) -> Self {
            if data.is_empty() {
                Self::UNKNOWN
            } else {
                // TODO: Make this code cleaner
                match data[0] {
                    0x10 => Self::DFF,
                    0x16 => Self::TXD,
                    0x1B => Self::ANM,
                    0x1E => Self::DMA,
                    _ => {
                        // check for PNG signature
                        if data.len() >= 4 && data[1..4] == [0x50, 0x4E, 0x47] {
                            return Self::PNG;
                        }
                        // put2d signature
                        if data.len() > 12 && data[0..12] == [0x70, 0x75, 0x74, 0x32, 0x64, 0x2D, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74] {
                            return Self::PUT2D { put2d_script: put2d::Put2D::from_data(data).unwrap()}
                        }
                        // mif fixed signature
                        if data.len() > 21 && data[0..21] == [
                            0x6D, 0x69, 0x66, 0x2D, 0x66, 0x69, 0x78, 0x65, 0x64, 0x2D, 0x64, 0x61, 0x74, 0x61, 0x2D, 0x73,
                            0x63, 0x72, 0x69, 0x70, 0x74] {
                            return Self::FIXED;
                        }
                        // mif attached signature
                        if data.len() > 24 && data[0..24] == [
                            0x6D, 0x69, 0x66, 0x2D, 0x61, 0x74, 0x74, 0x61, 0x63, 0x68, 0x65, 0x64, 0x2D, 0x64, 0x61, 0x74, 
                            0x61, 0x2D, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74] {
                            return Self::ATTACHED;
                        }
                        // mif postbl signature
                        if data.len() > 22 && data[0..22] == [
                            0x6D, 0x69, 0x66, 0x2D, 0x70, 0x6F, 0x73, 0x74, 0x62, 0x6C, 0x2D, 0x64, 0x61, 0x74, 0x61, 0x2D, 
                            0x73, 0x63, 0x72, 0x69, 0x70, 0x74] {
                            return Self::POSTBL;
                        }
                        // mif comid signature
                        if data.len() > 21 && data[0..21] == [
                            0x6D, 0x69, 0x66, 0x2D, 0x63, 0x6F, 0x6D, 0x69, 0x64, 0x2D, 0x64, 0x61, 0x74, 0x61, 0x2D, 0x73, 
                            0x63, 0x72, 0x69, 0x70, 0x74] {
                            return Self::COMID;
                        }
                        // mif name signature
                        if data.len() > 20 && data[0..20] == [
                            0x6D, 0x69, 0x66, 0x2D, 0x6E, 0x61, 0x6D, 0x65, 0x2D, 0x64, 0x61, 0x74, 0x61, 0x2D, 0x73, 0x63, 
                            0x72, 0x69, 0x70, 0x74] {
                            return Self::NAME;
                        }
                        // check for "font-type" signature
                        if data.len() >= 20 && data[0..9] == [0x66, 0x6F, 0x6E, 0x74, 0x2D, 0x74, 0x79, 0x70, 0x65] {
                            return Self::FTI;
                        }
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
                            Self::STATS{stats_file: stats::Stats::from_data(data)}
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
                Self::ATTACHED => write!(f, ".cv3"),
                Self::COMID => write!(f, ".comid"),
                Self::DMA => write!(f, ".dma"),
                Self::DFF => write!(f, ".dff"),
                Self::FIXED => write!(f, ".fix"),
                Self::FTI => write!(f, ".fti"),
                Self::MAPINFO => write!(f, ".mapinfo"),
                Self::NAME => write!(f, ".name"),
                Self::PNG => write!(f, ".png"),
                Self::POSTBL => write!(f, ".postbl"),
                Self::PUT2D{..} => write!(f, ".put2d"),
                Self::STATS{..} => write!(f, ".stats"),
                Self::TXD => write!(f, ".txd"),
                Self::TXT => write!(f, ".txt"),
                Self::UNKNOWN => write!(f, ""),
            }
        }
    }
}