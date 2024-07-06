use crate::api::schema::*;
use custom_logger::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct MarkdownFile {
    pub path: String,
    pub contents: String,
    pub headings: Vec<String>,
}

enum FileState {
    None,
    Heading,
}

impl MarkdownFile {
    pub fn new(path: String, contents: String) -> Self {
        Self {
            path,
            contents,
            headings: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        let mut contents = Vec::new();
        let mut state = FileState::None;
        let mut headings = String::new();

        for line in self.contents.lines() {
            // we are only interested in headings
            match state {
                FileState::None => {
                    if line.starts_with('#') && !line.is_empty() {
                        state = FileState::Heading;
                        headings = String::new();
                        headings.push_str(line);
                        headings.push('\n');
                    }
                }
                FileState::Heading => {
                    state = FileState::None;
                    contents.push(headings.clone());
                }
            }
        }
        self.headings = contents.clone();
    }
}

trait HasFileExt {
    fn has_file_extension(&self, ending: &str) -> bool;
}

impl HasFileExt for Path {
    fn has_file_extension(&self, ending: &str) -> bool {
        if let Some(path) = self.to_str() {
            return path.ends_with(ending);
        }
        false
    }
}

// Load files from directory by ending
pub fn load_files_from_dir(
    log: &Logging,
    dir: PathBuf,
    ending: &str,
    prefix: &PathBuf,
) -> Result<Vec<MarkdownFile>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            let mut sub_files = load_files_from_dir(log, path, ending, prefix)?;
            files.append(&mut sub_files);
        } else if path.is_file() && path.has_file_extension(ending) {
            log.info(&format!("reading file {:?} for embedding", path));
            let contents = fs::read_to_string(&path)?;
            let path = Path::new(&path).strip_prefix(prefix)?.to_owned();
            let key = path.to_str().expect("path should be valid");
            let mut file = MarkdownFile::new(key.to_string(), contents);
            file.parse();
            files.push(file);
        }
    }
    Ok(files)
}
