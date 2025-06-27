use custom_logger as log;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct MarkdownFile {
    pub path: String,
    pub contents: String,
    pub headers: Option<String>,
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
    dir: PathBuf,
    ending: &str,
    prefix: &PathBuf,
    use_headers: bool,
    header_regex: Option<String>,
) -> Result<Vec<MarkdownFile>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            let mut sub_files =
                load_files_from_dir(path, ending, prefix, use_headers, header_regex.clone())?;
            files.append(&mut sub_files);
        } else if path.is_file() && path.has_file_extension(ending) {
            log::debug!("reading file {:?} for embedding", path);
            let contents = fs::read_to_string(&path)?;
            let path = Path::new(&path).strip_prefix(prefix)?.to_owned();
            let path_id = path.to_str().expect("path should be valid");
            if !use_headers {
                let words: Vec<String> = contents.split_whitespace().map(str::to_string).collect();
                let res = batch_file_contents(words, path_id.to_string());
                files.append(&mut res.unwrap());
            } else {
                let res = batch_file_headers(contents, path_id.to_string(), header_regex.clone());
                files.append(&mut res.unwrap());
            }
        }
    }
    Ok(files)
}

pub fn batch_file_contents(
    words: Vec<String>,
    path_id: String,
) -> Result<Vec<MarkdownFile>, Box<dyn std::error::Error>> {
    let mut result: Vec<MarkdownFile> = Vec::new();
    let batch_size = 200;
    let overlap = 20;
    let batch_count = words.len() / batch_size;
    let remainder = words.len() % batch_size;

    log::info!("batch count {}", batch_count);
    log::info!("remainder  {}", remainder);
    log::info!("path id    {}", path_id);

    for i in 0..batch_count {
        let from = i * batch_size;
        let to = (i * batch_size) + batch_size + overlap;
        log::info!("from {}", from);
        log::info!("to   {}", to);
        let mkd = MarkdownFile {
            path: format!("{}-{}", path_id, i),
            headers: None,
            contents: words[from..to].join(" ").clone(),
        };
        log::info!("content length  {}", mkd.contents.len());
        result.insert(0, mkd);
    }

    let mkd = MarkdownFile {
        path: format!("{}-{}", path_id, batch_count),
        headers: None,
        contents: words[batch_count * batch_size..].join(" ").clone(),
    };
    log::info!("remainder from {}", batch_count * batch_size);
    log::info!("remainder length {}", mkd.contents.len());
    result.insert(0, mkd);

    Ok(result)
}

pub fn batch_file_headers(
    words: String,
    path_id: String,
    header_regex: Option<String>,
) -> Result<Vec<MarkdownFile>, Box<dyn std::error::Error>> {
    let mut result: Vec<MarkdownFile> = Vec::new();
    let mut headers = String::new();

    log::debug!("path id    {}", path_id);

    let lines: Vec<String> = words.split("\n").map(str::to_string).collect();
    for line in lines.iter() {
        let header_text = header_regex.as_ref().map_or("# script", |v| v).to_string();
        if line.contains(&header_text.clone()) {
            headers.push_str(&format!("{}\n", line));
            break;
        }
    }
    let mkd = MarkdownFile {
        path: format!("{}", path_id),
        headers: Some(headers),
        contents: words.clone(),
    };
    if mkd.headers.is_some() {
        log::debug!("headers length  {}", mkd.headers.as_ref().unwrap().len());
    }
    log::debug!("content length  {}", mkd.contents.len());
    result.insert(0, mkd);

    Ok(result)
}
