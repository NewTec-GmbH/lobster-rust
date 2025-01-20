/// FileReference and GithubReference to track file source and posiiton in files.

use json::{JsonValue, object::Object};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LocationEnum<F, G> {
    File(F),
    Github(G)
}

pub type NodeLocation = LocationEnum<FileReference, GithubReference>;

impl NodeLocation {
    pub fn to_string(&self) -> String {
        match self {
            LocationEnum::File(f) => f.to_string(),
            LocationEnum::Github(g) => g.to_string()
        }
    }

    pub fn set_line(&mut self, line: usize) -> bool {
        match self {
            LocationEnum::File(f) => f.set_line(line),
            LocationEnum::Github(g) => g.set_line(line)
        }
    }

    pub fn set_col(&mut self, col: usize) -> bool {
        match self {
            LocationEnum::File(f) => f.set_col(col),
            LocationEnum::Github(g) => g.set_col(col)
        }
    }

    pub fn from(filename: String, line: Option<usize>, column: Option<usize>) -> Self {
        let fr = FileReference {
            filename,
            line,
            column
        };
        NodeLocation::File(fr)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FileReference {
    pub(crate) filename: String,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct GithubReference {
    
}

impl  FileReference {

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

impl From<&NodeLocation> for JsonValue {
    fn from(value: &NodeLocation) -> Self {
        match value {
            NodeLocation::File(f) => {
                let mut location_json = JsonValue::Object(Object::new());
                let _ = location_json.insert("kind", "file");
                let _ = location_json.insert("file", f.filename.clone());
                let _ = location_json.insert("line", f.line);
                let _ = location_json.insert("column", f.column);
                location_json
            },
            NodeLocation::Github(_) => {
                JsonValue::Object(Object::new())
            }
        }
    }
}
