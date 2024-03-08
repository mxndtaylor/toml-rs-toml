use std::str::FromStr;

use crate::table::Iter;
use crate::{Item, RawString, Table};

/// Type representing a parsed TOML document
#[derive(Debug, Clone)]
pub struct ImDocument<S> {
    pub(crate) root: Item,
    // Trailing comments and whitespaces
    pub(crate) trailing: RawString,
    pub(crate) raw: S,
}

impl ImDocument<&'static str> {
    /// Creates an empty document
    pub fn new() -> Self {
        Default::default()
    }
}

#[cfg(feature = "parse")]
impl<S: AsRef<str>> ImDocument<S> {
    /// Parse a TOML document
    pub fn parse(raw: S) -> Result<Self, crate::TomlError> {
        crate::parser::parse_document(raw)
    }
}

impl<S> ImDocument<S> {
    /// Returns a reference to the root item.
    pub fn as_item(&self) -> &Item {
        &self.root
    }

    /// Returns a reference to the root table.
    pub fn as_table(&self) -> &Table {
        self.root.as_table().expect("root should always be a table")
    }

    /// Returns an iterator over the root table.
    pub fn iter(&self) -> Iter<'_> {
        self.as_table().iter()
    }

    /// Whitespace after last element
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }
}

impl<S: AsRef<str>> ImDocument<S> {
    /// Access the raw, unparsed document
    pub fn raw(&self) -> &str {
        self.raw.as_ref()
    }
}

impl<S: Into<String>> ImDocument<S> {
    /// Allow editing of the [`Document`]
    pub fn into_mut(self) -> Document {
        let mut doc = self.into_spanned_document();
        doc.despan();
        doc
    }

    pub(crate) fn into_spanned_document(self) -> Document {
        Document {
            root: self.root,
            trailing: self.trailing,
            raw: Some(self.raw.into()),
        }
    }
}

impl Default for ImDocument<&'static str> {
    fn default() -> Self {
        Self {
            root: Item::Table(Table::with_pos(Some(0))),
            trailing: Default::default(),
            raw: "",
        }
    }
}

#[cfg(feature = "parse")]
impl FromStr for ImDocument<String> {
    type Err = crate::TomlError;

    /// Parses a document from a &str
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s.to_owned())
    }
}

impl<S> std::ops::Deref for ImDocument<S> {
    type Target = Table;

    fn deref(&self) -> &Self::Target {
        self.as_table()
    }
}

/// Type representing a TOML document
#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) root: Item,
    // Trailing comments and whitespaces
    pub(crate) trailing: RawString,
    pub(crate) raw: Option<String>,
}

impl Document {
    /// Creates an empty document
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns a reference to the root item.
    pub fn as_item(&self) -> &Item {
        &self.root
    }

    /// Returns a mutable reference to the root item.
    pub fn as_item_mut(&mut self) -> &mut Item {
        &mut self.root
    }

    /// Returns a reference to the root table.
    pub fn as_table(&self) -> &Table {
        self.root.as_table().expect("root should always be a table")
    }

    /// Returns a mutable reference to the root table.
    pub fn as_table_mut(&mut self) -> &mut Table {
        self.root
            .as_table_mut()
            .expect("root should always be a table")
    }

    /// Returns an iterator over the root table.
    pub fn iter(&self) -> Iter<'_> {
        self.as_table().iter()
    }

    /// Set whitespace after last element
    pub fn set_trailing(&mut self, trailing: impl Into<RawString>) {
        self.trailing = trailing.into();
    }

    /// Whitespace after last element
    pub fn trailing(&self) -> &RawString {
        &self.trailing
    }

    /// # Panics
    ///
    /// If run on on a `Document` not generated by the parser
    pub(crate) fn despan(&mut self) {
        self.root.despan(self.raw.as_deref().unwrap());
        self.trailing.despan(self.raw.as_deref().unwrap());
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            root: Item::Table(Table::with_pos(Some(0))),
            trailing: Default::default(),
            raw: Default::default(),
        }
    }
}

#[cfg(feature = "parse")]
impl FromStr for Document {
    type Err = crate::TomlError;

    /// Parses a document from a &str
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let im = ImDocument::from_str(s)?;
        Ok(im.into_mut())
    }
}

impl std::ops::Deref for Document {
    type Target = Table;

    fn deref(&self) -> &Self::Target {
        self.as_table()
    }
}

impl std::ops::DerefMut for Document {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_table_mut()
    }
}

impl From<Table> for Document {
    fn from(root: Table) -> Self {
        Self {
            root: Item::Table(root),
            ..Default::default()
        }
    }
}

#[test]
#[cfg(feature = "parse")]
#[cfg(feature = "display")]
fn default_roundtrip() {
    Document::default().to_string().parse::<Document>().unwrap();
}
