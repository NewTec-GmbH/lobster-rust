/// RustVisitor to traverse the syntax tree and gather data

use ra_ap_syntax::{SyntaxElement, SyntaxNode, SyntaxToken, NodeOrToken, SyntaxKind};
use regex::Regex;

use crate::{syntax_extensions::Searchable, traceable_node::{ImplementationData, RustTraceableNode, NodeKind}, NodeLocation, utils::trim_filename};

pub(crate) trait Visitable {
    fn visit(&self, visitor: &mut dyn Visitor);
}

impl Visitable for SyntaxElement {
    fn visit(&self, visitor: &mut dyn Visitor) {
        match self {
            NodeOrToken::Node(n)  => n.visit(visitor),
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
                        let Some(next) = subnode.next_sibling_or_token() else { break; };
                        current = next;
                    },
                    NodeOrToken::Token(subtoken) => {
                        let Some(next) = subtoken.next_sibling_or_token() else { break; };
                        current = next;
                    },
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

pub(crate) struct VisitorData {
    whitespace_data: WhitespaceData,
    node_stack: Vec<RustTraceableNode>
}

impl VisitorData {
    pub(crate) fn get_root(&self) -> Option<&RustTraceableNode> {
        self.node_stack.first()
    }

    pub(crate) fn get_root_mut(&mut self) -> Option<&mut RustTraceableNode> {
        self.node_stack.first_mut()
    }
}

struct WhitespaceData {
    current_line: usize,
    last_linebrk: usize
}

impl WhitespaceData {
    fn calculate_element_location(&self, element: &SyntaxElement) -> (usize, usize) {
        let element_start = usize::from(element.text_range().start());
        let col = element_start - self.last_linebrk;
        (self.current_line, col)
    }
}

pub(crate) struct RustVisitor {
    pub(crate) filename: String,
    pub(crate) vdata: VisitorData,
}

impl RustVisitor {
    pub(crate) fn new(filename: String) -> Self {
        RustVisitor {
            filename,
            vdata: VisitorData {
                whitespace_data: WhitespaceData { current_line: 1, last_linebrk: 0 },
                node_stack: Vec::new()
            }
         }
    }

    fn get_enclosing_scope(&self) -> Option<&ImplementationData> {
        // Get reference to the implementation data of the latest Impl node.
        self.vdata.node_stack.iter().rev()
        .filter(|n| NodeKind::Impl == n.kind).next()
        .map(|rtn| rtn.impl_data.as_ref()).flatten()
    }

    // Node visit functions

    fn enter_source(&mut self, source_node: &SyntaxNode) {
        let mut root_node = RustTraceableNode::from_node(source_node, String::new()).unwrap();
        root_node.name = self.filename.clone();
        self.vdata.node_stack.push(root_node);
    }

    fn enter_fn(&mut self, fn_node: &SyntaxNode) {
        // Calculate position in file.
        let (line, col): (usize, usize);
        if let Some(fn_kw_token) = fn_node.get_tokens_kind(SyntaxKind::FN_KW).pop() {
            (line, col) = self.vdata.whitespace_data.calculate_element_location(&NodeOrToken::Token(fn_kw_token));
        } else {
            (line, col) = (self.vdata.whitespace_data.current_line, 0);
        }
        // Get filename as prefix
        let filepath = self.vdata.get_root().unwrap().name.clone();
        let mut prefix = trim_filename(&filepath).unwrap_or(String::new());
        let location = NodeLocation::from(filepath, Some(line), Some(col));
        
        // Check for enclosing impl
        if let Some(impl_data) = self.get_enclosing_scope() {
            prefix += ".";
            prefix += &impl_data.target;
        }

        // Parse node.
        let node = RustTraceableNode::from_node_with_location(fn_node, location, prefix).unwrap_or_else(|| panic!("Could not parse fn at line {:#?}.", self.vdata.whitespace_data.current_line));
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
            (line, col) = self.vdata.whitespace_data.calculate_element_location(&NodeOrToken::Token(struct_kw_token));
        } else {
            (line, col) = (self.vdata.whitespace_data.current_line, 1);
        }
        let filepath = self.vdata.get_root().unwrap().name.clone();
        let prefix = trim_filename(&filepath).unwrap_or(String::new());
        let location = NodeLocation::from(filepath, Some(line), Some(col));

        // Parse node.
        let node = RustTraceableNode::from_node_with_location(struct_node, location, prefix).unwrap_or_else(|| panic!("Could not parse struct at line {:#?}.", self.vdata.whitespace_data.current_line));
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

    // Token visit functions

    fn visit_whitespace(&mut self, whitespace_token: &SyntaxToken) {
        // Update whitespace data to hold current line and charpos of last linebreak
        let ws_data = &mut self.vdata.whitespace_data;

        let linebreaks = whitespace_token.text().chars().filter(|c| '\n' == *c).count();
        ws_data.current_line += linebreaks;

        if let Some((lbpos, _)) = whitespace_token.text().char_indices().filter(|(_, c)| '\n' == *c).last() {
            ws_data.last_linebrk = usize::from(whitespace_token.text_range().start()) + lbpos;
        }
    }

    fn visit_comment(&mut self, comment_token: &SyntaxToken) {
        // Parse comment for lobster trace or justification annotations
        if let Some(cnode) = self.vdata.node_stack.last_mut() {
            let trace_re = Regex::new(r"///?.*lobster-trace: (?<ref>[[:alnum:]\._-]+).*").unwrap();
            let just_re = Regex::new(r"///?.*lobster-exclude: (?<just>[[:alnum:]\._-]+).*").unwrap();

            if let Some(cap) =  trace_re.captures(comment_token.text()) {
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
            SyntaxKind::FN => self.enter_fn(node),
            SyntaxKind::SOURCE_FILE => self.enter_source(node),
            SyntaxKind::STRUCT => self.enter_struct(node),
            SyntaxKind::IMPL => self.enter_impl(node),
            _ => (),
        }
    }

    fn node_exit(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::FN => self.exit_fn(node),
            SyntaxKind::STRUCT => self.exit_struct(node),
            SyntaxKind::IMPL => self.exit_impl(node),
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
