#[derive(Debug, Clone)]
pub struct FileState {
    pub open_files: Vec<OpenFile>,
}

#[derive(Debug, Clone)]
pub struct OpenFile {
    pub file_path: String,
    pub current_file: bool
}

impl FileState {
    pub fn new() -> Self {
        Self {
            open_files: vec![],
        }
    }

    pub fn insert_file(&mut self, path: String) {
        self.open_files.push(OpenFile {
            file_path: path,
            current_file: true
        });
    }
}