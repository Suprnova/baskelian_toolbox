pub mod dat_file {
    use std::{
        fs::File,
        io::{Read, Seek, Error},
    };

    // this could be rewritten to have a DAT as the main file, and files within that DAT are also DATs, only actually extracting the files
    // in the subsequent DAT, but for now we will use InnerDAT
    // in general everything here is bad and should be rewritten this is just a bodge to match the file specifications
    #[derive(Default)]
    pub struct DAT {
        /// The file that the DAT originates from
        file: Option<File>,
        /// The number of entries in this file
        entry_count: u32,
        /// The table of each file's info and offset within the file
        table_entries: Vec<TableEntry>,
        /// The inner DAT files of the main DAT
        pub inner_dats: Vec<InnerDAT>,
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
            let mut i = 0;
            while i < dat_struct.entry_count {
                // this throws an error if even one table is messed up, we could be more lenient with something like that
                file.read_exact(&mut buffer)?;
                let entry = TableEntry::from_entry(buffer);
                dat_struct.table_entries.push(entry);
                i = i + 1;
            }
            for entry in dat_struct.table_entries.iter() {
                dat_struct.inner_dats.push(dat_struct.read_entry(&entry).unwrap());
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
            while buffer != [0; 8] {
                file.read_exact(&mut buffer)?;
                inner_struct.file_table_entries.push(FileTableEntry::from_file_entry(buffer));
            }
            Ok(inner_struct)
        }

        pub fn read_file(&self, inner_dat: &InnerDAT, entry: &FileTableEntry) -> Result<Vec<u8>, Error> {
            let mut file = self.file.as_ref().unwrap();
            file.seek(std::io::SeekFrom::Start((inner_dat.offset + entry.address).into()))?;
            let mut buffer: Vec<u8> = vec![0; entry.size.try_into().unwrap()];
            file.read_exact(&mut buffer)?;
            Ok(buffer)
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
        pub entry_count: u32
    }

    impl TableEntry {
        fn from_entry(entry: [u8; 12]) -> Self {
            Self {
                address: u32::from_le_bytes(entry[0..4].try_into().expect("invalid table entry address")),
                size: u32::from_le_bytes(entry[4..8].try_into().expect("invalid table entry size")),
                entry_count: u32::from_le_bytes(entry[8..12].try_into().expect("invalid table entry count")),
            }
        }
    }

    pub struct FileTableEntry {
        /// The memory address within the parent DAT in which the file appears
        pub address: u32,
        /// The size of the file
        pub size: u32
    }

    impl FileTableEntry {
        fn from_file_entry(entry: [u8; 8]) -> Self {
            Self {
                address: u32::from_le_bytes(entry[0..4].try_into().expect("invalid table entry address")),
                size: u32::from_le_bytes(entry[4..8].try_into().expect("invalid table entry size")),
            }
        }
    }
}