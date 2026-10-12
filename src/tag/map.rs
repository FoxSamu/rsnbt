use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::*;

use super::*;

#[derive(Debug, Clone, Copy)]
struct KeyRef {
    value: *const String,
}

impl KeyRef {
    fn tmp(key: &String) -> KeyRef {
        KeyRef {
            value: key as *const String,
        }
    }
}

impl Hash for KeyRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { (*self.value).hash(state) };
    }
}

impl PartialEq for KeyRef {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.value == *other.value }
    }
}

impl Eq for KeyRef {}

#[derive(Debug)]
struct Node {
    key:   Box<String>, // Must be boxed so rust doesn't move it and break our pointer in KeyRef
    value: Tag,
    prev:  *mut Node,
    next:  *mut Node,
}

impl Node {
    fn new(key: String, value: Tag) -> Self {
        Self {
            key: Box::new(key),
            value,
            prev: null_mut(),
            next: null_mut(),
        }
    }

    fn key_ref(&self) -> KeyRef {
        KeyRef {
            value: Box::as_ptr(&self.key),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            key:   self.key.clone(),
            value: self.value.clone(),
            prev:  null_mut(),
            next:  null_mut(),
        }
    }
}

enum InsertMode {
    Start,
    End,
    KeepPositionOtherwiseStart,
    KeepPositionOtherwiseEnd,
}

/// A mapping from [`String`]s to [`Tag`]s. [`TagMap`]s exhibit much of the functionality of a [`HashMap`],
/// namely that they can store values under keys, and that each key only exists once. [`TagMap`]s are used
/// to represent [compound tags][Tag::Compound].
///
/// # Order preservation
/// Where [`TagMap`]s share a lot of similarities with `HashMap<String, Tag>` values, a key difference is
/// that a [`TagMap`] preserves an iteration order. By default, this order is the order in which elements
/// were inserted:
/// ```
/// use crate::tag::*;
///
/// let mut map = TagMap::new();
/// map.insert(String::from("foo"), 3);
/// map.insert(String::from("bar"), 5);
/// map.insert(String::from("baz"), 7);
/// map.insert(String::from("gus"), 11);
///
/// // Iterator now returns elements in the inserted order
/// let mut iter = map.into_iter();
/// assert_eq!(Some((String::from("foo"), 3)), iter.next());
/// assert_eq!(Some((String::from("bar"), 5)), iter.next());
/// assert_eq!(Some((String::from("baz"), 7)), iter.next());
/// assert_eq!(Some((String::from("gus"), 11)), iter.next());
/// ```
///
/// Different insertion methods have different but deterministic effects on iteration order.
///
/// Note that [`TagMap`]s are equal under [`PartialEq`] regardless of the iteration order. This means that
/// creating one map with `foo: 3, bar: 4` and another map with `bar: 4, foo: 3` will result in two maps
/// that are logically equal.
///
/// # Instantiation
/// It is possible to create [`TagMap`]s using a special macro, [`tag_map`]. This macro accepts a JSON-like syntax:
/// ```no_run
/// use crate::tag::*;
///
/// let non_const_key = "gus";
/// let non_const_value = true;
///
/// let map = tag_map! {
///     "foo": 3,
///     "bar": "hello world",
///
///     // Values can be any expression
///     "baz": non_const_value,
///
///     // To use expressions as keys, wrap them in
///     // square brackets
///     [non_const_key]: false
/// };
/// ```
///
/// The iteration order of maps created by this macro is the order in which keys were defined in the macro body.
pub struct TagMap {
    map:  HashMap<KeyRef, Box<Node>>,
    head: *mut Node,
    tail: *mut Node,
}

impl TagMap {
    pub fn new() -> Self {
        Self {
            map:  HashMap::new(),
            head: null_mut(),
            tail: null_mut(),
        }
    }

