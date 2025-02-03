//! Utility function to extract the path from path attributes.

// BSD 3-Clause License
//
// Copyright (c) 2025, NewTec GmbH
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived from
//    this software without specific prior written permission.
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

use ra_ap_syntax::{SyntaxKind, SyntaxNode};
use std::path::{Path, PathBuf};

use crate::syntax_extensions::Searchable;

/// Extracts the path from an attribute, if it is a path attribute.
///
/// Checks if a PATH node is nested in the META node of the attribute node.
/// If this is the case, it extracts the literal as a path.
///
/// ### Parameters
/// * `attr_node` - The attribute node to check.
///
/// ### Returns
/// Some(PathBuf) if a path vould be extracted, otherwise None.
///
pub(crate) fn extract_path_attribute(attr_node: &SyntaxNode) -> Option<PathBuf> {
    let meta_node = attr_node.get_child_kind(SyntaxKind::META)?;
    let _ = meta_node.get_child_kind(SyntaxKind::PATH)?;
    let literal_node = meta_node.get_child_kind(SyntaxKind::LITERAL)?;
    let path_string = literal_node
        .get_tokens_kind(SyntaxKind::STRING)
        .first()?
        .clone()
        .text()
        .to_string();
    let path = PathBuf::new().join(Path::new(&path_string[1..path_string.len() - 1]));
    Some(path)
}
