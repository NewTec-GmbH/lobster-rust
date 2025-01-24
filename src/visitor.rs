/// RustVisitor to traverse the syntax tree and gather data
use ra_ap_edition::Edition;
use ra_ap_syntax::{
    AstNode, NodeOrToken, SourceFile, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::{
    includes::resolve_include,
    syntax_extensions::Searchable,
    traceable_node::{ContextData, NodeKind, RustTraceableNode},
    NodeLocation,
};

pub(crate) trait Visitable {
    fn visit(&self, visitor: &mut dyn Visitor);
}

impl Visitable for SyntaxElement {
    fn visit(&self, visitor: &mut dyn Visitor) {
        match self {
            NodeOrToken::Node(n) => n.visit(visitor),
            NodeOrToken::Token(t) => t.visit(visitor),
        }
    }
}

impl Visitable for SyntaxNode {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.node_enter(self);

        // Iterate over all subnodes and tokens contained in this node
        if let Some(mut current) = self.first_child_or_token() {
            loop {
                current.visit(visitor);
                match current {
                    NodeOrToken::Node(subnode) => {
                        let Some(next) = subnode.next_sibling_or_token() else {
                            break;
                        };
                        current = next;
                    }
                    NodeOrToken::Token(subtoken) => {
                        let Some(next) = subtoken.next_sibling_or_token() else {
                            break;
                        };
                        current = next;
                    }
                }
            }
        }

        visitor.node_exit(self);
    }
}

impl Visitable for SyntaxToken {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.token_visit(self);
    }
}

pub(crate) trait Visitor {
    fn node_enter(&mut self, node: &SyntaxNode);
    fn node_exit(&mut self, node: &SyntaxNode);
    fn token_visit(&mut self, token: &SyntaxToken);
    fn travel(&mut self, root: &SyntaxNode);
}

struct VisitorData {
    whitespace_data: WhitespaceData,
    node_stack: Vec<RustTraceableNode>,
}

impl VisitorData {
    fn get_root(&self) -> Option<&RustTraceableNode> {
        self.node_stack.first()
    }

    fn _get_root_mut(&mut self) -> Option<&mut RustTraceableNode> {
        self.node_stack.first_mut()
    }
}

struct WhitespaceData {
    current_line: usize,
    last_linebrk: usize,
}

impl WhitespaceData {
    fn calculate_element_location(&self, element: &SyntaxElement) -> (usize, usize) {
        let element_start = usize::from(element.text_range().start());
        let col = element_start - self.last_linebrk;
        (self.current_line, col)
    }
}

pub(crate) struct RustVisitor {
    filepath: PathBuf,
    vdata: VisitorData,
    module_visitors: Vec<RustVisitor>,
}

impl RustVisitor {
    pub(crate) fn new(filepath: PathBuf) -> Self {
        RustVisitor {
            filepath,
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

    fn get_enclosing_scope(&self) -> Option<&ContextData> {
        // Get reference to the implementation data of the latest Impl node.
        self.vdata
            .node_stack
            .iter()
            .rev()
            .filter(|n| NodeKind::Context == n.kind)
            .next()
            .map(|rtn| rtn.context_data.as_ref())
            .flatten()
    }

    fn get_filename(&self) -> String {
        self.filepath
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

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

    fn enter_source(&mut self, source_node: &SyntaxNode) {
        let mut root_node = RustTraceableNode::from_node(source_node, String::new()).unwrap();
        root_node.name = self.get_filename();
        self.vdata.node_stack.push(root_node);
    }

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
        // Get filename as prefix
        let filepath = self.vdata.get_root().unwrap().name.clone();
        let mut prefix = self.get_filename();
        let location = NodeLocation::from(filepath, Some(line), Some(col));

        // Check for enclosing context
        if let Some(context_data) = self.get_enclosing_scope() {
            prefix += ".";
            prefix += &context_data.namespace;
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

    fn exit_fn(&mut self, _: &SyntaxNode) {
        // Pop function node from stack and add it to its parent node
        let closed_fn = self.vdata.node_stack.pop().unwrap();

        if let Some(enclosing_node) = self.vdata.node_stack.last_mut() {
            enclosing_node.append_child(closed_fn);
        }
    }

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
        let filepath = self.vdata.get_root().unwrap().name.clone();
        let mut prefix = self.get_filename();
        let location = NodeLocation::from(filepath, Some(line), Some(col));

        // Check for enclosing context
        if let Some(context_data) = self.get_enclosing_scope() {
            prefix += ".";
            prefix += &context_data.namespace;
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

    fn exit_struct(&mut self, _: &SyntaxNode) {
        // Pop struct node from stack and add it to its parent node
        let closed_struct = self.vdata.node_stack.pop().unwrap();

        if let Some(enclosing_node) = self.vdata.node_stack.last_mut() {
            enclosing_node.append_child(closed_struct);
        }
    }

    fn enter_impl(&mut self, impl_node: &SyntaxNode) {
        let node = RustTraceableNode::from_node(impl_node, String::new()).unwrap();
        self.vdata.node_stack.push(node);
    }

    fn exit_impl(&mut self, _impl_node: &SyntaxNode) {
        let closed_impl = self.vdata.node_stack.pop().unwrap();
        if let Some(enclosing_node) = self.vdata.node_stack.last_mut() {
            enclosing_node.append_child(closed_impl);
        }
    }

    fn enter_module(&mut self, mod_node: &SyntaxNode) {
        let name_node = mod_node.get_child_kind(SyntaxKind::NAME).unwrap();
        let last_child = mod_node.children_with_tokens().last().unwrap();
        match last_child {
            NodeOrToken::Token(t) => {
                if t.kind() == SyntaxKind::SEMICOLON {
                    // Found module declaration. Resolve to corresponding file.
                    if let Some(modpath) =
                        resolve_include(&self.filepath, &name_node.text().to_string())
                    {
                        self.module_visitors.push(RustVisitor::new(modpath));
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

    fn node_exit(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::FN => self.exit_fn(node),
            SyntaxKind::STRUCT => self.exit_struct(node),
            SyntaxKind::IMPL => self.exit_impl(node),
            SyntaxKind::MODULE => self.exit_module(node),
            _ => (),
        }
    }

    fn token_visit(&mut self, token: &SyntaxToken) {
        match token.kind() {
            SyntaxKind::WHITESPACE => self.visit_whitespace(token),
            SyntaxKind::COMMENT => self.visit_comment(token),
            _ => (),
        }
    }

    fn travel(&mut self, root: &SyntaxNode) {
        root.visit(self);
    }
}
