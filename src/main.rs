use std::fs::{self, File};
use std::io::BufWriter;
use location::{NodeLocation, FileReference};
use ra_ap_syntax::{AstNode, SourceFile};
use ra_ap_edition::Edition;
use json::{object::Object, JsonValue};
use visitor::{RustVisitor, Visitor};
use clap::Parser;

mod location;
mod visitor;
mod traceable_node;
mod syntax_extensions;
mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Directory of main.rs (or lib.rs)
    dir: Option<String>,

    /// Output directory for the .lobster file
    out: Option<String>,

    /// Parse lib.rs as project root instead of main.rs
    #[arg(short, long)]
    lib: bool,

    /// Generate activity traces (tests) instead of an implementation trace. UNSUPPORTED
    #[arg(long)]
    activity: bool,

    /// Only trace functions with tags
    #[arg(long)]
    only_tagged_functions: bool
}

fn main() {
    let args = Cli::parse();

    let mut filename;
    if args.lib {
        filename = "lib.rs".to_string();
    } else {
        filename = "main.rs".to_string();
    }

    if let Some(directory) = args.dir {
        filename.insert_str(0, &directory);
    }

    let text: String;
    match fs::read_to_string(&filename) {
        Ok(t) => text = t,
        Err(e) => panic!("File: {}\n{}", &filename, e),
    }
    let parse = SourceFile::parse(&text, Edition::Edition2024);
    let tree: SourceFile = parse.tree();
    let root_node = tree.syntax();

    let mut visitor = RustVisitor::new(filename);

    visitor.travel(&root_node);

    let module = visitor.vdata.get_root_mut().unwrap();
    
    let mut data: Vec<JsonValue> = Vec::new();
    module.to_lobster(&mut data);

    // add data
    let mut jout = JsonValue::Object(Object::new());
    let _ = jout.insert("data", data);

    // write json to output file
    let outfile: String;
    if let Some(outarg) = args.out {
        outfile = outarg;
    } else {
        outfile = "rust.lobster".to_string();
    }
    match File::create(&outfile) {
        Err(e) => panic!("Outfile: {}\n{}", &outfile, e),
        Ok(outfile) => {
            let mut outwriter = BufWriter::new(outfile);
            let _ = jout.write_pretty(&mut outwriter, 4);
        }
    }
}

struct _Teststruct {
    // lobster-trace: TEST.struct
    testfield: u32
}

impl _Teststruct {
    fn _increase(&mut self) {
        // lobster-trace: TEST.method
        self.testfield += 1;
    }
}
