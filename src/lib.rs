pub mod errors;
pub mod file;
extern crate encoding_rs;
extern crate num_derive;

pub mod dat {
    use std::{
        fs::File as ioFile,
        io::{Error, Read, Seek},
    };

    use crate::file::{File, FileType};
    pub use crate::{
        errors,
        file::{put2d, stats},
    };

    pub struct DAT {
        /// The file that the DAT originates from
        file: ioFile,
        /// The inner DAT files of the main DAT
        pub inner_dats: Vec<InnerDAT>,
    }

    impl DAT {
        pub fn from_file(mut file: ioFile) -> Result<Self, Error> {
            let inner_dats: Vec<InnerDAT> = vec![];
            let mut buf: [u8; 4] = [0; 4];
            file.read_exact(&mut buf)?;
            let entry_count = u32::from_le_bytes(buf);
            let mut buf: [u8; 12] = [255; 12];
            let mut i = 0;
            let mut dat = Self { file, inner_dats };
            while i < entry_count {
                // this throws an error if even one table is messed up, we could be more lenient with something like that
                dat.file.read_exact(&mut buf)?;
                let head = dat.file.stream_position().unwrap();
                let entry = InnerDAT::new(&dat, buf);
                dat.file.seek(std::io::SeekFrom::Start(head))?;
                dat.inner_dats.push(entry);
                i += 1;
            }
            Ok(dat)
        }

        fn index_files(&self, inner_dat: &InnerDAT) -> Result<Vec<File>, Error> {
            let offset = inner_dat.offset;
            let mut file = &self.file;
            file.seek(std::io::SeekFrom::Start(offset.into()))?;
            let mut count_buffer: [u8; 4] = [0; 4];
            file.read_exact(&mut count_buffer)?;
            let entry_count = u32::from_le_bytes(count_buffer);
            let mut buffer: [u8; 8] = [255; 8];
            let mut i = 0;
            let mut files: Vec<File> = vec![];
            while i < entry_count {
                file.read_exact(&mut buffer)?;
                let head = file.stream_position().unwrap();
                files.push(File::new(self, inner_dat, buffer, &files));
                file.seek(std::io::SeekFrom::Start(head))?;
                i += 1;
            }
            Ok(files)
        }

        pub fn read_file(&self, inner_dat: &InnerDAT, file: &File) -> Result<Vec<u8>, Error> {
            let mut io_file = &self.file;
            io_file.seek(std::io::SeekFrom::Start(
                (inner_dat.offset + file.offset).into(),
            ))?;
            let mut buffer: Vec<u8> = vec![0; file.size.try_into().unwrap()];
            io_file.read_exact(&mut buffer)?;
            Ok(buffer)
        }
    }

    pub struct InnerDAT {
        offset: u32,
        size: u32,
        entry_count: u32,
        pub archive_name: Option<String>,
        pub archive_type: ArchiveType,
        pub files: Vec<File>,
    }

    impl InnerDAT {
        pub fn new(dat_file: &DAT, entry: [u8; 12]) -> Self {
            let files: Vec<File> = vec![];
            let mut inner_dat = Self {
                offset: u32::from_le_bytes(
                    entry[0..4].try_into().expect("invalid table entry address"),
                ),
                size: u32::from_le_bytes(entry[4..8].try_into().expect("invalid table entry size")),
                entry_count: u32::from_le_bytes(
                    entry[8..12].try_into().expect("invalid table entry count"),
                ),
                archive_name: None,
                archive_type: ArchiveType::UNKNOWN,
                files,
            };
            inner_dat.files = dat_file.index_files(&inner_dat).unwrap();
            inner_dat.archive_type = ArchiveType::from_archive(&inner_dat);
            inner_dat.archive_name = inner_dat.get_names();
            // Todo: Add enum to describe archive type (put2d, character, object, etc)
            // Also add name handling
            inner_dat
        }

        fn get_names(&mut self) -> Option<String> {
            match self.archive_type {
                ArchiveType::CHARACTER => {
                    let file = &self.files[6];
                    if let FileType::NAME { name } = &file.file_type {
                        Some(
                            name.file_path
                                .split('/')
                                .nth(4)
                                .unwrap()
                                .strip_suffix(".name.out")
                                .unwrap()
                                .to_string(),
                        )
                    } else {
                        None
                    }
                }
                ArchiveType::UI => {
                    let file = &self.files.first().unwrap();
                    if let FileType::PUT2D { put2d_script } = &file.file_type {
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
                }
                _ => None,
            }
        }
    }

    pub enum ArchiveType {
        CHARACTER,
        OBJECT,
        UI,
        UNKNOWN,
    }

    impl ArchiveType {
        pub fn from_archive(inner: &InnerDAT) -> Self {
            match inner.files.first().unwrap().file_type {
                FileType::DFF => {
                    if inner.entry_count > 7
                        && matches!(inner.files[6].file_type, FileType::NAME { .. })
                    {
                        ArchiveType::CHARACTER
                    } else if inner.entry_count >= 2
                        && matches!(inner.files[1].file_type, FileType::TXD)
                    {
                        ArchiveType::OBJECT
                    } else {
                        ArchiveType::UNKNOWN
                    }
                }
                FileType::PUT2D { .. } => ArchiveType::UI,
                FileType::UNKNOWN => {
                    if inner.entry_count > 7
                        && matches!(inner.files[6].file_type, FileType::NAME { .. })
                    {
                        ArchiveType::CHARACTER
                    } else {
                        ArchiveType::UNKNOWN
                    }
                }
                _ => ArchiveType::UNKNOWN,
            }
        }
    }
}
