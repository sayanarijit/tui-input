use std::ops::{Deref, DerefMut};

#[cfg(test)]
mod tests;

#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "String", into = "String"))]
pub(super) struct Value {
    s: String,
    chars: usize,
}

impl Value {
    pub(super) fn new<T>(s: T) -> Self
    where
        T: Into<String>,
    {
        let s = s.into();
        let chars = s.chars().count();
        Self { s, chars }
    }

    pub(super) fn as_str(&self) -> &str {
        &self.s
    }

    pub(super) fn chars(&self) -> usize {
        self.chars
    }

    pub(super) fn edit(&mut self) -> ValueMut<'_> {
        ValueMut {
            v: self,
            dirty: false,
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<Value> for String {
    fn from(v: Value) -> Self {
        v.s
    }
}

pub(super) struct ValueMut<'a> {
    v: &'a mut Value,
    dirty: bool,
}

impl Deref for ValueMut<'_> {
    type Target = String;

    fn deref(&self) -> &String {
        &self.v.s
    }
}

impl DerefMut for ValueMut<'_> {
    fn deref_mut(&mut self) -> &mut String {
        self.dirty = true;
        &mut self.v.s
    }
}

impl Drop for ValueMut<'_> {
    fn drop(&mut self) {
        if self.dirty {
            self.v.chars = self.v.s.chars().count();
        }
    }
}
