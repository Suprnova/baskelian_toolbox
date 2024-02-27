use std::{fs::File, io::stdin, path::Path};

extern crate num_derive;

use baskelian_toolbox::dat::{DAT, FileType};
fn main() {
    let mut s = String::new();
    println!("Please enter the path to the DATA.DAT file.");
    stdin().read_line(&mut s).expect("Error parsing user input");
    let input_trimmed = s.trim();
    let dat_file = File::open(input_trimmed).unwrap();
    let dat = DAT::from_file(dat_file).unwrap();
    for (i, inner_dat) in dat.inner_dats.as_slice().iter().enumerate() {
        for (j, entry) in inner_dat.file_table_entries.as_slice().iter().enumerate() {
            let file = dat.read_file(inner_dat, entry).unwrap();
            /*if let FileType::STATS{stats_file} = &file.file_type {
                dbg!(stats_file);
            } else {
                continue;
            }*/
            let file_path = Path::new(&input_trimmed).parent().unwrap().join(format!("{i}-{j}{}", file.file_type));
            std::fs::write(file_path, file.data).unwrap();
            println!("Finished file {i}-{j}{}", file.file_type);
        }
    }
}