use std::{
    fs::{create_dir, File},
    path::Path,
};

use baskelian_toolbox::dat::DAT;
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
        let mut all_same_name = false;
        if inner_dat
            .files
            .iter()
            .all(|f| f.file_name == inner_dat.archive_name && f.file_name.is_some())
        {
            all_same_name = true;
        }
        for (j, file) in inner_dat.files.iter().enumerate() {
            let data = dat.read_file(inner_dat, file).unwrap();
            let mut file_path = Path::new(&dat_path).parent().unwrap().join(format!(
                "extracted/{}-{}{}",
                inner_dat.archive_name.clone().unwrap_or(i.to_string()),
                file.file_name.clone().unwrap_or(j.to_string()),
                file.file_type
            ));

            if all_same_name {
                file_path = Path::new(&dat_path).parent().unwrap().join(format!(
                    "extracted/{}{}",
                    inner_dat.archive_name.clone().unwrap(),
                    file.file_type
                ));
            }
            std::fs::write(file_path.clone(), data).unwrap();
            println!(
                "Finished file {}",
                file_path.to_str().unwrap().split('\\').last().unwrap()
            );
        }
    }
}
