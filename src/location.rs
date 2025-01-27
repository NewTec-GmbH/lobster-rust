//! FileReference and GithubReference to track file source and posiiton in files.
use json::{object::Object, JsonValue};

/// Wrapper enum to hold both FileReference and GithubReference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LocationEnum<F, G> {
    File(F),
    Github(G),
}

/// Shorthand for the LocationEnum around FileReference and GithubReference.
pub type NodeLocation = LocationEnum<FileReference, GithubReference>;

impl NodeLocation {
    /// Calls the to_string method of the wrapped struct.
    pub fn to_string(&self) -> String {
        match self {
            LocationEnum::File(f) => f.to_string(),
            LocationEnum::Github(g) => g.to_string(),
        }
    }

    /// Calls the set_line method of the wrapped struct.
    pub fn set_line(&mut self, line: usize) -> bool {
        match self {
            LocationEnum::File(f) => f.set_line(line),
            LocationEnum::Github(g) => g.set_line(line),
        }
    }

    /// Calls the set_col method of the wrapped struct.
    pub fn set_col(&mut self, col: usize) -> bool {
        match self {
            LocationEnum::File(f) => f.set_col(col),
            LocationEnum::Github(g) => g.set_col(col),
        }
    }

    /// Construct a new wrapped FileReference.
    ///
    /// Shorthand function to construct an already wrapped FileReference
    ///
    /// ### Parameters
    /// * `filename` - Filename for the FileReference.
    /// * `line` - Optional line number in the file.
    /// * `column` - Optional column.
    ///
    /// ### Returns
    /// NodeLocation wrapping a FileReference.
    pub fn from(filename: String, line: Option<usize>, column: Option<usize>) -> Self {
        let fr = FileReference {
            filename,
            line,
            column,
        };
        NodeLocation::File(fr)
    }
}

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
/// Struct to define the location of an item on GitHub.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct GithubReference {}

impl FileReference {
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
    fn to_string(&self) -> String {
        let mut result = self.filename.clone();
        if let Some(line) = self.line {
            result.push_str(&format!(":{}", line));
        }
        if let Some(col) = self.column {
            result.push_str(&format!(":{}", col));
        }
        result
    }

    fn set_line(&mut self, line: usize) -> bool {
        self.line = Some(line);
        true
    }

    fn set_col(&mut self, col: usize) -> bool {
        self.column = Some(col);
        true
    }
}

impl GithubReference {
    fn to_string(&self) -> String {
        "GithubRef".to_string()
    }

    fn set_line(&mut self, _: usize) -> bool {
        false
    }
    fn set_col(&mut self, _: usize) -> bool {
        false
    }
}

/// Implement JsonValue::from(node: &NodeLocation)
///
/// This is needed in the conversion from a RustTraceableNode to a JsonValue.
impl From<&NodeLocation> for JsonValue {
    /// Convert NodeLocation to a JsonValue.
    ///
    /// Parse a JsonValue from a NodeLocation.
    /// This conversion returns json in the form of a location object in the lobster common interchange format.
    /// This conversion is needed when converting a RustTraceableNode to lobster, as the node will contain a NodeLocation.
    /// The relevant fields of the location are parsed to the corresponding json fields.
    ///
    /// ### Parameters
    /// * `value` - NodeLocation to convert to JsonValue.
    ///
    /// ### Returns Json object holding the location data in lobser common interchange format.
    fn from(value: &NodeLocation) -> Self {
        match value {
            NodeLocation::File(f) => {
                let mut location_json = JsonValue::Object(Object::new());
                let _ = location_json.insert("kind", "file");
                let _ = location_json.insert("file", f.filename.clone());
                let _ = location_json.insert("line", f.line);
                let _ = location_json.insert("column", f.column);
                location_json
            }
            NodeLocation::Github(_) => JsonValue::Object(Object::new()),
        }
    }
}
