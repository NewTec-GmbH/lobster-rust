//! # Visitor trait and RustVisitor.

use ra_ap_edition::Edition;
use ra_ap_syntax::{
    AstNode, NodeOrToken, SourceFile, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::{
    syntax_extensions::{Searchable, Visitable},
    traceable_node::{NodeKind, RustTraceableNode},
    utils::extract_path_attr::extract_path_attribute,
    utils::module_resolution::resolve_module_declaration,
    NodeLocation,
};

/// Visitor trait
///
/// Implementation of the Visitor trait is needed to visit structs implementing the Visitable trait.
/// The methods node_enter, node_exit and token_visit have to be defined and function like callbacks while traversing a Visitable struct.
/// The travel function will visit the root node given and recusively traverse the tree defined by it.
pub(crate) trait Visitor {
    fn node_enter(&mut self, node: &SyntaxNode);
    fn node_exit(&mut self, node: &SyntaxNode);
    fn token_visit(&mut self, token: &SyntaxToken);
    fn travel(&mut self, root: &SyntaxNode);
}

/// Visitor data to hold the visitors mutable state.
///
/// The visitor data includes whitespace data to keep track of the whitespace token information already parsed.
/// The node stack is used to track nested nodes in the tree and allows inferring context information about enclosing nodes, while new nested nodes and
/// tokens are being parsed. The node stack is also used to build a tree of RustTraceableNodes that can be accessed after the visitor is finished parsing by accessing the root node
/// from the stack.
struct VisitorData {
    whitespace_data: WhitespaceData,
    node_stack: Vec<RustTraceableNode>,
}

impl VisitorData {
    /// Get a reference to the root node.
    ///
    /// Returns some reference to the first node on the stack (the root node) if there is one.
    fn get_root(&self) -> Option<&RustTraceableNode> {
        self.node_stack.first()
    }

    /// Get a mutable reference to the root node.
    ///
    /// Returns some mutable reference to the first node on the stack (the root node) if there is one.
    fn _get_root_mut(&mut self) -> Option<&mut RustTraceableNode> {
        self.node_stack.first_mut()
    }
}

/// Whitespace data to track whitespace token info.
///
/// The whitespace data is used to keep track of the current line in the file and the char position of the last parsed linebreak in the file.
/// It should be updated when visiting WHITESPACE kind tokens. The data can then be used to provide accurate locations of functions and structs in the source file.
/// This is used because the SyntaxTree from ra_ap_syntax only tracks character ranges in the file, disregarding line information.
struct WhitespaceData {
    current_line: usize,
    last_linebrk: usize,
}

impl WhitespaceData {
    /// Calculate the position for a given SyntaxElement.
    ///
    /// Provides the line and the column for a given SyntaxElement.
    /// Tis is only correct if all WHITESPACE tokens before the element containing line breaks were already parsed to whitespace data!
    ///
    /// ### Parameters
    /// * `element` - SyntaxElement to calculate line and column for.
    ///
    /// ### Returns
    /// Tuple of line and column for the given element.
    fn calculate_element_location(&self, element: &SyntaxElement) -> (usize, usize) {
        let element_start = usize::from(element.text_range().start());
        let col = element_start - self.last_linebrk;
        (self.current_line, col)
    }
}

/// RustVisitor to traverse the syntax tree and gather RustTraceableNodes.
///
/// The RustVisitor implements the Visitor trait.
/// This allows traversing the SyntaxNodes of the SyntaxTree that were extended with the Visitable trait.
/// The node_enter, node_exit and token_visit funtions work as a mapping
/// from the nodes and tokes SyntaxKind to specific RustVisitor methods that define the handling of each node and token kind.
pub(crate) struct RustVisitor {
    /// Filepath to the source file (.rs) the visitor is parsing.
    filepath: PathBuf,
    /// Context String that is prepended to any tags and names found in this file.
    default_context: String,
    /// Visitor data to track the visitors internal state while traversing the SyntaxTree.
    vdata: VisitorData,
    /// Other visitors that are used to visit files that were included via module declarations in this visitors source file.
    module_visitors: Vec<RustVisitor>,
}

impl RustVisitor {
    /// Constructs a new RustVisitor.
    ///
    /// Constructs a new RustVisitor for the file given by the filepath.
    /// Default context is provided via the context parameter.
    ///
    /// ### Parameters
    /// * `filepath` - Path to the file the visitor shall parse.
    /// * `context` - Default context for the visitor, will be prepended to parsed names and tags. */
    ///
    /// ### Returns
    /// A Rustvisitor for the given file.
    pub(crate) fn new(filepath: PathBuf, context: String) -> Self {
        RustVisitor {
            filepath,
            default_context: context,
            vdata: VisitorData {
                whitespace_data: WhitespaceData {
                    current_line: 1,
                    last_linebrk: 0,
                },
                node_stack: Vec::new(),
            },
            module_visitors: Vec::new(),
        }
    }

    /// Builds a context from any enclosing nodes on the stack.
    ///
    /// Traverses the stack to find context nodes that hold context data.
    /// Combines the namespaces of the context data into one context String.
    ///
    /// ### Returns
    /// Some context string if any enclosing nodes hold context data, otherwise None.
    fn get_enclosing_context(&self) -> Option<String> {
        // Get reference to the implementation data of the latest Impl node.
        let nested_in: Vec<String> = self
            .vdata
            .node_stack
            .iter()
            .filter(|n| NodeKind::Context == n.kind)
            .map(|rtn| rtn.context_data.as_ref())
            .flatten()
            .map(|context_data| context_data.namespace.clone())
            .collect();

        if nested_in.len() > 0 {
            Some(nested_in.join("."))
        } else {
            None
        }
    }

    /// Get the filename of the file the visior is parsing.
    ///
    /// Extracts the filename from the filepath to the file the visitor is parsing.
    ///
    /// ### Returns
    /// String containing the filename.
    fn get_filename(&self) -> String {
        self.filepath
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    /// Parse the corresponding file for the RustVisitor.
    ///
    /// Reads the contents of the file pointed to by the filepath.
    /// Parses the contents of the file into a SyntaxTree.
    /// Traverses the tree by calling travel on the root node of the tree.
    /// Recursively also parses all included modules by calling .parse_file() of its module_visitors.
    pub(crate) fn parse_file(&mut self) {
        let text: String;
        match fs::read_to_string(&self.filepath) {
            Ok(t) => text = t,
            Err(e) => panic!("File: {:#?}\n{}", &self.filepath, e),
        }
        let parse = SourceFile::parse(&text, Edition::Edition2024);
        let tree: SourceFile = parse.tree();
        let root_node = tree.syntax();

        self.travel(root_node);

        for subvisitor in self.module_visitors.iter_mut() {
            subvisitor.parse_file();
        }
    }

    /// Resturns its own root node and the root nodes of all module_visitors.
    ///
    /// Gathers its own root_node (first on the stack) and the root nodes of all module visitors in a Vector.
    ///
    /// ### Returns
    /// Vecor of the root nodes.
    pub(crate) fn get_traceable_nodes(&mut self) -> Vec<RustTraceableNode> {
        let mut out_nodes: Vec<RustTraceableNode>;
        if 0 == self.vdata.node_stack.len() {
            out_nodes = Vec::new();
        } else {
            out_nodes = vec![self.vdata.node_stack.remove(0)];
        }

        for subvisitor in self.module_visitors.iter_mut() {
            out_nodes.append(&mut subvisitor.get_traceable_nodes());
        }

        out_nodes
    }

    /*********************** Node visit functions ***********************/

    /// Callback for source node enter.
    ///
    /// Puts the root node on the stack for the source file.
    ///
    /// ### Parameters
    /// * `source_node` - SyntaxNode of kind SOURCE. Top level node of a source file.
    fn enter_source(&mut self, source_node: &SyntaxNode) {
        let mut root_node = RustTraceableNode::from_node(source_node, String::new()).unwrap();
        root_node.name = self.get_filename();
        self.vdata.node_stack.push(root_node);
    }

    /// Callback for FN node enter.
    ///
    /// Parses function information for the given FN node.
    /// Determines location, context, name and builds and puts the RustTraceableNode on the node stack.
    ///
    /// ### Parameters
    /// * `fn_node` - SyntaxNode of kind FN.
    fn enter_fn(&mut self, fn_node: &SyntaxNode) {
        // Calculate position in file.
        let (line, col): (usize, usize);
        if let Some(fn_kw_token) = fn_node.get_tokens_kind(SyntaxKind::FN_KW).pop() {
            (line, col) = self
                .vdata
                .whitespace_data
                .calculate_element_location(&NodeOrToken::Token(fn_kw_token));
        } else {
            (line, col) = (self.vdata.whitespace_data.current_line, 0);
        }
        // Get default context + filepath as prefix.
        let filepath = self.vdata.get_root().unwrap().name.clone();
        let mut prefix;
        if self.default_context.is_empty() {
            prefix = self.get_filename();
        } else {
            prefix = self.default_context.clone() + "." + &self.get_filename();
        }
        let location = NodeLocation::from(filepath, Some(line), Some(col));

        // Check for enclosing context.
        if let Some(context_prefix) = self.get_enclosing_context() {
            prefix += ".";
            prefix += &context_prefix;
        }

        // Parse node.
        let node = RustTraceableNode::from_node_with_location(fn_node, location, prefix)
            .unwrap_or_else(|| {
                panic!(
                    "Could not parse fn at line {:#?}.",
                    self.vdata.whitespace_data.current_line
                )
            });
        self.vdata.node_stack.push(node);
    }

    /// Callback for FN node exit.
    ///
    /// Retrieves the node from the stack and appends it as a child to the enclosing node.
    ///
    /// ### Parameters
    /// * `_` - SyntaxNode of kind FN.
    fn exit_fn(&mut self, _: &SyntaxNode) {
        // Pop function node from stack and add it to its parent node
        let closed_fn = self.vdata.node_stack.pop().unwrap();

        if let Some(enclosing_node) = self.vdata.node_stack.last_mut() {
            enclosing_node.append_child(closed_fn);
        }
    }

    /// Callback for STRUCT node enter.
    ///
    /// Parses struct information for the given STRUCT node.
    /// Determines location, context, name and builds and puts the RustTraceableNode on the node stack.
    ///
    /// ### Parameters
    /// * `struct_node` - SyntaxNode of kind STRUCT.
    fn enter_struct(&mut self, struct_node: &SyntaxNode) {
        // Calculate position in file.
        let (line, col): (usize, usize);
        if let Some(struct_kw_token) = struct_node.get_tokens_kind(SyntaxKind::STRUCT_KW).pop() {
            (line, col) = self
                .vdata
                .whitespace_data
                .calculate_element_location(&NodeOrToken::Token(struct_kw_token));
        } else {
            (line, col) = (self.vdata.whitespace_data.current_line, 1);
        }
        // Get default context + filepath as prefix.
        let filepath = self.vdata.get_root().unwrap().name.clone();
        let mut prefix;
        if self.default_context.is_empty() {
            prefix = self.get_filename();
        } else {
            prefix = self.default_context.clone() + "." + &self.get_filename();
        }
        let location = NodeLocation::from(filepath, Some(line), Some(col));

        // Check for enclosing context.
        if let Some(context_prefix) = self.get_enclosing_context() {
            prefix += ".";
            prefix += &context_prefix;
        }

        // Parse node.
        let node = RustTraceableNode::from_node_with_location(struct_node, location, prefix)
            .unwrap_or_else(|| {
                panic!(
                    "Could not parse struct at line {:#?}.",
                    self.vdata.whitespace_data.current_line
                )
            });
        self.vdata.node_stack.push(node);
    }

    /// Callback for STRUCT node exit.
    ///
    /// Retrieves the node from the stack and appends it as a child to the enclosing node.
    ///
    /// ### Parameters
    /// * `_` - SyntaxNode of kind SRUCT.
    fn exit_struct(&mut self, _: &SyntaxNode) {
        // Pop struct node from stack and add it to its parent node
        let closed_struct = self.vdata.node_stack.pop().unwrap();

        if let Some(enclosing_node) = self.vdata.node_stack.last_mut() {
            enclosing_node.append_child(closed_struct);
        }
    }

    /// Callback forIMPL node enter.
    ///
    /// Parses context information for the given IMPL node.
    /// Puts the CONTEXT RustTraceableNode on the node stack.
    ///
    /// ### Parameters
    /// * `impl_node` - SyntaxNode of kind IMPL.
    fn enter_impl(&mut self, impl_node: &SyntaxNode) {
        let node = RustTraceableNode::from_node(impl_node, String::new()).unwrap();
        self.vdata.node_stack.push(node);
    }

    /// Callback for IMPL node exit.
    ///
    /// Retrieves the node from the stack and appends it as a child to the enclosing node.
    ///
    /// ### Parameters
    /// * `_` - SyntaxNode of kind IMPL.
    fn exit_impl(&mut self, _impl_node: &SyntaxNode) {
        let closed_impl = self.vdata.node_stack.pop().unwrap();
        if let Some(enclosing_node) = self.vdata.node_stack.last_mut() {
            enclosing_node.append_child(closed_impl);
        }
    }

    /// Callback for MODULE node enter.
    ///
    /// Parses information for the given MODULE node.
    /// Determines if the node represents a module declaration.
    /// If so, the module is resolved to a file path and a module visitor for the new source file is created.
    /// If the node instead represents a local module definition, it is parsed to a context node and put on the stack.
    ///
    /// ### Parameters
    /// * `mod_node` - SyntaxNode of kind MODULE.
    fn enter_module(&mut self, mod_node: &SyntaxNode) {
        let name_node = mod_node.get_child_kind(SyntaxKind::NAME).unwrap();
        let last_child = mod_node.children_with_tokens().last().unwrap();
        match last_child {
            NodeOrToken::Token(t) => {
                if t.kind() == SyntaxKind::SEMICOLON {
                    // Found module declaration. Resolve to corresponding file.
                    let attrs = mod_node.get_children_kind(SyntaxKind::ATTR);
                    let path_attributes: Vec<PathBuf> = attrs
                        .iter()
                        .map(|att| extract_path_attribute(att))
                        .flatten()
                        .collect();

                    if let Some(module_path) = path_attributes.first() {
                        // Resolve the path given by the path attribute.
                        self.module_visitors.push(RustVisitor::new(
                            self.filepath.parent().unwrap().join(module_path),
                            "".to_string(),
                        ));
                    } else {
                        // Follow the standard module declaration resolution.
                        if let Some((modpath, context)) = resolve_module_declaration(
                            &self.filepath,
                            &name_node.text().to_string(),
                        ) {
                            let nested_context;
                            if context.is_empty() {
                                nested_context = self.default_context.clone()
                            } else {
                                if self.default_context.is_empty() {
                                    nested_context = context;
                                } else {
                                    nested_context = self.default_context.clone() + "." + &context;
                                }
                            }
                            self.module_visitors
                                .push(RustVisitor::new(modpath, nested_context));
                        }
                    }
                }
            }
            NodeOrToken::Node(n) => {
                if n.kind() == SyntaxKind::ITEM_LIST {
                    // Found local module. Parse as Context.
                    let context_node =
                        RustTraceableNode::from_node(mod_node, String::new()).unwrap();
                    self.vdata.node_stack.push(context_node);
                }
            }
        }
    }

    /// Callback for MODULE node exit.
    ///
    /// Retrieves the node from the stack and appends it as a child to the enclosing node.
    /// As only MODULE nodes representing a local module definitions are put on the node stack,
    /// it is first checked if this applies to the node. Otherwise nothing is done.
    ///
    /// ### Parameters
    /// * `mod_node` - SyntaxNode of kind MODULE.
    fn exit_module(&mut self, mod_node: &SyntaxNode) {
        // Only pop the last Context if this mod_node was added as a Context.
        match mod_node.children_with_tokens().last().unwrap() {
            NodeOrToken::Node(n) => {
                if n.kind() == SyntaxKind::ITEM_LIST {
                    let closed_context = self.vdata.node_stack.pop().unwrap();
                    if let Some(enclosing_node) = self.vdata.node_stack.last_mut() {
                        enclosing_node.append_child(closed_context);
                    }
                }
            }
            _ => (),
        }
    }

    /*********************** Token visit functions ***********************/

    /// Callback for WHITESPACE token visit.
    ///
    /// Parsed the contents of the WHITESPACE token to track linebreaks in the file.
    ///
    /// ### Parameters
    /// * `whitespace_token` - Token of kind WHITESPACE.
    fn visit_whitespace(&mut self, whitespace_token: &SyntaxToken) {
        // Update whitespace data to hold current line and charpos of last linebreak
        let ws_data = &mut self.vdata.whitespace_data;

        let linebreaks = whitespace_token
            .text()
            .chars()
            .filter(|c| '\n' == *c)
            .count();
        ws_data.current_line += linebreaks;

        if let Some((lbpos, _)) = whitespace_token
            .text()
            .char_indices()
            .filter(|(_, c)| '\n' == *c)
            .last()
        {
            ws_data.last_linebrk = usize::from(whitespace_token.text_range().start()) + lbpos;
        }
    }

    /// Callback for COMMENT token visit.
    ///
    /// Parsed the contents of the COMMENT token.
    /// Possible requirement references or justifications are found by regex application.
    /// If a reference or justification is found, it is added to the enclosing node (from the node stack).
    ///
    /// ### Parameters
    /// * `comment_token` - Token of kind COMMENT.
    fn visit_comment(&mut self, comment_token: &SyntaxToken) {
        // Parse comment for lobster trace or justification annotations
        if let Some(cnode) = self.vdata.node_stack.last_mut() {
            let trace_re = Regex::new(r"///?.*lobster-trace: (?<ref>[[:alnum:]\._-]+).*").unwrap();
            let just_re =
                Regex::new(r"///?.*lobster-exclude: (?<just>[[:alnum:]\._-]+).*").unwrap();

            if let Some(cap) = trace_re.captures(comment_token.text()) {
                if let Some(refmatch) = cap.name("ref") {
                    let mut refstring = refmatch.as_str().to_string();
                    refstring.insert_str(0, "req ");
                    cnode.refs.push(refstring);
                }
            }
            if let Some(cap) = just_re.captures(comment_token.text()) {
                if let Some(justmatch) = cap.name("just") {
                    let juststring = justmatch.as_str().to_string();
                    cnode.just.push(juststring);
                }
            }
        }
    }
}

impl Visitor for RustVisitor {
    /// Callback for node enter.
    ///
    /// Calls the specific callback for the SyntaxKind of the node.
    ///
    /// ### Parameters
    /// * `node` - Syntax node that is visited.
    fn node_enter(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::SOURCE_FILE => self.enter_source(node),
            SyntaxKind::FN => self.enter_fn(node),
            SyntaxKind::STRUCT => self.enter_struct(node),
            SyntaxKind::IMPL => self.enter_impl(node),
            SyntaxKind::MODULE => self.enter_module(node),
            _ => (),
        }
    }

    /// Callback for node exit.
    ///
    /// Calls the specific callback for the SyntaxKind of the node.
    ///
    /// ### Parameters
    /// * `node` - Syntax node that was visited.
    fn node_exit(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::FN => self.exit_fn(node),
            SyntaxKind::STRUCT => self.exit_struct(node),
            SyntaxKind::IMPL => self.exit_impl(node),
            SyntaxKind::MODULE => self.exit_module(node),
            _ => (),
        }
    }

    /// Callback for token visit.
    ///
    /// Calls the specific callback for the SyntaxKind of the token.
    ///
    /// ### Parameters
    /// * `token` - Syntax token that is visited.
    fn token_visit(&mut self, token: &SyntaxToken) {
        match token.kind() {
            SyntaxKind::WHITESPACE => self.visit_whitespace(token),
            SyntaxKind::COMMENT => self.visit_comment(token),
            _ => (),
        }
    }

    /// Visit the source tree defined by the root node.
    ///
    /// Calls visit on the root node. As SyntaxNode implements the Visitable trait,
    /// this is expected to traverse the SyntaxTree defined by the root node.
    ///
    /// ### Parameters
    /// * `root` - Syntax node.
    fn travel(&mut self, root: &SyntaxNode) {
        root.visit(self);
    }
}
