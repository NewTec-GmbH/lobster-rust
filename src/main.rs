use std::{env, fmt::Display, fs::{self, File}};
use std::io::BufWriter;
use location::{NodeLocation, FileReference};
use ra_ap_syntax::{AstNode, Direction, NodeOrToken, SourceFile, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxElement};
use ra_ap_edition::Edition;
use json::{self, object::Object, JsonValue};
use visitor::{RustVisitor, Visitor};

mod location;
mod visitor;

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

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeKind {
    Module,
    Struct,
    Enum,
    Trait,
    Function,
    Impl
}

impl NodeKind {
    fn to_string(&self) -> &str{
        match self {
            NodeKind::Module => "Module",
            NodeKind::Struct => "Struct",
            NodeKind::Enum => "Enum",
            NodeKind::Trait => "Trait",
            NodeKind::Function => "Function",
            NodeKind::Impl => "Impl"
        }
    }
}

#[derive(Debug, Clone)]
struct RustTraceableNode {
    // lobster-trace: HOLD.refs
    name: String,
    kind: NodeKind,
    location: NodeLocation,
    children: Vec<RustTraceableNode>,
    just: Vec<String>,
    refs: Vec<String>,
    impl_data: Option<ImplementationData>
}

impl RustTraceableNode {
    fn new(name: String, location: NodeLocation, kind: NodeKind) -> RustTraceableNode {
        RustTraceableNode {
            name,
            kind,
            location,
            children: Vec::new(),
            just: Vec::new(),
            refs: Vec::new(),
            impl_data: None
        }
    }

    fn from_node(node: &SyntaxNode) -> Option<Self> {
        let location = FileReference {
            filename: "main.rs".to_string(),
            line: None,
            column: None
        };
        
        // lobster-trace: PARSE.nodes
        if let Some(node_kind) = syntax_kind_to_node_kind(node.kind()) {
            match node_kind {
                NodeKind::Function => {
                    let name_node = node.get_child_kind(SyntaxKind::NAME)?;
                    let name = name_node.text().to_string();
                    Some(RustTraceableNode::new(name, NodeLocation::File(location), node_kind))
                },
                NodeKind::Module => {
                    Some(RustTraceableNode::new("FILE".to_string(), NodeLocation::File(location), node_kind))
                },
                NodeKind::Struct => {
                    let name_node = node.get_child_kind(SyntaxKind::NAME)?;
                    let name = name_node.text().to_string();
                    Some(RustTraceableNode::new(name, NodeLocation::File(location), node_kind))
                },
                NodeKind::Impl => {
                    let traceable_node: Option<RustTraceableNode>;
                    // get target and optional trait that gets implemented
                    let path_nodes = node.get_childs_kind(SyntaxKind::PATH_TYPE);

                    // either impl STRUCTNAME or impl TRAITNAME for STRUCTNAME
                    if path_nodes.len() == 2 {
                        // expect the for kw to be present
                        let for_kw = node.get_tokens_kind(SyntaxKind::FOR_KW);
                        if for_kw.len() == 0 {
                            traceable_node = None;
                        } else {
                            let traitref = path_nodes[0].text().to_string();
                            let structref = path_nodes[1].text().to_string();
                            let impl_data = ImplementationData::new(structref, Some(traitref));
                            let mut new_node = RustTraceableNode::new("Impl".to_string(), NodeLocation::File(location), node_kind);
                            new_node.impl_data = Some(impl_data);
                            traceable_node = Some(new_node);
                        }
                    } else if path_nodes.len() == 1 {
                        let structref = path_nodes[0].text().to_string();
                        let impl_data = ImplementationData::new(structref, None);
                        let mut new_node = RustTraceableNode::new("Impl".to_string(), NodeLocation::File(location), node_kind);
                        new_node.impl_data = Some(impl_data);
                        traceable_node = Some(new_node);
                    } else {
                        println!("No path nodes in impl???");
                        traceable_node =  None;
                    }
                    traceable_node
                }
                _ => None
            }

        } else {
            None
        }
    }

    fn from_node_with_location(node: &SyntaxNode, location: NodeLocation) -> Option<Self> {
        let mut new_node = Self::from_node(node);
        new_node.as_mut().map(| n| n.location = location);
        new_node
    }

    fn append_child(&mut self, child: RustTraceableNode) { 
        self.children.push(child);
    }

