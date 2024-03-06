use std::{fs::{create_dir, File}, path::{Path, PathBuf}};

pub fn setup() -> (File, PathBuf) {
    let dat_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("_artifacts/DATA.DAT");
    let extract_err = create_dir(dat_path.parent().unwrap().join("extracted")).err();
    if extract_err.is_some() {
        if !Path::new(&dat_path.parent().unwrap().join("extracted")).exists() {
            panic!("unknown directory error")
        }
    }
    return (File::open(&dat_path).expect("DATA.DAT file not found in _artifacts folder!"), dat_path)
}