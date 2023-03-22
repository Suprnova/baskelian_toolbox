use std::{fs::File, io::stdin, path::Path};

use baskelian_toolbox::dat_file::DAT;
fn main() {
    let mut s = String::new();
    println!("Please enter the path to the DATA.DAT file.");
    stdin().read_line(&mut s).expect("Error parsing user input");
    let input_trimmed = s.trim();
    let file = File::open(input_trimmed).unwrap();
    let dat = DAT::from_file(file).unwrap();
    for (i, inner_dat) in dat.inner_dats.as_slice().iter().enumerate() {
        for (j, entry) in inner_dat.file_table_entries.as_slice().iter().enumerate() {
            let mut extension = String::new();
            let file = dat.read_file(inner_dat, entry).unwrap();
            if file.len() == 0 {
                continue;
            }
            // This will be adapted into a set of enums and detector functions in the main library, this is a temporary POC
            if file[0] == 0x16 {
                extension = String::from(".txd");
            } else if file[0] == 0x10 {
                extension = String::from(".dff");
            } else if file[0] == 0x1B {
                extension = String::from(".anm");
            } else {
                let mut current_index: usize = 0;
                let mut space_count: u8 = 0;
                while space_count <= 20 && current_index < 150 && current_index < file.len() && file[current_index] != 0x0A {
                    if file[current_index] == 0x20 {
                        space_count += 1;
                    }
                    current_index += 1;
                }
                if space_count == 20 {
                    extension = String::from(".stats");
                }
            }
            let file_path = Path::new(&input_trimmed).parent().unwrap().join(format!("{i}-{j}{extension}"));
            std::fs::write(file_path, file).unwrap();
            println!("Finished file {i}-{j}{extension}");
        }
    }
}