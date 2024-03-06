use std::fs;

use baskelian_toolbox::dat::{FileType, DAT};

mod common;

#[test]
fn extract_stats() {
    let (dat_file, path) = common::setup();
    let extract_path = path.parent().unwrap().join("extracted");
    let dat = DAT::from_file(dat_file).unwrap();
    let mut i = 0;
    for inner in &dat.inner_dats {
        let mut j = 0;
        for entry in &inner.file_table_entries {
            let file = dat.read_file(inner, entry).unwrap();
            if let FileType::STATS{stats_file} = &file.file_type {
                assert_eq!(stats_file.entries[11].unknown_6_len, 3);
                fs::write(extract_path.join(format!("{i}{j}{}", file.file_type)), file.data).expect("error writing file")
            }
            j += 1;
        }
        i += 1;
    }
    assert!(extract_path.join("35-1.stats").exists());
}