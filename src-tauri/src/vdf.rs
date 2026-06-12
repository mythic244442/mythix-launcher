// vdf.rs
// Minimal parser for Valve's KeyValues / VDF format.
//
// Supports the subset used by toolmanifest.vdf and compatibilitytool.vdf:
//   "key" "value"
//   "key"
//   {
//       "subkey" "subval"
//   }
//
// Does not support:
//   - Conditional blocks (#if / #endif)
//   - Nested includes
//   - Binary VDF

use std::collections::HashMap;
use std::path::Path;
use std::fs;

use crate::error::LauncherError;

/// A VDF node: either a leaf string value or a map of child nodes.
#[derive(Debug, Clone)]
pub enum VdfNode {
    Value(String),
    Map(HashMap<String, VdfNode>),
}

impl VdfNode {
    /// Get a child node by key (case-insensitive).
    pub fn get(&self, key: &str) -> Option<&VdfNode> {
        match self {
            VdfNode::Map(m) => m.get(&key.to_lowercase()),
            VdfNode::Value(_) => None,
        }
    }

    /// Get a child's string value.
    pub fn str_val(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            VdfNode::Value(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Convenience: unwrap as map.
    pub fn as_map(&self) -> Option<&HashMap<String, VdfNode>> {
        match self {
            VdfNode::Map(m) => Some(m),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            VdfNode::Value(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────────

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a [u8]) -> Self {
        Self { src, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn advance(&mut self) {
        if self.pos < self.src.len() {
            self.pos += 1;
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip ASCII whitespace
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
                self.advance();
            }
            // Skip // line comments
            if self.src.get(self.pos..self.pos + 2) == Some(b"//") {
                while !matches!(self.peek(), Some(b'\n') | None) {
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn read_quoted_string(&mut self) -> Result<String, LauncherError> {
        // Consume opening quote
        if self.peek() != Some(b'"') {
            return Err(LauncherError::Io(format!(
                "VDF: expected '\"' at byte {}",
                self.pos
            )));
        }
        self.advance();

        let mut s = String::new();
        loop {
            match self.peek() {
                None => {
                    return Err(LauncherError::Io("VDF: unterminated string".into()));
                }
                Some(b'"') => {
                    self.advance();
                    break;
                }
                Some(b'\\') => {
                    self.advance();
                    match self.peek() {
                        Some(b'n')  => { s.push('\n'); self.advance(); }
                        Some(b't')  => { s.push('\t'); self.advance(); }
                        Some(b'"')  => { s.push('"');  self.advance(); }
                        Some(b'\\') => { s.push('\\'); self.advance(); }
                        Some(c) => {
                            s.push('\\');
                            s.push(c as char);
                            self.advance();
                        }
                        None => break,
                    }
                }
                Some(c) => {
                    s.push(c as char);
                    self.advance();
                }
            }
        }
        Ok(s)
    }

    fn parse_node(&mut self) -> Result<(String, VdfNode), LauncherError> {
        self.skip_whitespace_and_comments();

        // Read the key
        let key = self.read_quoted_string()?.to_lowercase();
        self.skip_whitespace_and_comments();

        match self.peek() {
            // Sub-map
            Some(b'{') => {
                self.advance(); // consume '{'
                let map = self.parse_map()?;
                Ok((key, VdfNode::Map(map)))
            }
            // Inline value
            Some(b'"') => {
                let val = self.read_quoted_string()?;
                Ok((key, VdfNode::Value(val)))
            }
            other => Err(LauncherError::Io(format!(
                "VDF: unexpected byte {:?} after key '{}' at pos {}",
                other, key, self.pos
            ))),
        }
    }

    fn parse_map(&mut self) -> Result<HashMap<String, VdfNode>, LauncherError> {
        let mut map = HashMap::new();
        loop {
            self.skip_whitespace_and_comments();
            match self.peek() {
                None | Some(b'}') => {
                    if self.peek() == Some(b'}') {
                        self.advance();
                    }
                    break;
                }
                Some(b'"') => {
                    let (k, v) = self.parse_node()?;
                    map.insert(k, v);
                }
                Some(c) => {
                    return Err(LauncherError::Io(format!(
                        "VDF: unexpected byte 0x{:02x} at pos {}",
                        c, self.pos
                    )));
                }
            }
        }
        Ok(map)
    }

    /// Parse the top-level document: one or more root-level key+block pairs.
    fn parse_document(&mut self) -> Result<HashMap<String, VdfNode>, LauncherError> {
        let mut root = HashMap::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek().is_none() {
                break;
            }
            if self.peek() != Some(b'"') {
                break;
            }
            let (k, v) = self.parse_node()?;
            root.insert(k, v);
        }
        Ok(root)
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse a VDF string into a node tree.
pub fn parse(src: &str) -> Result<HashMap<String, VdfNode>, LauncherError> {
    let mut p = Parser::new(src.as_bytes());
    p.parse_document()
}

/// Load and parse a VDF file.
pub fn load(path: &Path) -> Result<HashMap<String, VdfNode>, LauncherError> {
    let src = fs::read_to_string(path)
        .map_err(|e| LauncherError::Io(format!("VDF read '{}': {e}", path.display())))?;
    parse(&src)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toolmanifest() {
        let src = r#"
// Generated file, do not edit
"manifest"
{
    "commandline"  "/proton %verb%"
    "version"      "2"
    "require_tool_appid" "1628350"
    "compatmanager_layer_name" "proton"
}
"#;
        let doc = parse(src).unwrap();
        let mf = doc.get("manifest").unwrap();
        assert_eq!(mf.str_val("commandline"), Some("/proton %verb%"));
        assert_eq!(mf.str_val("require_tool_appid"), Some("1628350"));
        assert_eq!(mf.str_val("compatmanager_layer_name"), Some("proton"));
    }

    #[test]
    fn parse_slr_manifest() {
        let src = r#"
"manifest"
{
    "commandline" "/_v2-entry-point --verb=%verb% --"
    "compatmanager_layer_name" "container-runtime"
}
"#;
        let doc = parse(src).unwrap();
        let mf = doc.get("manifest").unwrap();
        assert_eq!(
            mf.str_val("commandline"),
            Some("/_v2-entry-point --verb=%verb% --")
        );
        assert_eq!(mf.str_val("require_tool_appid"), None);
    }

    #[test]
    fn parse_compat_tool_vdf() {
        let src = r#"
"compatibilitytools"
{
  "compat_tools"
  {
    "my-tool"
    {
      "display_name" "My Tool"
      "install_path" "."
    }
  }
}
"#;
        let doc = parse(src).unwrap();
        let tools = doc
            .get("compatibilitytools").unwrap()
            .get("compat_tools").unwrap()
            .get("my-tool").unwrap();
        assert_eq!(tools.str_val("display_name"), Some("My Tool"));
    }
}