    fn add_node(&mut self, node: Node, end: bool) {
        // Get a key reference
        let key_ref = node.key_ref();

        // This method only adds new nodes
        if self.map.contains_key(&key_ref) {
            panic!("Cannot overwrite node");
        }

        // Box node to fix its pointer
        let mut boxed_node = Box::new(node);

        // Link node
        Self::link_node(&mut self.head, &mut self.tail, boxed_node.as_mut(), end);

        // Insert node into map
        self.map.insert(key_ref, boxed_node);
    }

    fn put_node(&mut self, node: Node, insert_mode: InsertMode) -> Option<Node> {
        // Get a key reference
        let key_ref = node.key_ref();

        let (keep_position, end) = match insert_mode {
            InsertMode::Start => (false, false),
            InsertMode::End => (false, true),
            InsertMode::KeepPositionOtherwiseStart => (true, false),
            InsertMode::KeepPositionOtherwiseEnd => (true, true),
        };

        // Add node if not new
        let existing = if let Some(v) = self.map.get(&key_ref) {
            v
        } else {
            self.add_node(node, end);
            return None;
        };

        // Box node to fix its pointer
        let mut boxed_node = Box::new(node);

        // Relink node
        if keep_position {
            // Relink in place
            Self::relink_node(&mut self.head, &mut self.tail, existing, boxed_node.as_mut());
        } else {
            // Unlink existing
            Self::unlink_node(&mut self.head, &mut self.tail, existing);

            // Link new at desired end
            Self::link_node(&mut self.head, &mut self.tail, boxed_node.as_mut(), end);
        }

        // Update node in map
        Some(*self.map.insert(key_ref, boxed_node).unwrap())
    }

    fn relink_node(head: &mut *mut Node, tail: &mut *mut Node, existing: &Node, new: &mut Node) {
        let prev = existing.prev;
        let next = existing.next;

        let node_ptr = new as *mut Node;

        // Update previous link
        new.prev = prev;
        if prev.is_null() {
            *head = node_ptr;
        } else {
            unsafe {
                (*prev).next = node_ptr;
            }
        }

        // Update next link
        new.next = next;
        if next.is_null() {
            *tail = node_ptr;
        } else {
            unsafe {
                (*next).prev = node_ptr;
            }
        }
    }

    fn link_node(head: &mut *mut Node, tail: &mut *mut Node, node: &mut Node, end: bool) {
        let node_ptr = node as *mut Node;
        if end {
            unsafe {
                if head.is_null() {
                    // First node
                    *head = node_ptr;
                    node.prev = null_mut();

                    *tail = node_ptr;
                    node.next = null_mut();
                } else {
                    // Not first node
                    (**tail).next = node_ptr;
                    node.prev = *tail;

                    node.next = null_mut();
                    *tail = node;
                };
            };
        } else {
            unsafe {
                if tail.is_null() {
                    // First node
                    *tail = node_ptr;
                    node.next = null_mut();

                    *head = node_ptr;
                    node.prev = null_mut();
                } else {
                    // Not first node
                    (**head).prev = node_ptr;
                    node.next = *head;

                    node.prev = null_mut();
                    *head = node;
                };
            };
        }
    }

    fn unlink_node(head: &mut *mut Node, tail: &mut *mut Node, node: &Node) {
        unsafe {
            if node.prev.is_null() && node.next.is_null() {
                *head = null_mut();
                *tail = null_mut();
            } else if node.prev.is_null() {
                let next = node.next;

                (*next).prev = null_mut();
                *head = next;
            } else if node.next.is_null() {
                let prev = node.prev;

                (*prev).next = null_mut();
                *tail = prev;
            } else {
                let prev = node.prev;
                let next = node.next;

                (*prev).next = next;
                (*next).prev = prev;
            };
        }
    }

    fn remove_node(&mut self, key: &KeyRef) -> Option<Node> {
        let boxed_node = self.map.remove(key)?;

        let mut node = *boxed_node;

        Self::unlink_node(&mut self.head, &mut self.tail, &mut node);

        Some(node)
    }

    fn get_node(&self, key: &KeyRef) -> Option<&Node> {
        self.map.get(key).map(|v| v.as_ref())
    }

