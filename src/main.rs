use clap::Parser;
use json::{object::Object, JsonValue};
use location::{FileReference, NodeLocation};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use visitor::RustVisitor;

mod includes;
mod location;
mod syntax_extensions;
mod traceable_node;
mod visitor;

fn main() {
    let args = args::Cli::parse();

    let filename;
    if args.lib {
        filename = Path::new("lib.rs");
    } else {
        filename = Path::new("main.rs");
    }

    let filepath = Path::new(&args.dir).join(filename);

    let mut visitor = RustVisitor::new(filepath);
    visitor.parse_file();

    let modules = visitor.get_traceable_nodes();

    let mut data: Vec<JsonValue> = Vec::new();
    for module in modules {
        module.to_lobster(&mut data);
    }

    // add data
    let mut jout = JsonValue::Object(Object::new());
    let _ = jout.insert("data", data);
    let _ = jout.insert("generator", "lobster-rust");
    let _ = jout.insert("schema", "lobster-imp-trace");
    let _ = jout.insert("version", 3);

    // write json to output file
    let outfile: &Path = Path::new(&args.out);
    match File::create(&outfile) {
        Err(e) => panic!("Outfile: {:#?}\n{}", &outfile, e),
        Ok(outfile) => {
            let mut outwriter = BufWriter::new(outfile);
            let _ = jout.write_pretty(&mut outwriter, 4);
        }
    }
}

struct _Teststruct {
    // lobster-trace: TEST.struct
    testfield: u32,
}

impl _Teststruct {
    fn _increase(&mut self) {
        // lobster-trace: TEST.method
        self.testfield += 1;
    }
}

#[allow(unused_parens)]
mod args {
    use clap::Parser;
    #[derive(Parser)]
    #[command(version, about, long_about = None)]
    pub(super) struct Cli {
        /// Directory of main.rs (or lib.rs).
        #[arg(default_value_t = ("./src/".to_string()))]
        pub(super) dir: String,

        /// Output directory for the .lobster file.
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
