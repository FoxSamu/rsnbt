use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::swap;
use std::ops::Deref;
use std::ptr::*;

use super::*;
use super::cast::*;


// ====================================================================================================
//
//   TAGMAP DATA STRUCTURE
//
// ====================================================================================================

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
    key: Box<String>, // Must be boxed so rust doesn't move it and break our pointer in KeyRef
    value: Tag,
    prev: *mut Node,
    next: *mut Node,
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
            key: self.key.clone(),
            value: self.value.clone(),
            prev: null_mut(),
            next: null_mut(),
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
/// were inserted. Different insertion methods have different but deterministic effects on iteration order.
///
/// Note that [`TagMap`]s are equal under [`PartialEq`] regardless of the iteration order. This means that
/// creating one map with `foo: 3, bar: 4` and another map with `bar: 4, foo: 3` will result in two maps
/// that are logically equal.
///
/// # Examples
/// ```
/// use rsnbt::tag::{Tag, TagMap, tag_map};
/// 
/// // the tag_map macro accepts a JSON-like syntax
/// let map = tag_map! { "foo": 1, "bar": "gus" };
/// 
/// assert_eq!(Some(&Tag::int(1)), map.get("foo"));
/// assert_eq!(Some(&Tag::string("gus")), map.get("bar"));
/// 
/// // this key is not part of the map
/// assert_eq!(None, map.get("baz"));
/// 
/// // iteration order is preserved
/// let mut iter = map.keys();
/// assert_eq!(Some(&"foo".into()), iter.next());
/// assert_eq!(Some(&"bar".into()), iter.next());
/// assert_eq!(None, iter.next());
/// ```
pub struct TagMap {
    map: HashMap<KeyRef, Box<Node>>,
    head: *mut Node,
    tail: *mut Node,
}

