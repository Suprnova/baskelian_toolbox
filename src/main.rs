use std::{fs::{create_dir, File}, path::Path};

extern crate num_derive;
extern crate encoding_rs;


use baskelian_toolbox::dat::{DAT, FileType};
fn main() {
    let dat_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("_artifacts/DATA.DAT");
    let extract_err = create_dir(dat_path.parent().unwrap().join("extracted")).err();
    if extract_err.is_some() {
        if !Path::new(&dat_path.parent().unwrap().join("extracted")).exists() {
            panic!("unknown directory error")
        }
    }
    let dat_file = File::open(&dat_path).expect("DATA.DAT file not found in _artifacts folder!");
    let dat = DAT::from_file(dat_file).unwrap();
    for (i, inner_dat) in dat.inner_dats.as_slice().iter().enumerate() {
        let mut archive_name: Option<String> = None;
        for (j, entry) in inner_dat.file_table_entries.as_slice().iter().enumerate() {
            let mut file = dat.read_file(inner_dat, entry).unwrap();
            let mut file_path = Path::new(&dat_path).parent().unwrap().join(format!("extracted/{i}-{j}{}", file.file_type));
            // file has file name, so all files in this archive have same file name
            // (temporary for put2d scripts, should be implemented library-side)
            if file.file_name.is_some() {
                archive_name = file.file_name;
            }
            if archive_name.is_some() {
                // an unknown file in a known archive_name must be a .txt
                // (temporary for put2d scripts)
                if matches!(file.file_type, FileType::UNKNOWN) {
                    file.file_type = FileType::TXT
                }
                file_path = Path::new(&file_path).parent().unwrap().join(format!("{}{}", archive_name.clone().unwrap(), file.file_type));
            }
            std::fs::write(file_path.clone(), file.data).unwrap();
            println!("Finished file {}", file_path.to_str().unwrap().split("\\").last().unwrap());
        }
    }
}