    /// Insert a [`Tag`] under the given key into this [`TagMap`]. If there was a tag previously
    /// stored with the same key, then that tag is moved out and returned.
    ///
    /// # Order
    /// This method appends new keys at the end of the iteration order, but keeps the order the same
    /// if a key already existed in this map. That is, inserting `foo` and then `bar` into an empty map
    /// creates an iteration order of `foo` and then `bar`, but then inserting a new value for `foo`
    /// will keep the order `foo` then `bar`.
    pub fn insert<T>(&mut self, key: String, value: T) -> Option<Tag>
    where
        T: Into<Tag>,
    {
        let node = Node::new(key, value.into());
        let existing = self.put_node(node, InsertMode::KeepPositionOtherwiseEnd)?;
        Some(existing.value)
    }

    /// Insert a [`Tag`] under the given key into this [`TagMap`]. If there was a tag previously
    /// stored with the same key, then that tag is moved out and returned.
    ///
    /// # Order
    /// This method appends all keys at the end of the iteration order, moving existing keys to the
    /// end. That is, reinserting `foo` and then `bar` into an empty map creates an iteration order
    /// of `foo` and then `bar`, and inserting a new value for `foo` will change the order into `bar`
    /// then `foo`.
    pub fn reinsert<T>(&mut self, key: String, value: T) -> Option<Tag>
    where
        T: Into<Tag>,
    {
        let node = Node::new(key, value.into());
        let existing = self.put_node(node, InsertMode::End)?;
        Some(existing.value)
    }

    /// Insert a [`Tag`] under the given key into this [`TagMap`]. If there was a tag previously
    /// stored with the same key, then that tag is moved out and returned.
    ///
    /// # Order
    /// This method prepends new keys at the start of the iteration order, but keeps the order the same
    /// if a key already existed in this map. That is, inserting `foo` and then `bar` into an empty map
    /// creates an iteration order of `bar` and then `foo`, but then inserting a new value for `foo`
    /// will keep the order `bar` then `foo`.
    pub fn insert_head<T>(&mut self, key: String, value: T) -> Option<Tag>
    where
        T: Into<Tag>,
    {
        let node = Node::new(key, value.into());
        let existing = self.put_node(node, InsertMode::KeepPositionOtherwiseStart)?;
        Some(existing.value)
    }

    /// Insert a [`Tag`] under the given key into this [`TagMap`]. If there was a tag previously
    /// stored with the same key, then that tag is moved out and returned.
    ///
    /// # Order
    /// This method prepends all keys at the start of the iteration order, moving existing keys to the
    /// start. That is, reinserting `foo` and then `bar` into an empty map creates an iteration order
    /// of `bar` and then `foo`, and inserting a new value for `foo` will change the order into `foo`
    /// then `bar`.
    pub fn reinsert_head<T>(&mut self, key: String, value: T) -> Option<Tag>
    where
        T: Into<Tag>,
    {
        let node = Node::new(key, value.into());
        let existing = self.put_node(node, InsertMode::Start)?;
        Some(existing.value)
    }

    /// Removes a value from this [`TagMap`] given its key, and returns it. If no value existed with given key,
    /// then this method returns [`None`].
    ///
    /// # Order
    /// This method preserves the iteration order of the remaining elements. That is, in a map with iteration order
    /// `foo`, `bar`, `baz`, removing `bar` will result in an iteration order `foo`, `baz`.
    ///
    /// Note that the position of a key in the iteration order is forgotten once that key is removed. This means that
    /// reinserting that key later will always result in that key appearing at the start or end of the iteration order
    /// (depending on the method used for insertion), even if it wasn't there before.
    pub fn remove(&mut self, key: &String) -> Option<Tag> {
        let node = self.remove_node(&KeyRef::tmp(key))?;
        Some(node.value)
    }

    /// Returns a reference to a value in this [`TagMap`] given its key. If no value existed with the given key,
    /// then this method returns [`None`].
    pub fn get(&self, key: &String) -> Option<&Tag> {
        let node = self.get_node(&KeyRef::tmp(key))?;
        Some(&node.value)
    }

