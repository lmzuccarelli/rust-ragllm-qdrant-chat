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
    pub sentences: Vec<String>,
}

enum FileState {
    None,
    CodeBlock,
    Sentence,
    Comments,
}

impl MarkdownFile {
    pub fn new(path: String, contents: String) -> Self {
        Self {
            path,
            contents,
            sentences: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        let mut contents = Vec::new();
        let mut state = FileState::None;
        let mut sentence = String::new();

        for line in self.contents.lines() {
            match state {
                FileState::None => {
                    if line.starts_with("```") {
                        state = FileState::CodeBlock;
                        sentence = String::new();
                        sentence.push_str(line);
                        sentence.push('\n');
                    } else if line.starts_with("---") {
                        state = FileState::Comments;
                    } else if !line.starts_with('#') && !line.is_empty() {
                        state = FileState::Sentence;
                        sentence = String::new();
                        sentence.push_str(line);
                        sentence.push('\n');
                    }
                }
                FileState::CodeBlock => {
                    sentence.push_str(line);
                    if line.starts_with("```") {
                        contents.push(sentence);
                        sentence = String::new();
                        state = FileState::None;
                    }
                }
                FileState::Comments => {
                    if line.starts_with("---") {
                        state = FileState::None;
                    }
                }
                FileState::Sentence => {
                    if line.is_empty() {
                        state = FileState::None;
                        contents.push(sentence);
                        sentence = String::new();
                    } else {
                        sentence.push_str(line);
                        sentence.push('\n');
                    }
                }
            }
        }
        self.sentences = contents;
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
            log.info(&format!("Path: {:?}", path));
            let contents = fs::read_to_string(&path)?;
            let path = Path::new(&path).strip_prefix(prefix)?.to_owned();
            let key = path.to_str().expect("should find path");
            let mut file = MarkdownFile::new(key.to_string(), contents);
            file.parse();
            files.push(file);
        }
    }
    Ok(files)
}
