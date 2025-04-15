//! Implementation of the `JObject`.
use crate::{JObject, JValue};
use core::mem;
#[expect(dead_code, reason = "todo")]
impl JObject {
  /// Clears all entries and index mappings from the object.
  #[inline]
  pub fn clear(&mut self) {
    self.entries.clear();
    self.idx.clear();
  }
  /// Returns a reference to the value associated with the given key.
  /// Returns `None` if the key is not found.
  #[inline]
  #[must_use]
  pub fn get(&self, key: &String) -> Option<&JValue> {
    Some(&self.entries.get(*self.idx.get(key)?)?.1)
  }
  /// Returns a reference to the key-value pair at the specified index.
  /// Returns `None` if the index is out of bounds.
  #[inline]
  #[must_use]
  pub fn get_index(&self, index: usize) -> Option<&(String, JValue)> {
    self.entries.get(index)
  }
  /// Returns a mutable reference to the key-value pair at the specified index.
  /// Returns `None` if the index is out of bounds.
  #[inline]
  pub fn get_index_mut(&mut self, index: usize) -> Option<&mut (String, JValue)> {
    self.entries.get_mut(index)
  }
  /// Returns a mutable reference to the value associated with the given key.
  /// Returns `None` if the key is not found.
  #[inline]
  pub fn get_mut(&mut self, key: &String) -> Option<&mut JValue> {
    Some(&mut self.entries.get_mut(*self.idx.get(key)?)?.1)
  }
  /// Inserts a key-value pair into the object.
  /// If the key already exists, replaces the value and returns the old one.
  /// Otherwise, inserts a new entry and returns `None`.
  #[inline]
  pub fn insert(&mut self, key: String, value: JValue) -> Option<JValue> {
    if let Some(&idx) = self.idx.get(&key) {
      if let Some(entry) = self.entries.get_mut(idx) {
        let old_value = mem::replace(&mut entry.1, value);
        Some(old_value)
      } else {
        None
      }
    } else {
      let index = self.entries.len();
      self.entries.push((key.clone(), value));
      self.idx.insert(key, index);
      None
    }
  }
  /// Returns `true` if the object contains no entries.
  #[inline]
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }
  /// Returns an iterator over all key-value pairs in insertion order.
  #[inline]
  pub fn iter(&self) -> impl Iterator<Item = &(String, JValue)> {
    self.entries.iter()
  }
  /// Returns the number of entries in the object.
  #[inline]
  #[must_use]
  pub fn len(&self) -> usize {
    self.entries.len()
  }
  /// Removes the entry with the given key and returns its value, if it exists.
  /// Updates the index map to reflect the removal.
  #[inline]
  pub fn remove(&mut self, key: &String) -> Option<JValue> {
    if let Some(&remove_idx) = self.idx.get(key) {
      let (_removed_key, removed_value) = self.entries.remove(remove_idx);
      self.idx.remove(key);
      for i in remove_idx..self.entries.len() {
        let ke = &self.entries.get(i)?.0;
        self.idx.insert(ke.clone(), i);
      }
      Some(removed_value)
    } else {
      None
    }
  }
}
