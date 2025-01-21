/// RustTraceableNode to hold information from parsing the syntax tree.
/// Also holds auxiliary structs and functions.

use std::fmt::Display;
use ra_ap_syntax::{SyntaxNode, SyntaxKind};
use json::{object::Object, JsonValue};

use crate::{NodeLocation, FileReference, syntax_extensions::Searchable};


#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NodeKind {
    Module,
    Struct,
    Enum,
    Trait,
    Function,
    Impl
}

impl NodeKind {
    pub(crate) fn to_string(&self) -> &str{
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
pub(crate) struct RustTraceableNode {
    // lobster-trace: HOLD.refs
    pub(crate) name: String,
    pub(crate)kind: NodeKind,
    pub(crate) location: NodeLocation,
    pub(crate) children: Vec<RustTraceableNode>,
    pub(crate) just: Vec<String>,
    pub(crate) refs: Vec<String>,
    pub(crate) impl_data: Option<ImplementationData>
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

    pub(crate) fn from_node(node: &SyntaxNode, prefix: String) -> Option<Self> {
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
                    let name =  prefix + "." + &name_node.text().to_string();
                    Some(RustTraceableNode::new(name, NodeLocation::File(location), node_kind))
                },
                NodeKind::Module => {
                    Some(RustTraceableNode::new("FILE".to_string(), NodeLocation::File(location), node_kind))
                },
                NodeKind::Struct => {
                    let name_node = node.get_child_kind(SyntaxKind::NAME)?;
                    let name = prefix+ "."  + &name_node.text().to_string();
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

    pub(crate) fn from_node_with_location(node: &SyntaxNode, location: NodeLocation, prefix: String) -> Option<Self> {
        let mut new_node = Self::from_node(node, prefix);
        new_node.as_mut().map(| n| n.location = location);
        new_node
    }

    pub(crate) fn append_child(&mut self, child: RustTraceableNode) { 
        self.children.push(child);
    }

    pub(crate) fn to_lobster(&self, items: &mut Vec<JsonValue>) {
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
pub(crate) struct ImplementationData {
    pub(crate) target: String,
    pub(crate) _trait_imp: Option<String>
}

impl ImplementationData {
    fn new(target: String, trait_imp: Option<String>) -> Self {
        ImplementationData {
            target,
            _trait_imp: trait_imp
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
