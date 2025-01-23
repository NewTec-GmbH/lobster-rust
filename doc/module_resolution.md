# Module Resolution Strategy

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

## Cases and resolution order

1. Inclusion of module on the same level.

    ```mod filename;```: Try to parse ```./filename.rs`` in the same directory as the current file.

2. Inclusion of submodules in directory with mod.rs file.

    ```mod dirname;``` If there is no dirname.rs, try to find ```./dirname/mod.rs```.

3. Inclusion of submodules in directory with directoryname.rs file.

    ```mod filename;``` If case 1 and 2 dont apply, try to find ```./current_file_name/filename.rs```.

## Currently unsupported

1. path attributes to alter the mod keyword: The path that would be included by the mod keyword can be altered by the ```path``` attribute. This can be useful to add code files including a - in their name.
