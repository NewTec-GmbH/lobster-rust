//! RustTraceableNode to hold information from parsing the syntax tree and auxiliary structs and fuctions.

use json::{object::Object, JsonValue};
use ra_ap_syntax::{SyntaxKind, SyntaxNode};
use std::fmt::Display;

use crate::{location::FileReference, syntax_extensions::Searchable};

/// Enum to define the different kinds of RustTraceableNodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NodeKind {
    /// The node is representing a source file.
    Source,
    /// The node is representing a struct.
    Struct,
    /// The node is representing an enum.
    Enum,
    /// The node is representing a trait.
    Trait,
    /// The node is representing a function.
    Function,
    /// The node is representing some context.
    Context,
}

impl NodeKind {
    /// Returns a &str representing the NodeKind.
    pub(crate) fn to_str(&self) -> &str {
        match self {
            NodeKind::Source => "Module",
            NodeKind::Struct => "Struct",
            NodeKind::Enum => "Enum",
            NodeKind::Trait => "Trait",
            NodeKind::Function => "Function",
            NodeKind::Context => "Context",
        }
    }
}

/// Struct to hold information about parsed syntax nodes.
/// This node can be converted to data in the lobster common interchange format.
#[derive(Debug, Clone)]
pub(crate) struct RustTraceableNode {
    // lobster-trace: HOLD.refs
    /// The name of the node, produced from context and parsed information.
    /// The name is also used to construct the tracing tag when converting to the lobster common interchange format.
    pub(crate) name: String,
    /// The kind of the node.
    pub(crate) kind: NodeKind,
    /// The location of the node.
    pub(crate) location: FileReference,
    /// Children of the node.
    pub(crate) children: Vec<RustTraceableNode>,
    /// Parsed justifications.
    pub(crate) just: Vec<String>,
    /// Parsed references to requirements.
    pub(crate) refs: Vec<String>,
    /// Optional context data to track local modules or impl blocks and resolve full names.
    pub(crate) context_data: Option<ContextData>,
}

impl RustTraceableNode {
    /// Constructs a new RTN.
    ///
    /// Constructs a new RustTraceableNode with the given name, location and kind.
    ///
    /// ### Parameters
    /// * `name` - Name of the new RTN.
    /// * `location` - Location of the new RTN.
    /// * `kind` - NodeKind of the new RTN.
    ///
    /// ### Returns
    /// A RustTraceableNode.
    fn new(name: String, location: FileReference, kind: NodeKind) -> RustTraceableNode {
        RustTraceableNode {
            name,
            kind,
            location,
            children: Vec::new(),
            just: Vec::new(),
            refs: Vec::new(),
            context_data: None,
        }
    }

