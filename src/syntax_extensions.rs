/// Traits to extend the ra_ap_syntax structs with additional functionality

use ra_ap_syntax::{NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxElement};

pub(crate) trait Searchable {
    // extends the type with practical filtering options
    fn get_child_kind(&self, kind: SyntaxKind) -> Option<SyntaxNode>;
    fn get_children_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode>;
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

    fn get_children_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode> {
        match self {
            NodeOrToken::Node(n) => {
                n.get_children_kind(kind)
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

    fn get_children_kind(&self, kind: SyntaxKind) -> Vec<SyntaxNode> {
        self.children().filter(|c| kind == c.kind()).collect()
    }

    fn get_tokens_kind(&self, kind: SyntaxKind) -> Vec<SyntaxToken> {
        self.children_with_tokens().map(|ton| match ton {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(t) => Some(t),
        }).flatten().filter(|t| kind == t.kind()).collect()
    }
}