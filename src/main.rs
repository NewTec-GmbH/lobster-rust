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

//! lobster-rust tool to prodce lobster common interchange format from a rust project.

use clap::Parser;
use json::{object::Object, JsonValue};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use utils::context::Context;
use visitor::RustVisitor;

mod location;
mod syntax_extensions;
mod traceable_node;
mod utils;
mod visitor;

/// Entry function of the tool.
///
/// This function defines the general workflow of lobste-rust.
/// First the CLI args are parsed  and the first visitor is created and started accordingly.
/// This visitor is expected to start module visitors for every resolved module inlusion by itself.
/// Afterwards all parsed information is combined into the lobster common interchange format.
fn main() {
    // Parse command line interface arguments.
    let args = args::Cli::parse();

    // Determine entry file filename (lib.rs instead of main.rs if --lib flag is set).
    let filename;
    if args.lib {
        filename = Path::new("lib.rs");
    } else {
        filename = Path::new("main.rs");
    }
    let filepath = Path::new(&args.dir).join(filename);

    // Create and run visitor on entry file.
    let mut visitor = RustVisitor::new(filepath, Context::Empty);
    visitor.parse_file();

    // Get root node of entry file and other modules in the project.
    let modules = visitor.get_traceable_nodes();

    // Convert parsed modules to lobster common interchange format.
    let data: Vec<JsonValue> = modules.iter().map(|m| m.to_lobster()).flatten().collect();

    // Combine parsed data and fixed information to full lobster common interchange format output.
    let mut jout = JsonValue::Object(Object::new());
    let _ = jout.insert("data", data);
    let _ = jout.insert("generator", "lobster-rust");
    let _ = jout.insert("schema", "lobster-imp-trace");
    let _ = jout.insert("version", 3);

    // Write lobster common interchange format to output file.
    let outfile: &Path = Path::new(&args.out);
    match File::create(&outfile) {
        // Panic if we cant write the results. Print error details.
        Err(e) => panic!("Outfile: {:#?}\n{}", &outfile, e),
        Ok(outfile) => {
            let mut outwriter = BufWriter::new(outfile);
            let _ = jout.write_pretty(&mut outwriter, 4);
        }
    }
}

/// Submodule to define the tools CLI.
#[allow(unused_parens)]
mod args {
    use clap::Parser;
    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    pub(super) struct Cli {
        /// Directory of main.rs (or lib.rs).
        #[arg(default_value_t = ("./src/".to_string()))]
        pub(super) dir: String,

        /// Output file for the lobster common interchange format output.
        #[arg(default_value_t = ("rust.lobster".to_string()))]
        pub(super) out: String,

        /// Parse lib.rs as project root instead of main.rs.
        #[arg(short, long)]
        pub(super) lib: bool,

        /// Generate activity traces (tests) instead of an implementation trace. UNSUPPORTED.
        #[arg(long)]
        pub(super) activity: bool,

        /// Only trace functions with tags. UNSUPPORTED.
        #[arg(long)]
        pub(super) only_tagged_functions: bool,
    }
}
