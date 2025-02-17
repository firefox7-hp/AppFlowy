use console::Style;

use similar::{ChangeTag, TextDiff};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
use walkdir::WalkDir;

pub fn read_file(path: &str) -> Option<String> {
    let mut file = File::open(path).unwrap_or_else(|_| panic!("Unable to open file at {}", path));
    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Ok(_) => Some(content),
        Err(e) => {
            log::error!("{}, with error: {:?}", path, e);
            Some("".to_string())
        }
    }
}

pub fn save_content_to_file_with_diff_prompt(content: &str, output_file: &str) {
    if Path::new(output_file).exists() {
        let old_content = read_file(output_file).unwrap();
        let new_content = content.to_owned();
        let write_to_file = || match OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(true)
            .open(output_file)
        {
            Ok(ref mut file) => {
                file.write_all(new_content.as_bytes()).unwrap();
            }
            Err(err) => {
                panic!("Failed to open log file: {}", err);
            }
        };
        if new_content != old_content {
            print_diff(old_content, new_content.clone());
            write_to_file()
        }
    } else {
        match OpenOptions::new().create(true).write(true).open(output_file) {
            Ok(ref mut file) => file.write_all(content.as_bytes()).unwrap(),
            Err(err) => panic!("Open or create to {} fail: {}", output_file, err),
        }
    }
}

pub fn print_diff(old_content: String, new_content: String) {
    let diff = TextDiff::from_lines(&old_content, &new_content);
    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
            };

            match change.tag() {
                ChangeTag::Delete => {
                    print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
                }
                ChangeTag::Insert => {
                    print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
                }
                ChangeTag::Equal => {}
            };
        }
        println!("---------------------------------------------------");
    }
}

#[allow(dead_code)]
pub fn is_crate_dir(e: &walkdir::DirEntry) -> bool {
    let cargo = e.path().file_stem().unwrap().to_str().unwrap().to_string();
    cargo == *"Cargo"
}

#[allow(dead_code)]
pub fn is_proto_file(e: &walkdir::DirEntry) -> bool {
    if e.path().extension().is_none() {
        return false;
    }
    let ext = e.path().extension().unwrap().to_str().unwrap().to_string();
    ext == *"proto"
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false)
}

pub fn create_dir_if_not_exist(dir: &str) {
    if !std::path::Path::new(&dir).exists() {
        std::fs::create_dir_all(&dir).unwrap();
    }
}

#[allow(dead_code)]
pub(crate) fn walk_dir<F1, F2>(dir: &str, filter: F2, mut path_and_name: F1)
where
    F1: FnMut(String, String),
    F2: Fn(&walkdir::DirEntry) -> bool,
{
    for (path, name) in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| filter(e))
        .map(|e| {
            (
                e.path().to_str().unwrap().to_string(),
                e.path().file_stem().unwrap().to_str().unwrap().to_string(),
            )
        })
    {
        path_and_name(path, name);
    }
}

#[allow(dead_code)]
pub fn suffix_relative_to_path(path: &str, base: &str) -> String {
    let base = Path::new(base);
    let path = Path::new(path);
    path.strip_prefix(base).unwrap().to_str().unwrap().to_owned()
}
