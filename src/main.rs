use std::{env, fs::{self, File}};
use std::io::BufWriter;
use location::{NodeLocation, FileReference};
use ra_ap_syntax::{AstNode, SourceFile};
use ra_ap_edition::Edition;
use json::{object::Object, JsonValue};
use visitor::{RustVisitor, Visitor};

mod location;
mod visitor;
mod traceable_node;
mod syntax_extensions;

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename;
    if args.len() > 1{
        filename = args[1].clone();
    } else {
        filename = "./src/main.rs".to_string();
    }

    let text = fs::read_to_string(&filename).expect("File does not exist");
    let parse = SourceFile::parse(&text, Edition::Edition2024);
    let tree: SourceFile = parse.tree();
    let root_node = tree.syntax();

    let mut visitor = RustVisitor::new(filename);

    visitor.travel(&root_node);

    let module = visitor.vdata.get_root_mut().unwrap();
    
    let mut data: Vec<JsonValue> = Vec::new();
    module.to_lobster(&mut data);

    imjustherefortesting();

    // add data
    let mut jout = JsonValue::Object(Object::new());
    let _ = jout.insert("data", data);

    // write json to output file
    let outfile = File::create("rust.lobster").unwrap();
        let mut outwriter = BufWriter::new(outfile);
    let _ = jout.write_pretty(&mut outwriter, 4);
}

fn imjustherefortesting() {
    // some dumb function we might want to trace
    // lobster-trace: DAMN.gj
    println!("Test fuction called!");
}