    /// Returns a reference to a value in this [`TagMap`] given its key and wraps it as a [`TagRef`]. If no value
    /// existed with the given key, then this method returns [`None`].
    pub fn get_ref(&self, key: &String) -> Option<TagRef<'_>> {
        Some(self.get(key)?.into())
    }

    /// Removes all elements from this [`TagMap`], discarding all of it.
    /// The iteration order of the map is completely discarded upon clearing.
    pub fn clear(&mut self) {
        self.map.clear();
        self.head = null_mut();
        self.tail = null_mut();
    }

    /// Returns whether this [`TagMap`] is emtpy, i.e., whether it has no elements.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the amount of elements in this [`TagMap`]. This is the amount of distinct
    /// keys in the tag map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns an iterator that iterates over references to all keys in this [`TagMap`].
    /// The iterator returns keys in the current iteration order of the map.
    pub fn keys(&self) -> Keys<'_> {
        Keys {
            next:     self.head,
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all values (tags) in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn values(&self) -> Values<'_> {
        Values {
            next:     self.head,
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all keys and values in this [`TagMap`], in pairs.
    /// The iterator returns elements in the current iteration order of the map.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            next:     self.head,
            lifetime: PhantomData,
        }
    }

    /// Moves this map into an iterator that iterates over all keys in this [`TagMap`].
    /// The iterator returns keys in the current iteration order of the map.
    pub fn into_keys(self) -> IntoKeys {
        IntoKeys {
            next:   self.head,
            _owner: self,
        }
    }

    /// Moves this map into an iterator that iterates over all values in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn into_values(self) -> IntoValues {
        IntoValues {
            next:   self.head,
            _owner: self,
        }
    }
}

impl Debug for TagMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (k, v) in self.iter() {
            map.entry(k, v);
        }
        map.finish()
    }
}

impl IntoIterator for TagMap {
    type Item = (String, Tag);

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            next:   self.head,
            _owner: self,
        }
    }
}

unsafe impl Sync for TagMap {}

impl Clone for TagMap {
    fn clone(&self) -> Self {
        let mut map = Self::new();

        for node in self.map.values() {
            map.add_node(node.deref().clone(), true);
        }

        map
    }
}

impl PartialEq for TagMap {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

pub struct Values<'a> {
    next:     *const Node,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            None
        } else {
            unsafe {
                let next = &(*self.next).value;
                self.next = (*self.next).next;

                Some(next)
            }
        }
    }
}

pub struct Keys<'a> {
    next:     *const Node,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            None
        } else {
            unsafe {
                let next = (*self.next).key.deref();
                self.next = (*self.next).next;

                Some(next)
            }
        }
    }
}

pub struct Iter<'a> {
    next:     *const Node,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Tag);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            None
        } else {
            unsafe {
                let nextk = (*self.next).key.deref();
                let nextv = &(*self.next).value;
                self.next = (*self.next).next;

                Some((nextk, nextv))
            }
        }
    }
}

const REPLACE_TAG: Tag = Tag::Byte(0);
const REPLACE_KEY: String = String::new();

pub struct IntoValues {
    next:   *mut Node,
    _owner: TagMap, // must keep ownership over pointer owner so the pointers don't get dropped yet
}

impl Iterator for IntoValues {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            None
        } else {
            unsafe {
                // Get a pointer to tag
                let next = &mut (*self.next).value as *mut Tag;

                // Advance iterator
                self.next = (*self.next).next;

                // Swap the pointed value with the replace tag
                // so we don't drop the moved value when the
                // iterator drops.
                Some(replace(next, REPLACE_TAG))
            }
        }
    }
}

pub struct IntoKeys {
    next:   *mut Node,
    _owner: TagMap, // must keep ownership over pointer owner so the pointers don't get dropped yet
}

