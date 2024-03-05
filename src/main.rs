use std::{fs::File, io::stdin, path::Path};

extern crate num_derive;
extern crate encoding_rs;


use baskelian_toolbox::dat::{DAT, FileType};
fn main() {
    let mut s = String::new();
    println!("Please enter the path to the DATA.DAT file.");
    stdin().read_line(&mut s).expect("Error parsing user input");
    let input_trimmed = s.trim();
    let dat_file = File::open(input_trimmed).unwrap();
    let dat = DAT::from_file(dat_file).unwrap();
    for (i, inner_dat) in dat.inner_dats.as_slice().iter().enumerate() {
        let mut archive_name: Option<String> = None;
        for (j, entry) in inner_dat.file_table_entries.as_slice().iter().enumerate() {
            let file = dat.read_file(inner_dat, entry).unwrap();
            /*if let FileType::PUT2D{put2d_script} = &file.file_type {
                //dbg!(put2d_script);
            } else if archive_name.is_none() {
                continue;
            }*/
            let mut file_path = Path::new(&input_trimmed).parent().unwrap().join(format!("{i}-{j}{}", file.file_type));
            if file.file_name.is_some() {
                archive_name = file.file_name;
            }
            if archive_name.is_some() {
                file_path = Path::new(&input_trimmed).parent().unwrap().join(format!("{}{}", archive_name.clone().unwrap(), file.file_type));
            }
            std::fs::write(file_path.clone(), file.data).unwrap();
            println!("Finished file {}", file_path.to_str().unwrap().split("\\").last().unwrap());
        }
    }
}