# Module Declaration Resolution

## Introduction

lobster-rusts parsing uses the rust-analyzers ra_ap_syntax crate functionality to parse the contents of a single source file into a syntax tree.
As most projects do in fact not consist of only a single source file, a method of parsing all relevant files is needed.

In Rust, each project has a defined entry point in the main.rs or lib.rs (or both, which we resolve via the --lib flag).
Every other source file (and the modules included) that is needed for the project build is then referenced via the ```mod``` keyword.

Via ```mod filename``` a source file on the same directory level can be included (This would include the module filename.rs).
```mod directoryname```on the other hand would seach for ./directoryname/mod.rs, with mod.rs then including one or several submodules via the ```mod```keyword.

This [example repository](https://github.com/superjose/rust-include-files-example/tree/master) shows the multiple ways to include modules with additional explanations.

This repository also showcases another way to include submodules without the ```mod.rs``` file, as can be seen with the /house/bathroom submodule.
Instead of a mod.rs file in the /bathroom directory, a bathroom.rs file **on the same level** as the bathroom directory lists the used modules.
This way of including submodules is valid since Rust 2018 and should therefore be supported.

This completes the 3 most common ways to include modules that need to be supported in lobster-rust.

## Supported cases and resolution order

1. Inclusion of module on the same level.

    ```mod filename;```: Try to parse ```./filename.rs``` in the same directory as the current file.

    Only applies if current file is ```mod.rs```, ```main.rs``` or ```lib.rs```.

2. Inclusion of submodules in directory with mod.rs file.

    ```mod dirname;``` If case one does not apply (no dirname.rs), try to find ```./dirname/mod.rs```.

    Only applies if current file is ```mod.rs```, ```main.rs``` or ```lib.rs```.

3. Inclusion of submodules in directory with the same name as the current file.

    ```mod filename;``` Try to resolve ```./current_file_stem/filename.rs```.

4. Inclsion of submodules in subdirectory of directory with the same name as the current file.

    ```mod dirname;``` Try to resolve ```./current_file_stem/dirname/mod.rs```.

## Additional details

1. path attributes (like ```#[path="./other-file.rs"]```) to alter the mod keyword: The path that would be included by the mod keyword can be altered by the ```path``` attribute. This can be useful to add code files including a - in their name.

## Unsupported

All other ways of including source files in a Rust project are currently not supported by lobster-rust. Please open an issue with details on the necessary module inclusion method if it needs to be integrated into the tool.
