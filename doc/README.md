# lobster-rust Documentation

## Overview

lobster-rust is a tool to extract requirement annotations from Rust source code.
It produces the [LOBSTER common interchange format](https://github.com/bmw-software-engineering/lobster/blob/main/documentation/schemas.md) as output.

The tool is written fully in Rust and can trace its own requirements to its own source code. The goal is 100% requirement adn code coverage, for progress see the [feature/requirements](https://github.com/NewTec-GmbH/lobster-rust/tree/feature/requirements) branch. The requirements for lobser-rust and the self tracing also serve as an extensive example on what can be achieved with LOBSTER and lobser-rust.

It should be mentioned again that the source code is fully doc commented, meaning cargo can create and display extensive documentation via ```cargo doc --open```.

## Tool Context

lobster-rust is intended to be used with BMWs [LOBSTER](https://github.com/bmw-software-engineering/lobster) and produces output in the lobster common interchange format. It has very limited use on its own.

Lobster is used to trace source code to requirements and generate tracing reports containing the results. Its ecosystem consists of multiple tools for requirements tracing, source code tracing and report generation.
While tools exist for e.g. Python and C++, no tool for Rust projects was available. lobster-rust fills this void and allows usage of LOBSTER and therefore also [TRLC](https://github.com/bmw-software-engineering/trlc) for Rust based projects.

An overview on how lobster-rust fits into the LOBSTER toolchain can be seen below, adapted from [pyTRLCConverters documentation](https://github.com/NewTec-GmbH/pyTRLCConverter).

![lobster-rust context](https://www.plantuml.com/plantuml/proxy?cache=no&src=https://raw.githubusercontent.com/NewTec-GmbH/lobster-rust/refs/heads/main/doc/diagrams/lobster_context.plantuml)

## Requirement Tracing

The tracing functionality is designed to mirror the functionality of [lobster-python](https://github.com/bmw-software-engineering/lobster/tree/main/packages/lobster-tool-python).
lobster-rust uses comments in the source code to extract trace information.

### Function Tracing

Functions (and methods) can be traced to requirements by adding tracing tags into the function body.

```rust
fn potato() -> String {
    // lobster-trace: something.example
    "potato".to_string()
}
```

This will produce lobster common interchange output as follows:

```jsom
{
    "tag": "rust main.potato",
    "name": "main.potato",
    "location": {
        "kind": "file",
        "file": "main",
        "line": 30,
        "column": 1
    },
    "messages": [],
    "just_up": [],
    "just_down": [],
    "just_global": [],
    "refs": [
        "req something.example"
    ],
    "language": "Rust",
    "kind": "Function"
}
```

Note that this does not require the comment to be a doc comment (```/// doc comment```) with three slashes, but it works with doc comments too. Also, although the comment is inside the function body, it is currently not required to be an inner comment (```//! inner-line doc comment```).

Justifications (or more fittingly exclusions) can also be added to annotate code that is not directly derived from a requirement.

```rust
fn potato() -> String {
    // lobster-exclude: No requirement for this, but its in the code anyways.
    "potato".to_string()
}
```

### Struct Tracing

Structs can be annotated just like functions.

```rust
struct PotatoFarm {
    // lobster-trace: something.important
    "acres": usize,
    "variety": PotatoVariety,
}
```

### Trait Implementation Tracing - Planned

Traits are Rusts interfaces and a powerful tool in the language. We are of the opinion that the implementation of a trait can already satisfy certain requirements. Of course this could be traced by leaving comments in every single method required for the trait implementation. But it would prove useful to annotate the full trait implementation with a single comment.

```rust
impl SomeTrait for  PotatoFarm {
    // lobster-trace: something.trait

    fn trait_one() {

    }

    fn trait_two() {

    }
}
```

As trait implementations do not show up as separate items in the lobster common interchange format, this requirement trace would ideally be applied to the struct for which the trait is implemented (The PotatoFarm in the example above).

This is a planned feature and not yet implemented!

## Architecture

### Class Diagram

![class structure](https://www.plantuml.com/plantuml/proxy?cache=no&src=https://raw.githubusercontent.com/NewTec-GmbH/lobster-rust/refs/heads/main/doc/diagrams/class_structure.plantuml)

### Module Declaration Resolution

Details on how lobster-rust resolves submodules can be found [here](https://github.com/NewTec-GmbH/lobster-rust/tree/main/doc/module_resolution.md).

### RustVisitor Workflow
