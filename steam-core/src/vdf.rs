//! Valve KeyValues ("VDF") text format, used by every Steam config file.
//! Insertion order is preserved: Steam rewrites these files itself.

use std::fmt;

/// A KeyValues value: either a quoted string or a nested object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdfValue {
    String(String),
    Object(VdfObject),
}

impl VdfValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            VdfValue::String(value) => Some(value),
            VdfValue::Object(_) => None,
        }
    }

    pub fn as_object(&self) -> Option<&VdfObject> {
        match self {
            VdfValue::Object(object) => Some(object),
            VdfValue::String(_) => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut VdfObject> {
        match self {
            VdfValue::Object(object) => Some(object),
            VdfValue::String(_) => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, VdfValue::Object(_))
    }
}

/// An ordered KeyValues object.
///
/// Lookups are case-insensitive because Steam is not consistent about key
/// casing between client versions and platforms.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VdfObject {
    entries: Vec<(String, VdfValue)>,
}

impl VdfObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &VdfValue)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_str(), value))
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|(key, _)| key.as_str())
    }

    fn position(&self, key: &str) -> Option<usize> {
        self.entries
            .iter()
            .position(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
    }

    pub fn get(&self, key: &str) -> Option<&VdfValue> {
        self.position(key).map(|index| &self.entries[index].1)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut VdfValue> {
        self.position(key)
            .map(move |index| &mut self.entries[index].1)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(VdfValue::as_str)
    }

    pub fn get_object(&self, key: &str) -> Option<&VdfObject> {
        self.get(key).and_then(VdfValue::as_object)
    }

    pub fn get_object_mut(&mut self, key: &str) -> Option<&mut VdfObject> {
        self.get_mut(key).and_then(VdfValue::as_object_mut)
    }

    /// Case-insensitive insert. Keeps the existing key spelling when present.
    pub fn insert(&mut self, key: &str, value: VdfValue) {
        match self.position(key) {
            Some(index) => self.entries[index].1 = value,
            None => self.entries.push((key.to_string(), value)),
        }
    }

    pub fn set_string(&mut self, key: &str, value: impl Into<String>) {
        self.insert(key, VdfValue::String(value.into()));
    }

    pub fn remove(&mut self, key: &str) -> Option<VdfValue> {
        let index = self.position(key)?;
        Some(self.entries.remove(index).1)
    }

    /// Returns the nested object for `key`, creating it when missing.
    pub fn ensure_object(&mut self, key: &str) -> &mut VdfObject {
        let index = match self.position(key) {
            Some(index) => index,
            None => {
                self.entries
                    .push((key.to_string(), VdfValue::Object(VdfObject::new())));
                self.entries.len() - 1
            }
        };

        if !self.entries[index].1.is_object() {
            self.entries[index].1 = VdfValue::Object(VdfObject::new());
        }

        self.entries[index]
            .1
            .as_object_mut()
            .expect("value was just replaced by an object")
    }

    /// Walks a nested path, creating missing objects when `create` is set.
    pub fn path_mut(&mut self, parts: &[&str], create: bool) -> Option<&mut VdfObject> {
        let mut current = self;
        for part in parts {
            if !current.get(part).is_some_and(VdfValue::is_object) {
                if !create {
                    return None;
                }
                current.ensure_object(part);
            }
            current = current.get_object_mut(part)?;
        }
        Some(current)
    }

    pub fn path(&self, parts: &[&str]) -> Option<&VdfObject> {
        let mut current = self;
        for part in parts {
            current = current.get_object(part)?;
        }
        Some(current)
    }
}

impl FromIterator<(String, VdfValue)> for VdfObject {
    fn from_iter<T: IntoIterator<Item = (String, VdfValue)>>(iter: T) -> Self {
        let mut object = VdfObject::new();
        for (key, value) in iter {
            object.insert(&key, value);
        }
        object
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdfError {
    pub message: String,
    pub offset: usize,
}

impl VdfError {
    fn at(offset: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }
}

impl fmt::Display for VdfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid KeyValues data at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for VdfError {}

#[derive(Debug, Clone)]
enum Token {
    Text(String),
    OpenBrace,
    CloseBrace,
}

fn tokenize(input: &str) -> Result<Vec<Token>, VdfError> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < bytes.len() {
        let current = bytes[index];

        if current.is_ascii_whitespace() {
            index += 1;
            continue;
        }

        // Steam's own files use "//" comments; keep supporting them so hand
        // edited configs do not break the parser.
        if current == b'/' && bytes.get(index + 1) == Some(&b'/') {
            match input[index..].find('\n') {
                Some(offset) => index += offset + 1,
                None => break,
            }
            continue;
        }

        if current == b'{' {
            tokens.push(Token::OpenBrace);
            index += 1;
            continue;
        }

        if current == b'}' {
            tokens.push(Token::CloseBrace);
            index += 1;
            continue;
        }

        if current == b'"' {
            index += 1;
            let mut value = String::new();
            let mut terminated = false;

            while index < bytes.len() {
                let byte = bytes[index];
                if byte == b'\\' && index + 1 < bytes.len() {
                    let escaped = input[index + 1..].chars().next().unwrap_or('\\');
                    value.push(escaped);
                    index += 1 + escaped.len_utf8();
                    continue;
                }
                if byte == b'"' {
                    index += 1;
                    terminated = true;
                    break;
                }
                let character = input[index..].chars().next().unwrap_or('\u{fffd}');
                value.push(character);
                index += character.len_utf8();
            }

            if !terminated {
                return Err(VdfError::at(index, "unterminated quoted string"));
            }

            tokens.push(Token::Text(value));
            continue;
        }

        let start = index;
        while index < bytes.len() {
            let byte = bytes[index];
            if byte.is_ascii_whitespace() || byte == b'{' || byte == b'}' || byte == b'"' {
                break;
            }
            index += 1;
        }
        tokens.push(Token::Text(input[start..index].to_string()));
    }

    Ok(tokens)
}

fn parse_object(tokens: &[Token], cursor: &mut usize) -> Result<VdfObject, VdfError> {
    let mut object = VdfObject::new();

    while *cursor < tokens.len() {
        let key = match &tokens[*cursor] {
            Token::Text(text) => text.clone(),
            Token::CloseBrace => {
                *cursor += 1;
                return Ok(object);
            }
            Token::OpenBrace => {
                return Err(VdfError::at(
                    *cursor,
                    "unexpected '{' where a key was expected",
                ))
            }
        };
        *cursor += 1;

        let token = tokens
            .get(*cursor)
            .ok_or_else(|| VdfError::at(*cursor, format!("missing value for key {key:?}")))?;

        match token {
            Token::OpenBrace => {
                *cursor += 1;
                let child = parse_object(tokens, cursor)?;
                object.insert(&key, VdfValue::Object(child));
            }
            Token::CloseBrace => {
                return Err(VdfError::at(
                    *cursor,
                    format!("unexpected '}}' after key {key:?}"),
                ))
            }
            Token::Text(text) => {
                object.insert(&key, VdfValue::String(text.clone()));
                *cursor += 1;
            }
        }
    }

    Ok(object)
}

/// Parses KeyValues text into an ordered object tree.
pub fn parse(input: &str) -> Result<VdfObject, VdfError> {
    let tokens = tokenize(input)?;
    let mut cursor = 0;
    let object = parse_object(&tokens, &mut cursor)?;
    if cursor != tokens.len() {
        return Err(VdfError::at(
            cursor,
            "unexpected trailing data after the top level object",
        ));
    }
    Ok(object)
}

fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn write_object(object: &VdfObject, depth: usize, output: &mut String) {
    let indent = "\t".repeat(depth);
    for (key, value) in &object.entries {
        match value {
            VdfValue::String(text) => {
                output.push_str(&format!(
                    "{indent}\"{}\"\t\t\"{}\"\n",
                    escape(key),
                    escape(text)
                ));
            }
            VdfValue::Object(child) => {
                output.push_str(&format!("{indent}\"{}\"\n", escape(key)));
                output.push_str(&format!("{indent}{{\n"));
                write_object(child, depth + 1, output);
                output.push_str(&format!("{indent}}}\n"));
            }
        }
    }
}

/// Serializes an object tree the same way Steam does, using tab indentation.
pub fn dump(object: &VdfObject) -> String {
    let mut output = String::new();
    write_object(object, 0, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
// steam login cache
"users"
{
	"76561198000000001"
	{
		"AccountName"		"first-account"
		"PersonaName"		"First \"Nickname\""
		"MostRecent"		"1"
		"Timestamp"		"1700000000"
	}
	"76561198000000002"
	{
		"AccountName"		"second-account"
		"PersonaName"		"Second"
		"MostRecent"		"0"
	}
}
"#;

    #[test]
    fn parses_nested_objects_and_escapes() {
        let root = parse(SAMPLE).expect("sample parses");
        let users = root.get_object("users").expect("users object");
        assert_eq!(users.len(), 2);

        let first = users
            .get_object("76561198000000001")
            .expect("first account");
        assert_eq!(first.get_str("AccountName"), Some("first-account"));
        assert_eq!(first.get_str("PersonaName"), Some("First \"Nickname\""));
        assert_eq!(first.get_str("mostrecent"), Some("1"));
    }

    #[test]
    fn round_trips_through_dump_and_parse() {
        let root = parse(SAMPLE).expect("sample parses");
        let reparsed = parse(&dump(&root)).expect("dumped file parses");
        assert_eq!(root, reparsed);
    }

    #[test]
    fn insert_keeps_existing_key_spelling_and_order() {
        let mut object = VdfObject::new();
        object.set_string("AutoLoginUser", "first");
        object.set_string("Other", "value");
        object.set_string("autologinuser", "second");

        let keys: Vec<&str> = object.keys().collect();
        assert_eq!(keys, vec!["AutoLoginUser", "Other"]);
        assert_eq!(object.get_str("AUTOLOGINUSER"), Some("second"));
    }

    #[test]
    fn path_mut_creates_missing_branches() {
        let mut root = VdfObject::new();
        {
            let steam = root
                .path_mut(&["Registry", "HKCU", "Software", "Valve", "Steam"], true)
                .expect("path is created");
            steam.set_string("AutoLoginUser", "player");
        }

        assert!(root.path_mut(&["Registry", "HKLM"], false).is_none());
        assert_eq!(
            root.path(&["Registry", "HKCU", "Software", "Valve", "Steam"])
                .and_then(|steam| steam.get_str("AutoLoginUser")),
            Some("player")
        );
    }

    #[test]
    fn reports_unterminated_strings() {
        let error = parse("\"users\"\n{\n\t\"broken\"\t\"value\n}\n").unwrap_err();
        assert!(error.to_string().contains("unterminated"));
    }

    #[test]
    fn removes_case_insensitively() {
        let mut object = VdfObject::new();
        object.set_string("MostRecent", "1");
        assert_eq!(
            object.remove("mostrecent"),
            Some(VdfValue::String("1".into()))
        );
        assert!(object.is_empty());
    }
}