    /// Constructs a new RTN from a SyntaxNode.
    ///
    /// Constructs a new RustTraceableNode from a given ra_ap_syntax SyntaxNode.
    ///
    /// ### Parameters
    /// * `node` - SyntaxNode that should be parsed to a corresponding RTN.
    /// * `prefix` - Prefix String to prepend to name and tag.
    ///
    /// ### Returns
    /// Some RustTraceableNode if parsing was sucessful, None otherwise.
    pub(crate) fn from_node(node: &SyntaxNode, prefix: String) -> Option<Self> {
        let location = FileReference::new_default();

        // lobster-trace: PARSE.nodes
        // Node handling is dependent on SyntaxKind of the SyntaxNode.
        if let Some(node_kind) = syntax_kind_to_node_kind(node.kind()) {
            match node_kind {
                NodeKind::Function => {
                    let name_node = node.get_child_kind(SyntaxKind::NAME)?;
                    let name = prefix + "." + &name_node.text().to_string();
                    Some(RustTraceableNode::new(name, location, node_kind))
                }
                NodeKind::Source => Some(RustTraceableNode::new(
                    "FILE".to_string(),
                    location,
                    node_kind,
                )),
                NodeKind::Struct => {
                    let name_node = node.get_child_kind(SyntaxKind::NAME)?;
                    let name = prefix + "." + &name_node.text().to_string();
                    Some(RustTraceableNode::new(name, location, node_kind))
                }
                NodeKind::Context => match node.kind() {
                    // IMPL and MODULE node conversion are done in separate functions to keep code simpler.
                    SyntaxKind::IMPL => RustTraceableNode::from_impl_node(node),
                    SyntaxKind::MODULE => RustTraceableNode::from_module_node(node),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }

    /// Constructs a new RTN from an IMPL SyntaxNode.
    ///
    /// Constructs a new RustTraceableNode from a given ra_ap_syntax SyntaxNode of IMPL SyntaxKind.
    /// The impl node is searched for path nodes, defining which struct is being implemented for,
    /// and optionally which trait is being implemented.
    /// This information is converted to context data that can be used while parsing enclosed nodes.
    ///
    /// ### Parameters
    /// * `node` - SyntaxNode that should be parsed to a context RTN.
    ///
    /// ### Returns
    /// Some RustTraceableNode if parsing was sucessful, None otherwise.
    fn from_impl_node(node: &SyntaxNode) -> Option<Self> {
        // Get target (the struct the impl is for) and optional trait that gets implemented.
        let path_nodes = node.get_children_kind(SyntaxKind::PATH_TYPE);

        // Either impl STRUCTNAME or impl TRAITNAME for STRUCTNAME.
        if path_nodes.len() == 2 {
            // Expect the for kw to be present when a trait is implemented (2 path nodes).
            let for_kw = node.get_tokens_kind(SyntaxKind::FOR_KW);
            if for_kw.len() == 0 {
                None
            } else {
                // Parse to context data.
                let traitref = path_nodes[0].text().to_string();
                let structref = path_nodes[1].text().to_string();
                let impl_data = ContextData::new(structref, Some(traitref));
                let mut new_node = RustTraceableNode::new(
                    "Impl".to_string(),
                    FileReference::new_default(),
                    NodeKind::Context,
                );
                new_node.context_data = Some(impl_data);
                Some(new_node)
            }
        } else if path_nodes.len() == 1 {
            // Parse to context data.
            let structref = path_nodes[0].text().to_string();
            let impl_data = ContextData::new(structref, None);
            let mut new_node = RustTraceableNode::new(
                "Impl".to_string(),
                FileReference::new_default(),
                NodeKind::Context,
            );
            new_node.context_data = Some(impl_data);
            Some(new_node)
        } else {
            // No path nodes or 3+, fail parsing.
            None
        }
    }

    /// Constructs a new RTN from an MODULE SyntaxNode.
    ///
    /// Constructs a new RustTraceableNode from a given ra_ap_syntax SyntaxNode of MODULE SyntaxKind.
    /// The module node is parsed to context data that is used when parsing enclodes nodes.
    ///
    /// ### Parameters
    /// * `node` - SyntaxNode that should be parsed to a context RTN.
    ///
    /// ### Returns
    /// Some RustTraceableNode if parsing was sucessful, None otherwise.
    fn from_module_node(node: &SyntaxNode) -> Option<Self> {
        let name_node = node.get_child_kind(SyntaxKind::NAME).unwrap();
        let mut new_node = RustTraceableNode::new(
            name_node.text().to_string(),
            FileReference::new_default(),
            NodeKind::Context,
        );
        let context = ContextData::new(name_node.text().to_string(), None);
        new_node.context_data = Some(context);
        Some(new_node)
    }

    /// Constructs a new RTN from a SyntaxNode and location.
    ///
    /// Constructs a new RustTraceableNode from a given ra_ap_syntax SyntaxNode with the given location.
    /// Helper function that calls from_node and adds the given location, resulting in cleaner code.
    ///
    /// ### Parameters
    /// * `node` - SyntaxNode that should be parsed to a corresponding RTN.
    /// * `location` - FileReference for the new RTN.
    /// * `prefix` - Prefix String to prepend to name and tag.
    ///
    /// ### Returns
    /// Some RustTraceableNode if parsing was sucessful, None otherwise.
    pub(crate) fn from_node_with_location(
        node: &SyntaxNode,
        location: FileReference,
        prefix: String,
    ) -> Option<Self> {
        let mut new_node = Self::from_node(node, prefix);
        new_node.as_mut().map(|n| n.location = location);
        new_node
    }

    /// Appends RustTraceableNode as a child.
    ///
    /// Pushes the given RTN to the children vector of this RTN.
    ///
    /// ### Parameters
    /// * `child` - RTN to add to children.
    pub(crate) fn append_child(&mut self, child: RustTraceableNode) {
        self.children.push(child);
    }

    /// Converst to lobster format and adds itselfs to the items.
    ///
    /// Converts the RustTraceableNode to the lobster common interchange format.
    /// This is either done by converting the node itself (done via the JsonValue::From implementation),
    /// or by converting and adding all of the nodes children, depending on node kind.
    ///
    /// ### Parameters
    /// * `items` - Vector of already converted items, new converted items will be appended.
    pub(crate) fn to_lobster(&self, items: &mut Vec<JsonValue>) {
        match self.kind {
            NodeKind::Source => {
                for child in &self.children {
                    child.to_lobster(items);
                }
            }
            NodeKind::Function => {
                items.push(JsonValue::from(self));
            }
            NodeKind::Struct => {
                items.push(JsonValue::from(self));
            }
            NodeKind::Context => {
                for child in &self.children {
                    child.to_lobster(items);
                }
            }
            _ => (),
        }
    }
}

/// Implement Display for RustTraceableNode.
impl Display for RustTraceableNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Node {} {} at {}",
            self.kind.to_str(),
            self.name,
            self.location.to_string()
        )?;
        Ok(())
    }
}