    fn to_lobster(&self, items: &mut Vec<JsonValue>) {
        match self.kind {
            NodeKind::Module => {
                for child in &self.children {
                    child.to_lobster(items);
                }
            },
            NodeKind::Function => {
                items.push(JsonValue::from(self));
            },
            NodeKind::Struct => {
                items.push(JsonValue::from(self));
            },
            NodeKind::Impl => {
                for child in &self.children {
                    child.to_lobster(items);
                }
            }
            _ => (),
        }
    }
}

impl Display for RustTraceableNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Node {:?} {} at {}", self.kind, self.name, self.location.to_string())?;
        Ok(())
    }
}

impl From<&RustTraceableNode> for JsonValue {
    fn from(node: &RustTraceableNode) -> JsonValue {
        // idk if we really want to do this
        let mut json_out = JsonValue::Object(Object::new());
        let _ = json_out.insert("tag", format!("rust {}", node.name));
        let _ = json_out.insert("name", format!("{}", node.name));
        let _ = json_out.insert("location", JsonValue::from(&node.location));
        let _ = json_out.insert("messages", JsonValue::Array(Vec::new()));
        let _ = json_out.insert("just_up", JsonValue::Array(node.just.iter().map(|j| JsonValue::String(j.to_string())).collect()));
        let _ = json_out.insert("just_down", JsonValue::Array(Vec::new()));
        let _ = json_out.insert("just_global", JsonValue::Array(Vec::new()));
        let _ = json_out.insert("refs", JsonValue::Array(node.refs.iter().map(|r| JsonValue::String(r.to_string())).collect()));
        let _ = json_out.insert("language", "Rust");
        let _ = json_out.insert("kind", node.kind.to_string());
        json_out
    }
}

#[derive(Debug, Clone)]
struct ImplementationData {
    target: String,
    trait_imp: Option<String>
}

impl ImplementationData {
    fn new(target: String, trait_imp: Option<String>) -> Self {
        ImplementationData {
            target,
            trait_imp
        }
    }
}
trait Searchable {
    // extends the type with practical filtering options
    fn get_child_kind(&self, kind: SyntaxKind) -> Option<SyntaxNode>;
    fn get_childs_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode>;
    fn get_tokens_kind(&self, kind: SyntaxKind) -> Vec<SyntaxToken>;
}

impl Searchable for SyntaxElement {
    fn get_child_kind(&self, kind: SyntaxKind) -> Option<SyntaxNode> {
        match self {
            NodeOrToken::Node(n) => {
                n.get_child_kind(kind)
            },
            _ => None
        }   
    }

    fn get_childs_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode> {
        match self {
            NodeOrToken::Node(n) => {
                n.get_childs_kind(kind)
            },
            _ => Vec::new()
        }
    }

    fn get_tokens_kind(&self, kind: SyntaxKind) -> Vec<SyntaxToken> {
        match self {
            NodeOrToken::Node(n) => {
                n.get_tokens_kind(kind)
            },
            _ => Vec::new()
        }  
    }
}

impl Searchable for SyntaxNode {
    fn get_child_kind(&self, kind: SyntaxKind) -> Option<SyntaxNode> {
        self.children().filter(|c| kind == c.kind()).next()
    }

    fn get_childs_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode> {
        self.children().filter(|c| kind == c.kind()).collect()
    }

    fn get_tokens_kind(&self, kind: SyntaxKind) -> Vec<SyntaxToken> {
        if let Some(ft) = self.first_token() {
            ft.siblings_with_tokens(Direction::Next).filter_map(|node_or_token| match node_or_token {
                NodeOrToken::Node(_) => None,
                NodeOrToken::Token(t) => {
                    if kind == t.kind() {
                        Some(t)
                    } else {
                        None
                    }
                }
            }).collect()
        } else {
            Vec::new()
        }
        
    }
}

fn syntax_kind_to_node_kind(kind: SyntaxKind) -> Option<NodeKind> {
    match kind {
        SyntaxKind::FN => Some(NodeKind::Function),
        SyntaxKind::SOURCE_FILE => Some(NodeKind::Module),
        SyntaxKind::STRUCT => Some(NodeKind::Struct),
        SyntaxKind::ENUM => Some(NodeKind::Enum),
        SyntaxKind::TRAIT => Some(NodeKind::Trait),
        SyntaxKind::IMPL => Some(NodeKind::Impl),
        _ => None
    }
}
