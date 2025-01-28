# General Workflow

Start from a rust source file (main.rs or lib.rs)

```rust
fn main() {
    // lobster-trace: section.requirement
    somethingIsHappening();
}
```

via the ra_ap_syntax cratee &rarr; syntax tree

![syntax tree](https://www.plantuml.com/plantuml/proxy?cache=no&src=https://raw.githubusercontent.com/NewTec-GmbH/lobster-rust/tree/main/doc/syntax_tree.plantuml)

via the RustVisitor &rarr; trace tree

![trace tree](https://www.plantuml.com/plantuml/proxy?cache=no&src=https://raw.githubusercontent.com/NewTec-GmbH/lobster-rust/tree/main/doc/trace_tree.plantuml)

This trace tree contains significantly less data than the Syntax tree.
Each node contains the data needed to represent itself in the lobster common nterchange format.

&rarr; Lobster common interchange format

```json
{
    "data": [
        {
            "tag": "rust main.main",
            "name": "main.main",
            "location": {
                "kind": "file",
                "file": "main",
                "line": 15,
                "column": 1
            },
            "messages": [],
            "just_up": [],
            "just_down": [],
            "just_global": [],
            "refs": [
                "req section.requirement"
            ],
            "language": "Rust",
            "kind": "Function"
        }
    ],
    "generator": "lobster-rust",
    "schema": "lobster-imp-trace",
    "version": 3
}
```
