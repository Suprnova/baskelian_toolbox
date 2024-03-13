pub struct Name {
    pub file_path: String,
    pub name_count: u16,
    pub names: Vec<String>,
}

impl Name {
    pub fn new(data: &[u8]) -> Self {
        let binding = String::from_utf8(data.to_vec()).unwrap();
        let mut lines = binding.lines();
        lines.next();
        let file_path = lines.next().unwrap().to_string();
        let name_count: u16 = lines.next().unwrap().parse().unwrap();
        let mut names: Vec<String> = vec![];
        for _ in 0..name_count {
            names.push(lines.next().unwrap().to_string());
        }
        Self {
            file_path,
            name_count,
            names,
        }
    }
}
