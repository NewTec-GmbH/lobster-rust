/// Traits to extend the ra_ap_syntax structs with additional functionality

use ra_ap_syntax::{Direction, NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken, SyntaxElement};

pub(crate) trait Searchable {
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