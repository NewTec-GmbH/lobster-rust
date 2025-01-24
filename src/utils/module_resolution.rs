use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

pub(crate) fn resolve_include(current_file: &Path, target_name: &str) -> Option<(PathBuf, String)> {
    let current_path = current_file.parent()?;

    // Option 1: target.rs file in the same directory.
    let file_target = target_name.to_string() + ".rs";
    let directory_read_results = fs::read_dir(current_path).ok()?;
    let directory_content: Vec<DirEntry> = directory_read_results
        .filter_map(|entry_result| entry_result.ok())
        .collect();
    for directory_entry in &directory_content {
        if let Some(file_name) = directory_entry.path().file_name() {
            if &file_target == file_name.to_str().unwrap() {
                return Some((directory_entry.path(), String::new()));
            }
        }
    }

    // Option 2: target directory with mod.rs.
    for directory_entry in &directory_content {
        if (directory_entry.path().is_dir()) && (directory_entry.path().ends_with(target_name)) {
            let subdirectory_content: Vec<DirEntry> = fs::read_dir(directory_entry.path())
                .ok()?
                .filter_map(|entry_result| entry_result.ok())
                .collect();
            for subdirectory_entry in subdirectory_content {
                if let Some(file_name) = subdirectory_entry.path().file_name() {
                    if "mod.rs" == file_name.to_str().unwrap() {
                        let context_string = directory_entry
                            .path()
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        return Some((subdirectory_entry.path(), context_string));
                    }
                }
            }
        }
    }

    // Option 3: target.rs in directory with same name as current file.
    let current_file_stem = current_file.file_stem()?.to_str()?;
    for directory_entry in &directory_content {
        if (directory_entry.path().is_dir())
            && (directory_entry.path().ends_with(current_file_stem))
        {
            let subdirectory_content: Vec<DirEntry> = fs::read_dir(directory_entry.path())
                .ok()?
                .filter_map(|entry_result| entry_result.ok())
                .collect();
            for subdirectory_entry in subdirectory_content {
                if let Some(file_name) = subdirectory_entry.path().file_name() {
                    if file_target == file_name.to_str().unwrap() {
                        return Some((subdirectory_entry.path(), current_file_stem.to_string()));
                    }
                }
            }
        }
    }

    // Option 4: target/mod.rs in directory with the same name as the current file. TODO

    // Include could not be resolved.
    None
}
