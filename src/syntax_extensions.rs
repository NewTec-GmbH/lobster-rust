//! Additional traits and their implementations for the SyntaxElements from the ra_ap_syntax crate.
use ra_ap_syntax::{NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use crate::visitor::Visitor;

/// Visitable trait
///
/// Implementation of the Visitable trait is needed to be visitable by structs implementing the Visitor trait.
/// The visit method has to be defined to establish which Visitor callbacks are called and how the Visitable struct is traversed.
pub(crate) trait Visitable {
    fn visit(&self, visitor: &mut dyn Visitor);
}

impl Visitable for SyntaxElement {
    /// Calls the visit function of the underlying node or token.
    ///
    /// ### Parameters
    /// * `visitor` - struct implementing the Visitor trait.
    fn visit(&self, visitor: &mut dyn Visitor) {
        match self {
            NodeOrToken::Node(n) => n.visit(visitor),
            NodeOrToken::Token(t) => t.visit(visitor),
        }
    }
}

impl Visitable for SyntaxNode {
    /// Visits the node.
    ///
    /// The node is visited by first calling the visitors node_enter method.
    /// Then the nodes children (nodes and tokens) are visited in order.
    /// Finally, the visitors node_exit is called.
    ///
    /// ### Parameters
    /// * `visitor` - struct implementing the Visitor trait.
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.node_enter(self);

        // Iterate over all subnodes and tokens contained in this node.
        for child in self.children_with_tokens() {
            child.visit(visitor);
        }

        visitor.node_exit(self);
    }
}

impl Visitable for SyntaxToken {
    /// Visits the token.
    ///
    /// The token is visited by calling the visitors visit_token method.
    ///
    /// ### Parameters
    /// * `visitor` - struct implementing the Visitor trait.
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.token_visit(self);
    }
}

/// Eextends the type with practical filtering options.
///
/// This trait is intended to extend the ra_ap_crates SyntaxElement with some additional access methods.
pub(crate) trait Searchable {
    /// Returns the first child of the given kind, if any such child is found.
    ///
    /// ### Parameters
    /// * `kind` - SyntaxKind of the child.
    fn get_child_kind(&self, kind: SyntaxKind) -> Option<SyntaxNode>;

    /// Returns a vector of the node children of the given kind.
    ///
    /// ### Parameters
    /// * `kind` - SyntaxKind of the child.
    fn get_children_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode>;

    /// Returns a vector of the token children of the given kind.
    ///
    /// ### Parameters
    /// * `kind` - SyntaxKind of the child.
    fn get_tokens_kind(&self, kind: SyntaxKind) -> Vec<SyntaxToken>;
}

impl Searchable for SyntaxElement {
    fn get_child_kind(&self, kind: SyntaxKind) -> Option<SyntaxNode> {
        match self {
            NodeOrToken::Node(n) => n.get_child_kind(kind),
            _ => None,
        }
    }

    fn get_children_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode> {
        match self {
            NodeOrToken::Node(n) => n.get_children_kind(kind),
            _ => Vec::new(),
        }
    }

    fn get_tokens_kind(&self, kind: SyntaxKind) -> Vec<SyntaxToken> {
        match self {
            NodeOrToken::Node(n) => n.get_tokens_kind(kind),
            _ => Vec::new(),
        }
    }
}

impl Searchable for SyntaxNode {
    fn get_child_kind(&self, kind: SyntaxKind) -> Option<SyntaxNode> {
        self.children().filter(|c| kind == c.kind()).next()
    }

    fn get_children_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode> {
        self.children().filter(|c| kind == c.kind()).collect()
    }

    fn get_tokens_kind(&self, kind: SyntaxKind) -> Vec<SyntaxToken> {
        self.children_with_tokens()
            .map(|ton| match ton {
                NodeOrToken::Node(_) => None,
                NodeOrToken::Token(t) => Some(t),
            })
            .flatten()
            .filter(|t| kind == t.kind())
            .collect()
    }
}
