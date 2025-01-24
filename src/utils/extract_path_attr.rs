use ra_ap_syntax::{SyntaxKind, SyntaxNode};
use std::path::{Path, PathBuf};

use crate::syntax_extensions::Searchable;

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