impl TagMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            head: null_mut(),
            tail: null_mut(),
        }
    }

    /// Replace a node in the linked list with a new one.
    fn link_overwrite(head: &mut *mut Node, tail: &mut *mut Node, existing: &Node, new: &mut Node) {
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

    /// Append a node to the linked list.
    fn link_push(head: &mut *mut Node, tail: &mut *mut Node, node: &mut Node, end: bool) {
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

    /// Removes a node from the linked list.
    fn link_remove(head: &mut *mut Node, tail: &mut *mut Node, node: &Node) {
        unsafe {
            if node.prev.is_null() && node.next.is_null() {
                // Node was only node
                
                *head = null_mut();
                *tail = null_mut();
            } else if node.prev.is_null() {
                // Node was first node

                let next = node.next;

                (*next).prev = null_mut();
                *head = next;
            } else if node.next.is_null() {
                // Node was last node

                let prev = node.prev;

                (*prev).next = null_mut();
                *tail = prev;
            } else {
                // Node was neither first nor last

                let prev = node.prev;
                let next = node.next;

                (*prev).next = next;
                (*next).prev = prev;
            };
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
        Self::link_push(&mut self.head, &mut self.tail, boxed_node.as_mut(), end);

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
            Self::link_overwrite(&mut self.head, &mut self.tail, existing, boxed_node.as_mut());
        } else {
            // Unlink existing
            Self::link_remove(&mut self.head, &mut self.tail, existing);

            // Link new at desired end
            Self::link_push(&mut self.head, &mut self.tail, boxed_node.as_mut(), end);
        }

        // Update node in map
        // We must explicitly remove the element first so that
        // the KeyRef is properly updated
        let elem = self.map.remove(&key_ref).unwrap();
        self.map.insert(key_ref, boxed_node);
        Some(*elem)
    }

    fn remove_node(&mut self, key: &KeyRef) -> Option<Node> {
        let boxed_node = self.map.remove(key)?;

        let mut node = *boxed_node;

        Self::link_remove(&mut self.head, &mut self.tail, &mut node);

        Some(node)
    }

    fn get_node(&self, key: &KeyRef) -> Option<&Node> {
        self.map.get(key).map(|v| v.as_ref())
    }

    fn get_node_mut(&mut self, key: &KeyRef) -> Option<&mut Node> {
        self.map.get_mut(key).map(|v| v.as_mut())
    }

    fn swap_nodes(&mut self, node1: &mut Node, node2: &mut Node) {
        // Swap eachother's links
        swap(&mut node1.prev, &mut node2.prev);
        swap(&mut node1.next, &mut node2.next);

        // Replace neighbour links
        unsafe {
            if node1.prev.is_null() {
                self.head = node1 as *mut Node;
            } else {
                (*node1.prev).next = node1 as *mut Node;
            }

            if node2.prev.is_null() {
                self.head = node2 as *mut Node;
            } else {
                (*node2.prev).next = node2 as *mut Node;
            }
            
            if node1.next.is_null() {
                self.tail = node1 as *mut Node;
            } else {
                (*node1.next).prev = node1 as *mut Node;
            }
            
            if node2.next.is_null() {
                self.tail = node2 as *mut Node;
            } else {
                (*node2.next).prev = node2 as *mut Node;
            }
        }
    }

    /// Insert a [`Tag`] under the given key into this [`TagMap`]. If there was a tag previously
    /// stored with the same key, then that tag is moved out and returned.
    ///
    /// # Order
    /// This method appends new keys at the end of the iteration order, but keeps the order the same
    /// if a key already existed in this map. That is, inserting `foo` and then `bar` into an empty map
    /// creates an iteration order of `foo` and then `bar`, but then inserting a new value for `foo`
    /// will keep the order `foo` then `bar`.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Tag>) -> Option<Tag> {
        let node = Node::new(key.into(), value.into());
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
    pub fn reinsert(&mut self, key: impl Into<String>, value: impl Into<Tag>) -> Option<Tag> {
        let node = Node::new(key.into(), value.into());
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
    pub fn insert_head(&mut self, key: impl Into<String>, value: impl Into<Tag>) -> Option<Tag> {
        let node = Node::new(key.into(), value.into());
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
    pub fn reinsert_head(&mut self, key: impl Into<String>, value: impl Into<Tag>) -> Option<Tag> {
        let node = Node::new(key.into(), value.into());
        let existing = self.put_node(node, InsertMode::Start)?;
        Some(existing.value)
    }

    /// Removes a value from this [`TagMap`] given its key, and returns it. If no value exists with given key,
    /// then this method does nothing and returns [`None`].
    ///
    /// # Order
    /// This method preserves the iteration order of the remaining elements. That is, in a map with iteration order
    /// `foo`, `bar`, `baz`, removing `bar` will result in an iteration order `foo`, `baz`.
    ///
    /// Note that the position of a key in the iteration order is forgotten once that key is removed. This means that
    /// reinserting that key later will always result in that key appearing at the start or end of the iteration order
    /// (depending on the method used for insertion), even if it wasn't there before.
    pub fn remove<T: ?Sized>(&mut self, key: &T) -> Option<Tag>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.remove_node(&KeyRef::tmp(&key.into()))?;
        Some(node.value)
    }

    /// Returns a reference to a value in this [`TagMap`] given its key. If no value exists with the given key,
    /// then this method returns [`None`].
    pub fn get<T: ?Sized>(&self, key: &T) -> Option<&Tag>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.get_node(&KeyRef::tmp(&key.into()))?;
        Some(&node.value)
    }

    /// Returns a mutable reference to a value in this [`TagMap`] given its key. If no value exists with the given key,
    /// then this method returns [`None`]. Note that mutating the tag value through the given reference will not cause
    /// the iteration order to change.
    pub fn get_mut<T: ?Sized>(&mut self, key: &T) -> Option<&mut Tag>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.get_node_mut(&KeyRef::tmp(&key.into()))?;
        Some(&mut node.value)
    }

    /// Returns a reference to a value in this [`TagMap`] given its key and wraps it as a [`TagRef`]. If no value
    /// exists with the given key, then this method returns [`None`].
    pub fn get_ref<T: ?Sized>(&self, key: &T) -> Option<TagRef<'_>>
    where
        for<'a> &'a T: Into<String>,
    {
        Some(self.get(key)?.into())
    }

    /// Returns a mutable reference to a value in this [`TagMap`] given its key and wraps it as a [`TagRefMut`]. If no value
    /// exists with the given key, then this method returns [`None`].
    pub fn get_ref_mut<T: ?Sized>(&mut self, key: &T) -> Option<TagRefMut<'_>>
    where
        for<'a> &'a T: Into<String>,
    {
        Some(self.get_mut(key)?.into())
    }

    /// Returns the [`TagId`] of an element in this map given its key. If no value exists with the given
    /// key, this method returns [`None`].
    pub fn get_id<T: ?Sized>(&self, key: &T) -> Option<TagId>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.get_node(&KeyRef::tmp(&key.into()))?;
        Some(node.value.id())
    }

    /// Checks if the given key refers to an element in this map.
    pub fn contains_key<T: ?Sized>(&self, key: &T) -> bool
    where
        for<'a> &'a T: Into<String>,
    {
        self.get_node(&KeyRef::tmp(&key.into())).is_some()
    }

    /// Checks if the given key refers to an element with given ID in this map.
    pub fn is<T: ?Sized>(&self, key: &T, id: TagId) -> bool
    where
        for<'a> &'a T: Into<String>,
    {
        match self.get_node(&KeyRef::tmp(&key.into())) {
            Some(n) => n.value.id() == id,
            None => false,
        }
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
            next: self.head,
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all values (tags) in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn values(&self) -> Values<'_> {
        Values {
            next: self.head,
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all keys and values in this [`TagMap`], in pairs.
    /// The iterator returns elements in the current iteration order of the map.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            next: self.head,
            lifetime: PhantomData,
        }
    }

    /// Moves this map into an iterator that iterates over all keys in this [`TagMap`].
    /// The iterator returns keys in the current iteration order of the map.
    pub fn into_keys(self) -> IntoKeys {
        IntoKeys {
            next: self.head,
            _owner: self,
        }
    }

    /// Moves this map into an iterator that iterates over all values in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn into_values(self) -> IntoValues {
        IntoValues {
            next: self.head,
            _owner: self,
        }
    }

    /// Reverses the order of iteration of this [`TagMap`].
    pub fn reverse(&mut self) {
        // Simply swap next/prev of every node and head/tail of linked list

        let mut cur = self.head;

        while !cur.is_null() {
            let rf = unsafe { &mut *cur };
            cur = rf.next;

            swap(&mut rf.prev, &mut rf.next);
        }

        swap(&mut self.head, &mut self.tail);
    }

    /// Swaps the order of iteration of two elements in this [`TagMap`], given their keys.
    /// This method only succeeds if both keys are associated with values. This method returns
    /// `true` if and only if both keys existed in the map and were swapped. If the keys are
    /// the same, then nothing happens and true is returned if and only if there existed a node
    /// for the given key.
    pub fn swap<T1: ?Sized, T2: ?Sized>(&mut self, key1: &T1, key2: &T2) -> bool
    where
        for<'a> &'a T1: Into<String>,
        for<'a> &'a T2: Into<String>,
    {
        let key1 = key1.into();
        let key2 = key2.into();

        if key1 == key2 {
            return self.get_node(&KeyRef::tmp(&key1)).is_some();
        }

        let Some(node1) = self.get_node_mut(&KeyRef::tmp(&key1.into())) else {
            return false;
        };

        // Reborrow to trick the borrow checker into allowing us
        // to mutably borrow self more than once
        // (safe because we know no concurrency happens here)
        let node1 = unsafe { &mut *(node1 as *mut Node) };

        let Some(node2) = self.get_node_mut(&KeyRef::tmp(&key2.into())) else {
            return false;
        };

        // Reborrow this as well
        let node2 = unsafe { &mut *(node2 as *mut Node) };

        // Borrow self mutably a third time (scandalous!)
        self.swap_nodes(node1, node2);

        true
    }

    // TODO
    // - Sorting
    // - Move to start/end
    // - Move after/before
    // - Next/previous
    // - Insert-after/before
    // - Current insert -> append/prepend
    // - Mutable iterators
    // - Better docs
    // - Extract to more generic structure?

    #[cfg(test)]
    fn assert_neighbour_links(&self, node: &Node) {
        let node_ptr = node as *const Node as *mut Node;

        if node.prev.is_null() {
            // node has no previous, head must point to this node
            assert_eq!(
                self.head, 
                node_ptr,
                "node {:?} has no prev pointer but is not the head node",
                node.key.deref()
            );
        } else {
            // node has previous, previous next must point to this node
            assert_eq!(
                unsafe { (*node.prev).next },
                node_ptr,
                "node {:?} prev pointer does not point to node whose next pointer points back",
                node.key.deref()
            )
        }

        if node.next.is_null() {
            // node has no next, tail must point to this node
            assert_eq!(
                self.tail,
                node_ptr,
                "node {:?} has no next pointer but is not the tail node",
                node.key.deref()
            )
        } else {
            // node has next, next previous must point to this node
            assert_eq!(
                unsafe { (*node.next).prev },
                node_ptr,
                "node {:?} next pointer does not point to node whose prev pointer points back",
                node.key.deref()
            )
        }
    }

    /// Attests the integrity of the [`TagMap`]. This goes over all the internal
    /// state and panics if there are any inconsistencies. Every unit test should
    /// call this to ensure every operation preserves the integrity.
    #[cfg(test)]
    fn assert_integrity(&self) {
        use std::collections::HashSet;

        // head and tail are either both null or both not null.
        // can't have just one null as that would mean the linked
        // list has elements from the start but is empty from the end
        // and that makes no sense
        if self.head.is_null() != self.tail.is_null() {
            panic!("must have head without tail or vice versa; head: {:?}, tail: {:?}", self.head, self.tail);
        }

        // collect all keys in a set
        let mut keys = HashSet::new();
        for (key, val) in self.map.iter() {
            // key must refer to correct address.
            // this can fail if a node is inserted under an existing key,
            // as HashMap will reuse the old key value, leaving a dangling
            // pointer once the old node is dropped
            assert_eq!(
                key.value,
                val.key.as_ref() as *const String,
                "key ref of {:?} points to wrong address", val.key.as_ref()
            );

            // test linking status of node
            self.assert_neighbour_links(val.deref());

            // keep track of key
            keys.insert(unsafe { &*(*key).value });
        }

        // go over linked list and see if every node is there
        let mut cur = self.head;
        while !cur.is_null() {
            unsafe {
                // remove key from set.
                // if that fails, then either the node is not in the
                // map but is in the linked list, or the linked list
                // cycles in on itself. neither should happen.
                if !keys.remove((*cur).key.deref()) {
                    panic!("element of linked list named {:?} was not in map: is there a cycle?", (*cur).key.deref());
                }

                // go to next node
                cur = (*cur).next;
            }
        }

        // the set of keys should be empty now, otherwise there are nodes
        // in the map that are not in the list. we can have this despite
        // checking all the node linking, as nodes can link in a cycle and
        // never be reachable from the linked list's ends.
        for key in keys {
            panic!("node with key {:?} is not part of linked list", key)
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
            next: self.head,
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

impl<S, T> From<HashMap<S, T>> for TagMap where S : Into<String>, T : Into<Tag> {
    fn from(value: HashMap<S, T>) -> Self {
        let mut map = Self::new();
        for (k, v) in value {
            map.insert(k, v);
        }
        map
    }
}



pub struct Values<'a> {
    next: *const Node,
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
    next: *const Node,
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
    next: *const Node,
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
    next: *mut Node,
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
    next: *mut Node,
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
    next: *mut Node,
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
                map.insert($($key)? $($key_expr)?, $value);
            )+
            map
        }
    };
}

