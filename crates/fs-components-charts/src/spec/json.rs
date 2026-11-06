//! A reader for one JSON object that knows which keys were read.
//!
//! Pattern: Iterator-like cursor over a map — every `take` marks a key as understood; `finish`
//! refuses the first key nobody read. This is what turns "silently ignored" into a named refusal.

use serde_json::{Map, Value};

use super::error::{Result, SpecError};

/// One object being read.
pub(super) struct Obj<'a> {
    map: &'a Map<String, Value>,
    pub path: String,
    used: Vec<&'a str>,
}

impl<'a> Obj<'a> {
    /// `value` must be an object.
    pub fn new(value: &'a Value, path: &str) -> Result<Self> {
        match value {
            Value::Object(map) => Ok(Self {
                map,
                path: path.to_string(),
                used: Vec::new(),
            }),
            other => Err(SpecError::invalid(
                path,
                format!("expected an object, found {}", kind_of(other)),
            )),
        }
    }

    /// The keys in order.
    pub fn keys(&self) -> impl Iterator<Item = &'a str> + use<'a> {
        self.map.keys().map(String::as_str)
    }

    /// The value of a key that was listed by `keys`.
    pub fn get(&self, key: &str) -> &'a Value {
        &self.map[key]
    }

    /// The JSON pointer of a child key.
    pub fn at(&self, key: &str) -> String {
        format!(
            "{}/{}",
            self.path,
            key.replace('~', "~0").replace('/', "~1")
        )
    }

    /// The value under `key`, marking it as read. `null` counts as present.
    pub fn take(&mut self, key: &'a str) -> Option<&'a Value> {
        self.used.push(key);
        self.map.get(key)
    }

    /// A string, if present.
    pub fn string(&mut self, key: &'a str) -> Result<Option<String>> {
        match self.take(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(v) => Err(self.wrong(key, "a string", v)),
        }
    }

    /// A finite number, if present.
    pub fn number(&mut self, key: &'a str) -> Result<Option<f64>> {
        match self.take(key) {
            None => Ok(None),
            Some(Value::Number(n)) => Ok(n.as_f64().filter(|x| x.is_finite())),
            Some(v) => Err(self.wrong(key, "a number", v)),
        }
    }

    /// A boolean, if present.
    pub fn boolean(&mut self, key: &'a str) -> Result<Option<bool>> {
        match self.take(key) {
            None => Ok(None),
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(v) => Err(self.wrong(key, "true or false", v)),
        }
    }

    /// The error for a value of the wrong kind.
    pub fn wrong(&self, key: &str, expected: &str, found: &Value) -> SpecError {
        SpecError::invalid(
            &self.at(key),
            format!("expected {expected}, found {}", kind_of(found)),
        )
    }

    /// Refuse the first key that was never read.
    pub fn finish(self) -> Result<()> {
        match self.map.keys().find(|k| !self.used.contains(&k.as_str())) {
            None => Ok(()),
            Some(key) => Err(SpecError::outside(
                &self.at(key),
                format!("property \"{key}\" is not part of the supported subset"),
            )),
        }
    }
}

/// A word for the kind of a JSON value, for messages.
pub(super) fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}
