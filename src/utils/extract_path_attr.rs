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