pub use tag_map;




// ====================================================================================================
//
//   TAG INTEGRATION
//
// ====================================================================================================

impl Tag {
    pub fn compound<T>(v: T) -> Tag
    where
        T: Into<TagMap>,
    {
        Tag::Compound(v.into())
    }

    pub fn into_tag_map(self) -> Result<TagMap, TagCastError<Self>> {
        if let Self::Compound(v) = self {
            Ok(v)
        } else {
            Err(TagCastError::new(TagId::Compound, self.id(), self))
        }
    }
}

impl From<TagMap> for Tag {
    fn from(value: TagMap) -> Self {
        Self::Compound(value)
    }
}






// ====================================================================================================
//
//   TESTING
//
// ====================================================================================================

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
        map.assert_integrity();

        let mut keys = map.into_iter();

        assert_eq!(Some(("foo".into(), Tag::int(3))), keys.next());
        assert_eq!(Some(("bar".into(), Tag::int(4))), keys.next());
        assert_eq!(Some(("baz".into(), Tag::int(5))), keys.next());
        assert_eq!(Some(("gus".into(), Tag::int(6))), keys.next());
        assert_eq!(None, keys.next());

    }

    #[test]
    fn test_basic_inserts_order() {
        let mut map = TagMap::new();
        map.insert("foo", Tag::Byte(3));
        map.insert("bar", Tag::Byte(15));
        map.insert("baz", Tag::Int(34123));
        map.insert("gus", Tag::Int(99128));
        map.assert_integrity();

        let mut keys = map.into_keys();
        assert_eq!(Some("foo".into()), keys.next());
        assert_eq!(Some("bar".into()), keys.next());
        assert_eq!(Some("baz".into()), keys.next());
        assert_eq!(Some("gus".into()), keys.next());
        assert_eq!(None, keys.next());
    }

    #[test]
    fn test_insert_overwrite_order() {
        let mut map = TagMap::new();
        map.insert("foo", Tag::Byte(3));
        map.insert("bar", Tag::Byte(15));
        map.insert("baz", Tag::Int(34123));
        map.insert("bar", Tag::Int(99128));
        map.assert_integrity();

        let mut keys = map.into_keys();
        assert_eq!(Some("foo".into()), keys.next());
        assert_eq!(Some("bar".into()), keys.next());
        assert_eq!(Some("baz".into()), keys.next());
        assert_eq!(None, keys.next());
    }

    #[test]
    fn test_remove_then_insert_order() {
        let mut map = TagMap::new();
        map.insert("foo", Tag::Byte(3));
        map.insert("bar", Tag::Byte(15));
        map.insert("baz", Tag::Int(34123));

        map.remove("bar");
        map.insert("bar", Tag::Int(99128));
        map.assert_integrity();

        let mut keys = map.into_keys();
        assert_eq!(Some("foo".into()), keys.next());
        assert_eq!(Some("baz".into()), keys.next());
        assert_eq!(Some("bar".into()), keys.next());
        assert_eq!(None, keys.next());
    }

    #[test]
    fn test_reverse() {
        let mut map = TagMap::new();
        map.insert("foo", Tag::Byte(3));
        map.insert("bar", Tag::Byte(15));
        map.insert("baz", Tag::Int(34123));
        map.insert("gus", Tag::Int(99128));

        map.reverse();
        map.assert_integrity();

        let mut keys = map.into_keys();
        assert_eq!(Some("gus".into()), keys.next());
        assert_eq!(Some("baz".into()), keys.next());
        assert_eq!(Some("bar".into()), keys.next());
        assert_eq!(Some("foo".into()), keys.next());
        assert_eq!(None, keys.next());
    }

    #[test]
    fn test_swap() {
        let mut map = TagMap::new();
        map.insert("foo", Tag::Byte(3));
        map.insert("bar", Tag::Byte(15));
        map.insert("baz", Tag::Int(34123));
        map.insert("gus", Tag::Int(99128));

        map.swap("foo", "baz");
        map.assert_integrity();

        let mut keys = map.into_keys();
        assert_eq!(Some("baz".into()), keys.next());
        assert_eq!(Some("bar".into()), keys.next());
        assert_eq!(Some("foo".into()), keys.next());
        assert_eq!(Some("gus".into()), keys.next());
        assert_eq!(None, keys.next());
    }
}
