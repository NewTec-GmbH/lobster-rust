//! FileReference and GithubReference to track file source and posiiton in files.
use json::{object::Object, JsonValue};

/// Struct to define the location of an item in a file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FileReference {
    /// Name of the file.
    pub(crate) filename: String,
    /// Line in the file.
    pub(crate) line: Option<usize>,
    /// Column in the line.
    pub(crate) column: Option<usize>,
}

impl FileReference {
    /// Create a new FileReference.
    ///
    /// ### Parameters
    /// * `filename` - Filename to reference.
    /// * `line` - Optional line to reference.
    /// * `column` - Optional column to reference.
    ///
    /// ### Returns
    /// New FileReference
    pub fn new(filename: String, line: Option<usize>, column: Option<usize>) -> Self {
        FileReference {
            filename,
            line,
            column,
        }
    }

    /// Create a new default FileReference.
    ///
    /// ### Returns
    /// FileReference with default values.
    pub(crate) fn new_default() -> Self {
        FileReference {
            filename: "main.rs".to_string(),
            line: None,
            column: None,
        }
    }

    /// Convert the FileReference to a String representation.
    ///
    /// ### Returns
    /// String representation of the file reference.
    pub(crate) fn to_string(&self) -> String {
        let mut result = self.filename.clone();
        if let Some(line) = self.line {
            result.push_str(&format!(":{}", line));
        }
        if let Some(col) = self.column {
            result.push_str(&format!(":{}", col));
        }
        result
    }
}

/// Implement JsonValue::from(node: &FileReference)
///
/// This is needed in the conversion from a RustTraceableNode to a JsonValue.
impl From<&FileReference> for JsonValue {
    /// Convert a FileReference to a JsonValue.
    ///
    /// Parse a JsonValue from a FileReference.
    /// This conversion returns json in the form of a location object in the lobster common interchange format.
    /// This conversion is needed when converting a RustTraceableNode to lobster, as the node will contain a FileReference.
    /// The relevant fields of the location are parsed to the corresponding json fields.
    ///
    /// ### Parameters
    /// * `reference` - FileReference to convert to JsonValue.
    ///
    /// ### Returns Json object holding the location data in lobser common interchange format.
    fn from(reference: &FileReference) -> Self {
        let mut location_json = JsonValue::Object(Object::new());
        let _ = location_json.insert("kind", "file");
        let _ = location_json.insert("file", reference.filename.clone());
        let _ = location_json.insert("line", reference.line);
        let _ = location_json.insert("column", reference.column);
        location_json
    }
}
