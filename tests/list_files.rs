use baskelian_toolbox::dat::DAT;

mod common;

#[test]
fn list_files() {
    let (dat_file, _) = common::setup();
    let dat = DAT::from_file(dat_file).unwrap();
    let mut file_count = 0;
    for inner in &dat.inner_dats {
        for entry in &inner.file_table_entries {
            let file = dat.read_file(inner, entry).unwrap();
            assert_eq!(file.data.len(), entry.size);
            file_count += 1;
        }
    }
    assert_eq!(file_count, 45896);
}