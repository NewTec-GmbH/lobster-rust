use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

/// Resolved a module declaration to a path.
///
/// Tries to resolve a module declaration.
/// This is dependent on the current file name.
/// Resolution options are detailed in the code and in [the documentation](https://github.com/NewTec-GmbH/lobster-rust/blob/feature/multifile/doc/module_resolution.md).
/// Additionally builds a context string if the module could be resolved to a path.
///
/// ### Parameters
/// * `current_file` - Path to the current file (where the module was declared via the ```mod``` keyword).
/// * `target_name` - Module name (The module name specified after the ```mod``` keyword).
///
/// ### Returns
/// Some(PathBuf, String) if the module could be resolved to a path. The String contains the additional context info.
///
pub(crate) fn resolve_module_declaration(
    current_file: &Path,
    target_name: &str,
) -> Option<(PathBuf, String)> {
    // Get cwd and target file name.
    let current_path = current_file.parent()?;
    let current_file_stem = current_file.file_stem()?.to_str()?;
    let file_target = target_name.to_string() + ".rs";

    // Read cwd contents.
    let directory_read_results = fs::read_dir(current_path).ok()?;
    let directory_content: Vec<DirEntry> = directory_read_results
        .filter_map(|entry_result| entry_result.ok())
        .collect();

    // Option 1 and 2 only available for main.rs, lib.rs or mod.rs.
    if ["main", "lib", "mod"].contains(&current_file_stem) {
        // Option 1: target.rs file in the same directory.
        for directory_entry in &directory_content {
            if let Some(file_name) = directory_entry.path().file_name() {
                if &file_target == file_name.to_str().unwrap() {
                    // Return path to target_name.rs
                    return Some((directory_entry.path(), String::new()));
                }
            }
        }

        // Option 2: target directory with mod.rs.
        for directory_entry in &directory_content {
            if (directory_entry.path().is_dir()) && (directory_entry.path().ends_with(target_name))
            {
                let subdirectory_content: Vec<DirEntry> = fs::read_dir(directory_entry.path())
                    .ok()?
                    .filter_map(|entry_result| entry_result.ok())
                    .collect();
                // If a subdirectory with the target name exists, find mod.rs in it.
                for subdirectory_entry in subdirectory_content {
                    if let Some(file_name) = subdirectory_entry.path().file_name() {
                        if "mod.rs" == file_name.to_str().unwrap() {
                            let context_string = directory_entry
                                .path()
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string();
                            // Return path to mod.rs.
                            return Some((subdirectory_entry.path(), context_string));
                        }
                    }
                }
            }
        }
    } else {
        // Option 3 and 4 only for files that are not main.rs, lib.rs or mod.rs.

        // Option 3: target.rs in directory with same name as current file.
        for directory_entry in &directory_content {
            if (directory_entry.path().is_dir())
                && (directory_entry.path().ends_with(current_file_stem))
            {
                let subdirectory_content: Vec<DirEntry> = fs::read_dir(directory_entry.path())
                    .ok()?
                    .filter_map(|entry_result| entry_result.ok())
                    .collect();
                // Find the target file in the subdirectory with the same name as the current file.
                for subdirectory_entry in subdirectory_content {
                    if let Some(file_name) = subdirectory_entry.path().file_name() {
                        if file_target == file_name.to_str().unwrap() {
                            // Return path to nested target_name.rs
                            return Some((
                                subdirectory_entry.path(),
                                current_file_stem.to_string(),
                            ));
                        }
                    }
                }
            }
        }

        // Option 4: target/mod.rs in directory with the same name as the current file.
        let current_file_stem = current_file.file_stem()?.to_str()?;
        for directory_entry in &directory_content {
            if (directory_entry.path().is_dir())
                && (directory_entry.path().ends_with(current_file_stem))
            {
                let subdirectory_content: Vec<DirEntry> = fs::read_dir(directory_entry.path())
                    .ok()?
                    .filter_map(|entry_result| entry_result.ok())
                    .collect();
                // Find the target directory with a mod.rs in the subdirectory with the same name as the current file.
                for subdirectory_entry in subdirectory_content {
                    if (subdirectory_entry.path().is_dir())
                        && (subdirectory_entry.path().ends_with(target_name))
                    {
                        // Search for mod.rs in nested directory.
                        let subsubdirectory_content: Vec<DirEntry> =
                            fs::read_dir(subdirectory_entry.path())
                                .ok()?
                                .filter_map(|entry_result| entry_result.ok())
                                .collect();
                        for subsubdirectory_entry in subsubdirectory_content {
                            if let Some(file_name) = subdirectory_entry.path().file_name() {
                                if "mod.rs" == file_name.to_str().unwrap() {
                                    // Build context string from sub and subsub directories.
                                    let context_string =
                                        current_file_stem.to_string() + "." + target_name;
                                    // Return path to nested mod.rs
                                    return Some((subsubdirectory_entry.path(), context_string));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Include could not be resolved.
    None
}
