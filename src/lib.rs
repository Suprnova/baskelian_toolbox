pub mod dat_file {
    use std::{
        fs::File,
        io::{Read, Seek, Error},
    };

    #[derive(Default)]
    pub struct DAT {
        // The file that the DAT originates from
        file: Option<File>,
        /// The first byte of the file, unknown meaning, likely a magic byte
        unknown1: [u8; 1],
        /// The table of each file's info and offset within the file
        pub table_entries: Vec<TableEntry>,
    }

    impl DAT {
        pub fn from_file(file: File) -> Result<Self, Error> {
            let mut dat_struct: Self = Default::default();
            dat_struct.file = Some(file);
            let mut file = dat_struct.file.as_ref().unwrap();
            file.read_exact(&mut dat_struct.unknown1)?;
            let mut buffer: [u8; 12] = [255; 12];
            while buffer != [0; 12] {
                // this throws an error if even one table is messed up, we could be more lenient with something like that
                file.read_exact(&mut buffer)?;
                dat_struct.table_entries.push(TableEntry::from_entry(buffer));
            }
            Ok(dat_struct)
        }

        pub fn read_entry(&self, entry: &TableEntry) -> Result<Vec<u8>, Error> {
            let mut file = self.file.as_ref().unwrap();
            file.seek(std::io::SeekFrom::Start(entry.address.into()))?;
            // this will fucking panic i need to rewrite this :3
            let mut buffer: Vec<u8> = vec![0; entry.size.try_into().unwrap()];
            file.read_exact(&mut buffer)?;
            Ok(buffer)
        }
    }

    #[allow(dead_code)]
    pub struct TableEntry {
        /// Unknown
        unknown1: u8,
        /// Unknown
        unknown2: u8,
        /// Unknown
        unknown3: u8,
        /// The memory address within the DAT in which the file appears
        pub address: u32,
        /// The size of the file
        pub size: u32,
        /// The first byte of the file, likely the magic byte, used to identify the file type?
        pub first_byte: u8,
    }

    impl TableEntry {
        fn from_entry(entry: [u8; 12]) -> Self {
            Self {
                unknown1: entry[0],
                unknown2: entry[1],
                unknown3: entry[2],
                address: u32::from_le_bytes(entry[3..7].try_into().expect("invalid table entry address")),
                size: u32::from_le_bytes(entry[7..11].try_into().expect("invalid table entry size")),
                first_byte: entry[11],
            }
        }
    }
}