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

//! FileReference and GithubReference to track file source and posiiton in files.

use json::{object::Object, JsonValue};
use std::fmt::Display;

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

    /// Set line and column of FileReference.
    ///
    /// This is useful for adapting a FileReference that already exists.
    /// For example a function will receive a FileReference when it is parsed, but the correct line
    /// and column can only be set once the contained fn keyword is parsed.
    ///
    /// ### Parameters
    /// * `line`- Line to set. Option.
    /// * `column` - Column to set. Option
    pub(crate) fn set_position(&mut self, line: Option<usize>, column: Option<usize>) {
        self.line = line;
        self.column = column;
    }
}

/// Implement JsonValue::from(node: &FileReference)
///
/// This is needed in the conversion from a RustTraceableNode to a JsonValue.
impl From<&FileReference> for JsonValue {
    /// Convert a FileReference to a JsonValue.
    ///
    /// Parse a JsonValue from a FileReference.
    /// This conversion returns json in the form of a location object in the lobster common
    /// interchange format. This conversion is needed when converting a RustTraceableNode to
    /// lobster, as the node will contain a FileReference. The relevant fields of the location
    /// are parsed to the corresponding json fields.
    ///
    /// ### Parameters
    /// * `reference` - FileReference to convert to JsonValue.
    ///
    /// ### Returns Json object holding the location data in lobser common interchange format.
    fn from(reference: &FileReference) -> Self {
        // lobster-trace: LobsterRust.lobster_common_interchange_format
        let mut location_json = JsonValue::Object(Object::new());
        let _ = location_json.insert("kind", "file");
        let _ = location_json.insert("file", reference.filename.clone());
        let _ = location_json.insert("line", reference.line);
        let _ = location_json.insert("column", reference.column);
        location_json
    }
}

impl Display for FileReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}:{:?}", self.filename, self.line, self.column)?;
        Ok(())
    }
}
