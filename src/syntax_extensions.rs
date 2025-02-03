//! Additional traits and their implementations for the SyntaxElements from the ra_ap_syntax crate.

// BSD 3-Clause License
//
// Copyright (c) 2025, NewTec GmbH
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions
//    and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of
//    conditions and the following disclaimer in the documentation and/or other materials provided
//    with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to
//    endorse or promote products derived from this software without specific prior written
//    permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICU5LAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::visitor::Visitor;
use ra_ap_syntax::{NodeOrToken, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// Visitable trait
///
/// Implementation of the Visitable trait is needed to be visitable by structs implementing the
/// Visitor trait. The visit method has to be defined to establish which Visitor callbacks are
/// called and how the Visitable struct is traversed.
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
/// This trait is intended to extend the ra_ap_crates SyntaxElement with some additional access
/// methods.
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
