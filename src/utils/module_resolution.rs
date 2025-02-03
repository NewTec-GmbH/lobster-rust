//! Utility functions to resolve module declarations to file paths.

// BSD 3-Clause License
//
// Copyright (c) 2025, NewTec GmbH
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions
//    and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of
//    conditions and the following disclaimer in the documentation and/or other materials provided
//    with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to
//    endorse or promote products derived from this software without specific prior written
//    permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICU5LAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::utils::context::Context;
use std::fs::{self, DirEntry};
use std::ops::Deref;
use std::path::{Path, PathBuf};

/// Resolved a module declaration to a path.
///
/// Tries to resolve a module declaration.
/// This is dependent on the current file name.
/// Resolution options are detailed in the code and in [the documentation](https://github.com/NewTec-GmbH/lobster-rust/blob/main/doc/module_resolution.md).
/// Additionally builds a context string if the module could be resolved to a path.
///
/// ### Parameters
/// * `current_file` - Path to the current file (where the module was declared via the ```mod```
///   keyword).
/// * `target_module_name` - Module name (The module name specified after the ```mod``` keyword).
///
/// ### Returns
/// Some(PathBuf, Context) if the module could be resolved to a path.
pub(crate) fn resolve_module_declaration(
    current_file: &Path,
    target_module_name: &str,
) -> Option<(PathBuf, Context)> {
    // Get cwd and target file name.
    let current_path = current_file.parent()?;
    let current_file_stem = current_file.file_stem()?.to_str()?;
    let file_target = target_module_name.to_string() + ".rs";

    // Read cwd contents.
    let directory_read_results = fs::read_dir(current_path).ok()?;
    let directory_content: Vec<PathBuf> = directory_read_results
        .filter_map(|entry_result| entry_result.ok().map(|content| content.path()))
        .collect();

    // For main.rs, lib.rs or mod.rs,
    // Rust tries to resolve the module in the current directory.
    if ["main", "lib", "mod"].contains(&current_file_stem) {
        // Option 1: file named target.rs
        if let Some(file_result) = check_file_module(&directory_content, &file_target) {
            return Some(file_result);
        } else {
            // Option 2: target directory with mod.rs.
            check_directory_module(&directory_content, &target_module_name)
        }
    } else {
        // For files other than main.rs, lib.rs or mod.rs,
        // Rust tries to resolve the submodule in a subdirectory with the same name as the current
        // file.
        check_nested_submodule(
            &directory_content,
            &file_target,
            &target_module_name,
            &current_file_stem,
        )
    }
}

/// Check for a file in the given directory contents with the module name.
///
/// Searches the provided directory contents for a rust source file that would match the module
/// name. If the file is found, a PathBuf to the found file and an empty Context are returned.
/// The Context is empty, as the file lies in the same directory and therefore the same Context.
///
/// ### Parameters
/// * `directory_content` - Paths to contents of the current directory.
/// * `file_target` - File name to search for to resolve the module.
///
/// ### Returns
/// Some(PathBuf, Context) if the module could be resolved to a file.
fn check_file_module(
    directory_content: &Vec<PathBuf>,
    file_target: &str,
) -> Option<(PathBuf, Context)> {
    for directory_entry in directory_content {
        if let Some(file_name) = directory_entry.file_name() {
            if file_target == file_name.to_str().unwrap() {
                // Return path to target_name.rs
                return Some((directory_entry.deref().to_path_buf(), Context::Empty));
            }
        }
    }
    None
}

/// Check for a directory in the given directory contents with the module name that contains a
/// mod.rs.
///
/// Searches the provided directory contents for a directory that matches the module name.
/// If the directory is found, it is checked to contain a mod.rs.
/// If a mod.rs is found, the PathBuf to the mod.rs is returned with a Context.
/// The Context contains the directory name to reflect the additional nesting.
///
/// ### Parameters
/// * `directory_content` - Paths to contents of the current directory.
/// * `target_module_name` - Directory name to search for to resolve the module.
///
/// ### Returns
/// Some(PathBuf, Context) if the module could be resolved to a directory (with mod.rs).
fn check_directory_module(
    directory_content: &Vec<PathBuf>,
    target_module_name: &str,
) -> Option<(PathBuf, Context)> {
    for directory_entry in directory_content {
        if (directory_entry.is_dir()) && (directory_entry.ends_with(target_module_name)) {
            let subdirectory_content: Vec<DirEntry> = fs::read_dir(directory_entry)
                .ok()?
                .filter_map(|entry_result| entry_result.ok())
                .collect();
            // If a subdirectory with the target name exists, find mod.rs in it.
            for subdirectory_entry in subdirectory_content {
                if let Some(file_name) = subdirectory_entry.path().file_name() {
                    if "mod.rs" == file_name.to_str().unwrap() {
                        // Return path to mod.rs.
                        return Some((
                            subdirectory_entry.path(),
                            Context::from_str(target_module_name),
                        ));
                    }
                }
            }
        }
    }
    None
}

/// Resolve a submodule.
///
/// Searches the provided directory contents for a subdirectory that matches the current file name.
/// This subdirectory is then searched for the target submodule, either as a file module or a
/// directory module.
///
/// ### Parameters
/// * `directory_content` - Paths to contents of the current directory.
/// * `file_target` - File name to search for to resolve the module.
/// * `target_module_name` - Directory name to search for to resolve the module.
///
/// ### Returns
/// Some(PathBuf, Context) if the module could be resolved to a source file or directory (with
/// mod.rs).
fn check_nested_submodule(
    directory_content: &Vec<PathBuf>,
    file_target: &str,
    target_module_name: &str,
    current_file_stem: &str,
) -> Option<(PathBuf, Context)> {
    // Find subdirectory with the same name as the current file.
    if let Some(subdirectory) = directory_content
        .iter()
        .filter(|directory_entry| {
            directory_entry.is_dir() && directory_entry.ends_with(current_file_stem)
        })
        .next()
    {
        // Get the contents of the subdirectory.
        let subdirectory_content: Vec<PathBuf> = fs::read_dir(subdirectory)
            .ok()?
            .filter_map(|entry_result| entry_result.ok().map(|content| content.path()))
            .collect();
        let subdirectory_context = Context::from_str(current_file_stem);

        // Try to resolve the submodule to a file or directory in the subdirectory.
        if let Some((file_module_path, nested_context)) =
            check_file_module(&subdirectory_content, file_target)
        {
            return Some((file_module_path, subdirectory_context + nested_context));
        } else if let Some((directory_module_path, nested_context)) =
            check_directory_module(&subdirectory_content, target_module_name)
        {
            return Some((directory_module_path, subdirectory_context + nested_context));
        }
    }
    None
}