/// Implement JsonValue::from(node: &RustTraceableNode)
///
/// This allows conversion from a RTN to a JsonValue.
impl From<&RustTraceableNode> for JsonValue {
    /// Convert RTN to a JsonValue.
    ///
    /// Parse a JsonValue from a RustTraceableNode.
    /// This conversion returns json in the form of a data item in the lobster common interchange format.
    /// The relevant fields of the RTN are parsed to the corresponding json fields.
    /// Fields in the interchange format without any relevance are added with no data (or as empty lists.)
    ///
    /// ### Parameters
    /// * `node` - RustTraceableNode to convert to JsonValue.
    ///
    /// ### Returns Json object holding the RTN data in lobser common interchange format.
    fn from(node: &RustTraceableNode) -> JsonValue {
        // idk if we really want to do this
        let mut json_out = JsonValue::Object(Object::new());
        let _ = json_out.insert("tag", format!("rust {}", node.name));
        let _ = json_out.insert("name", format!("{}", node.name));
        let _ = json_out.insert("location", JsonValue::from(&node.location));
        let _ = json_out.insert("messages", JsonValue::Array(Vec::new()));
        let _ = json_out.insert(
            "just_up",
            JsonValue::Array(
                node.just
                    .iter()
                    .map(|j| JsonValue::String(j.to_string()))
                    .collect(),
            ),
        );
        let _ = json_out.insert("just_down", JsonValue::Array(Vec::new()));
        let _ = json_out.insert("just_global", JsonValue::Array(Vec::new()));
        let _ = json_out.insert(
            "refs",
            JsonValue::Array(
                node.refs
                    .iter()
                    .map(|r| JsonValue::String(r.to_string()))
                    .collect(),
            ),
        );
        let _ = json_out.insert("language", "Rust");
        let _ = json_out.insert("kind", node.kind.to_str());
        json_out
    }
}

/// Holds namespace and optional trait information.
#[derive(Debug, Clone)]
pub(crate) struct ContextData {
    pub(crate) namespace: String,
    pub(crate) _trait_imp: Option<String>,
}

impl ContextData {
    /// Construct new context data.
    ///
    /// ### Parameters
    /// * `namespace` - String to represent some enclosing namespace, be it a local module name or the target struct name of an impl block.
    /// * `trait_imp` - Optional name of the trait being implemented (for impl blocks that implement a trait for a target struct).
    ///
    /// ### Returns
    /// The newly constructed context data.
    fn new(namespace: String, trait_imp: Option<String>) -> Self {
        ContextData {
            namespace,
            _trait_imp: trait_imp,
        }
    }
}

/// Convert SyntaxKind to NodeKind.
///
/// Converts the SyntaxKind of a SyntaxNode from the ra_ap_syntax crate to the corresponding NodeKind of a RustTraceableNode.
///
/// ### Parameters
/// * `kind` - SyntaxKind to find corresponding NodeKind for.
///
/// ### Returns
/// Some NodeKind if a corresponding NodeKind exists, otherwise None.
fn syntax_kind_to_node_kind(kind: SyntaxKind) -> Option<NodeKind> {
    match kind {
        SyntaxKind::FN => Some(NodeKind::Function),
        SyntaxKind::SOURCE_FILE => Some(NodeKind::Source),
        SyntaxKind::STRUCT => Some(NodeKind::Struct),
        SyntaxKind::ENUM => Some(NodeKind::Enum),
        SyntaxKind::TRAIT => Some(NodeKind::Trait),
        SyntaxKind::IMPL => Some(NodeKind::Context),
        SyntaxKind::MODULE => Some(NodeKind::Context),
        _ => None,
    }
}
