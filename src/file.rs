pub mod name;
pub mod put2d;
pub mod stats;

use std::{fmt, io::Error};

use crate::dat::{InnerDAT, DAT};

pub struct File {
    pub file_type: FileType,
    pub file_name: Option<String>,
    pub(crate) offset: u32,
    pub(crate) size: u32,
}

impl File {
    pub fn new(
        dat_file: &DAT,
        inner_dat: &InnerDAT,
        entry: [u8; 8],
        current_files: &[File],
    ) -> Self {
        let mut file = Self {
            file_type: FileType::UNKNOWN,
            file_name: None,
            offset: u32::from_le_bytes(entry[0..4].try_into().expect("invalid table entry offset")),
            size: u32::from_le_bytes(entry[4..8].try_into().expect("invalid table entry size")),
        };
        let data = dat_file.read_file(inner_dat, &file).unwrap();
        file.file_type = FileType::from_data(&data);
        file.file_name = match &file.file_type {
            FileType::PUT2D { put2d_script } => Some(
                put2d_script
                    .txd_path
                    .clone()
                    .split('/')
                    .last()
                    .unwrap()
                    .strip_suffix(".txd")
                    .unwrap()
                    .to_string(),
            ),
            FileType::ANM => {
                if current_files.len() > 7 {
                    if let FileType::NAME { name } = &current_files.get(6).unwrap().file_type {
                        Some(name.names.get(current_files.len() - 7).unwrap().clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            FileType::TXD | FileType::UNKNOWN => {
                if !current_files.is_empty() {
                    if let FileType::PUT2D { put2d_script } =
                        &current_files.first().unwrap().file_type
                    {
                        Some(
                            put2d_script
                                .txd_path
                                .clone()
                                .split('/')
                                .last()
                                .unwrap()
                                .strip_suffix(".txd")
                                .unwrap()
                                .to_string(),
                        )
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        file
    }

    pub fn read_file(&self, dat_file: &DAT, inner_dat: &InnerDAT) -> Result<Vec<u8>, Error> {
        dat_file.read_file(inner_dat, self)
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
    NAME {
        name: name::Name,
    },
    /// Portable Network Graphics File
    PNG,
    /// Baskelian MIF PostBL Data Script
    POSTBL,
    /// Baskelian Put2D Script
    PUT2D {
        put2d_script: put2d::Put2D,
    },
    /// Baskelian Stats File
    STATS {
        stats_file: stats::Stats,
    },
    /// RenderWare Texture Dictionary (0x16)
    TXD,
    TXT,
    UNKNOWN,
}

impl FileType {
    fn from_data(data: &[u8]) -> Self {
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
                    if data.len() > 12
                        && data[0..12]
                            == [
                                0x70, 0x75, 0x74, 0x32, 0x64, 0x2D, 0x73, 0x63, 0x72, 0x69, 0x70,
                                0x74,
                            ]
                    {
                        return Self::PUT2D {
                            put2d_script: put2d::Put2D::from_data(data).unwrap(),
                        };
                    }
                    // mif fixed signature
                    if data.len() > 21
                        && data[0..21]
                            == [
                                0x6D, 0x69, 0x66, 0x2D, 0x66, 0x69, 0x78, 0x65, 0x64, 0x2D, 0x64,
                                0x61, 0x74, 0x61, 0x2D, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74,
                            ]
                    {
                        return Self::FIXED;
                    }
                    // mif attached signature
                    if data.len() > 24
                        && data[0..24]
                            == [
                                0x6D, 0x69, 0x66, 0x2D, 0x61, 0x74, 0x74, 0x61, 0x63, 0x68, 0x65,
                                0x64, 0x2D, 0x64, 0x61, 0x74, 0x61, 0x2D, 0x73, 0x63, 0x72, 0x69,
                                0x70, 0x74,
                            ]
                    {
                        return Self::ATTACHED;
                    }
                    // mif postbl signature
                    if data.len() > 22
                        && data[0..22]
                            == [
                                0x6D, 0x69, 0x66, 0x2D, 0x70, 0x6F, 0x73, 0x74, 0x62, 0x6C, 0x2D,
                                0x64, 0x61, 0x74, 0x61, 0x2D, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74,
                            ]
                    {
                        return Self::POSTBL;
                    }
                    // mif comid signature
                    if data.len() > 21
                        && data[0..21]
                            == [
                                0x6D, 0x69, 0x66, 0x2D, 0x63, 0x6F, 0x6D, 0x69, 0x64, 0x2D, 0x64,
                                0x61, 0x74, 0x61, 0x2D, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74,
                            ]
                    {
                        return Self::COMID;
                    }
                    // mif name signature
                    if data.len() > 20
                        && data[0..20]
                            == [
                                0x6D, 0x69, 0x66, 0x2D, 0x6E, 0x61, 0x6D, 0x65, 0x2D, 0x64, 0x61,
                                0x74, 0x61, 0x2D, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74,
                            ]
                    {
                        return Self::NAME {
                            name: name::Name::new(data),
                        };
                    }
                    // check for "font-type" signature
                    if data.len() >= 20
                        && data[0..9] == [0x66, 0x6F, 0x6E, 0x74, 0x2D, 0x74, 0x79, 0x70, 0x65]
                    {
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
                        Self::STATS {
                            stats_file: stats::Stats::from_data(data),
                        }
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
            Self::NAME { .. } => write!(f, ".name"),
            Self::PNG => write!(f, ".png"),
            Self::POSTBL => write!(f, ".postbl"),
            Self::PUT2D { .. } => write!(f, ".put2d"),
            Self::STATS { .. } => write!(f, ".stats"),
            Self::TXD => write!(f, ".txd"),
            Self::TXT => write!(f, ".txt"),
            Self::UNKNOWN => write!(f, ""),
        }
    }
}
