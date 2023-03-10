use std::{fs::File, io::stdin, path::Path};

use baskelian_toolbox::dat_file::DAT;
fn main() {
    let mut s = String::new();
    println!("Please enter the path to the DATA.DAT file.");
    stdin().read_line(&mut s).expect("Error parsing user input");
    let input_trimmed = s.trim();
    let file = File::open(input_trimmed).unwrap();
    let dat = DAT::from_file(file).unwrap();
    for (i, entry) in dat.table_entries.as_slice().iter().enumerate() {
        let file = dat.read_entry(&entry).unwrap();
        let file_path = Path::new(&input_trimmed).parent().unwrap().join(i.to_string());
        std::fs::write(file_path, file).unwrap();
        println!("Finished file {i}");
    }
}