pub mod dat_file {
    use std::{
        fs::File,
        io::{Read, Seek, Error},
    };

    #[derive(Default)]
    pub struct DAT {
        // The file that the DAT originates from
        file: Option<File>,
        /// The number of entries in this file
        entry_count: u32,
        /// The table of each file's info and offset within the file
        pub table_entries: Vec<TableEntry>,
    }

    impl DAT {
        pub fn from_file(file: File) -> Result<Self, Error> {
            let mut dat_struct: Self = Default::default();
            dat_struct.file = Some(file);
            let mut file = dat_struct.file.as_ref().unwrap();
            let mut count_buffer: [u8; 4] = [0; 4];
            file.read_exact(&mut count_buffer)?;
            dat_struct.entry_count = u32::from_le_bytes(count_buffer);
            let mut buffer: [u8; 12] = [255; 12];
            // could simply use entry_count instead of waiting for an empty buffer
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
        /// The memory address within the DAT in which the file appears
        pub address: u32,
        /// The size of the file
        pub size: u32,
        /// The number of entries in the file
        pub entry_count: u32
    }

    impl TableEntry {
        fn from_entry(entry: [u8; 12]) -> Self {
            Self {
                address: u32::from_le_bytes(entry[0..4].try_into().expect("invalid table entry address")),
                size: u32::from_le_bytes(entry[4..8].try_into().expect("invalid table entry size")),
                entry_count: u32::from_le_bytes(entry[8..12].try_into().expect("invalid entry count")),
            }
        }
    }
}