impl Iterator for IntoKeys {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            None
        } else {
            unsafe {
                // Get a pointer to key
                let next = Box::as_mut_ptr(&mut (*self.next).key);

                // Advance iterator
                self.next = (*self.next).next;

                // Swap the pointed value with the replace key
                // so we don't drop the moved value when the
                // iterator drops.
                Some(replace(next, REPLACE_KEY))
            }
        }
    }
}

pub struct IntoIter {
    next:   *mut Node,
    _owner: TagMap, // must keep ownership over pointer owner so the pointers don't get dropped yet
}

impl Iterator for IntoIter {
    type Item = (String, Tag);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_null() {
            None
        } else {
            unsafe {
                // Get a pointer to tag
                let nextk = Box::as_mut_ptr(&mut (*self.next).key);

                // Get a pointer to tag
                let nextv = &mut (*self.next).value as *mut Tag;

                // Advance iterator
                self.next = (*self.next).next;

                // Swap the pointed value with the replace key
                // so we don't drop the moved value when the
                // iterator drops.
                Some((replace(nextk, REPLACE_KEY), replace(nextv, REPLACE_TAG)))
            }
        }
    }
}

#[macro_export]
macro_rules! tag_map {
    {} => { TagMap::new() };

    {
        $($($key:literal)? $([$key_expr:expr])? : $value:expr),+ $(,)?
    } => {
        {
            use $crate::tag::TagMap;

            let mut map = TagMap::new();
            $(
                map.insert(String::from($($key)? $($key_expr)?), $value);
            )+
            map
        }
    };
}

pub use tag_map;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_macro() {
        let map = tag_map! {
            "foo": 3,
            "bar": 4,
            "baz": 5,
            "gus": 6,
        };

        let mut keys = map.into_iter();
        assert_eq!(Some((String::from("foo"), Tag::from(3))), keys.next());
        assert_eq!(Some((String::from("bar"), Tag::from(4))), keys.next());
        assert_eq!(Some((String::from("baz"), Tag::from(5))), keys.next());
        assert_eq!(Some((String::from("gus"), Tag::from(6))), keys.next());
        assert_eq!(None, keys.next());
    }

    #[test]
    fn test_basic_inserts_order() {
        let mut map = TagMap::new();
        map.insert(String::from("foo"), Tag::Byte(3));
        map.insert(String::from("bar"), Tag::Byte(15));
        map.insert(String::from("baz"), Tag::Int(34123));
        map.insert(String::from("gus"), Tag::Int(99128));

        let mut keys = map.into_keys();
        assert_eq!(Some(String::from("foo")), keys.next());
        assert_eq!(Some(String::from("bar")), keys.next());
        assert_eq!(Some(String::from("baz")), keys.next());
        assert_eq!(Some(String::from("gus")), keys.next());
        assert_eq!(None, keys.next());
    }

    #[test]
    fn test_insert_overwrite_order() {
        let mut map = TagMap::new();
        map.insert(String::from("foo"), Tag::Byte(3));
        map.insert(String::from("bar"), Tag::Byte(15));
        map.insert(String::from("baz"), Tag::Int(34123));
        map.insert(String::from("bar"), Tag::Int(99128));

        let mut keys = map.into_keys();
        assert_eq!(Some(String::from("foo")), keys.next());
        assert_eq!(Some(String::from("bar")), keys.next());
        assert_eq!(Some(String::from("baz")), keys.next());
        assert_eq!(None, keys.next());
    }

    #[test]
    fn test_remove_then_insert_order() {
        let mut map = TagMap::new();
        map.insert(String::from("foo"), Tag::Byte(3));
        map.insert(String::from("bar"), Tag::Byte(15));
        map.insert(String::from("baz"), Tag::Int(34123));

        map.remove(&String::from("bar"));
        map.insert(String::from("bar"), Tag::Int(99128));

        let mut keys = map.into_keys();
        assert_eq!(Some(String::from("foo")), keys.next());
        assert_eq!(Some(String::from("baz")), keys.next());
        assert_eq!(Some(String::from("bar")), keys.next());
        assert_eq!(None, keys.next());
    }
}
