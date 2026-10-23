use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
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
    fn link_overwrite(&mut self, existing: &Node, new: &mut Node) {
        let prev = existing.prev;
        let next = existing.next;

        let node_ptr = new as *mut Node;

        // Update previous link
        new.prev = prev;
        if prev.is_null() {
            self.head = node_ptr;
        } else {
            unsafe {
                (*prev).next = node_ptr;
            }
        }

        // Update next link
        new.next = next;
        if next.is_null() {
            self.tail = node_ptr;
        } else {
            unsafe {
                (*next).prev = node_ptr;
            }
        }
    }

    /// Append a node to the linked list.
    fn link_push(&mut self, node: &mut Node, end: bool) {
        let node_ptr = node as *mut Node;
        if end {
            unsafe {
                if self.head.is_null() {
                    // First node
                    self.head = node_ptr;
                    node.prev = null_mut();

                    self.tail = node_ptr;
                    node.next = null_mut();
                } else {
                    // Not first node
                    (*self.tail).next = node_ptr;
                    node.prev = self.tail;

                    node.next = null_mut();
                    self.tail = node;
                };
            };
        } else {
            unsafe {
                if self.tail.is_null() {
                    // First node
                    self.tail = node_ptr;
                    node.next = null_mut();

                    self.head = node_ptr;
                    node.prev = null_mut();
                } else {
                    // Not first node
                    (*self.head).prev = node_ptr;
                    node.next = self.head;

                    node.prev = null_mut();
                    self.head = node;
                };
            };
        }
    }

    /// Removes a node from the linked list.
    fn link_remove(&mut self, node: &Node) {
        unsafe {
            if node.prev.is_null() && node.next.is_null() {
                // Node was only node
                
                self.head = null_mut();
                self.tail = null_mut();
            } else if node.prev.is_null() {
                // Node was first node

                let next = node.next;

                (*next).prev = null_mut();
                self.head = next;
            } else if node.next.is_null() {
                // Node was last node

                let prev = node.prev;

                (*prev).next = null_mut();
                self.tail = prev;
            } else {
                // Node was neither first nor last

                let prev = node.prev;
                let next = node.next;

                (*prev).next = next;
                (*next).prev = prev;
            };
        }
    }

    /// Removes a node from the linked list.
    fn link_insert_between(&mut self, node: &mut Node, prev: *mut Node, next: *mut Node) {
        unsafe {
            if prev.is_null() {
                self.head = node as *mut Node;
            } else {
                (*prev).next = node as *mut Node;
            }

            if next.is_null() {
                self.tail = node as *mut Node;
            } else {
                (*next).prev = node as *mut Node;
            }

            node.next = next;
            node.prev = prev;
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
        self.link_push(boxed_node.as_mut(), end);

        // Insert node into map
        self.map.insert(key_ref, boxed_node);
    }

    fn remove_node(&mut self, key: &KeyRef) -> Option<Node> {
        let boxed_node = self.map.remove(key)?;

        let mut node = *boxed_node;

        self.link_remove(&mut node);

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
        
        // Get a key reference
        let key_ref = node.key_ref();

        // Box node to fix its pointer
        let mut boxed_node = Box::new(node);

        // Relink in place
        let old = if let Some(existing) = self.map.get(&key_ref) {
            // Reborrow to satisfy borrow chekcer
            let existing = unsafe { &*(existing.deref() as *const _) };

            // Overwrite node in place
            self.link_overwrite(existing, boxed_node.as_mut());

            // We must explicitly remove the element first so that
            // the KeyRef is properly updated. Also allows us to move
            // out the node early
            Some(self.map.remove(&key_ref).unwrap().value)
        } else {
            self.link_push(boxed_node.as_mut(), true);

            None
        };

        // Update node in map
        self.map.insert(key_ref, boxed_node);
        old
    }

    fn prepare_insert(&mut self, key: String, value: Tag) -> (Option<Tag>, KeyRef, Box<Node>) {
        let node = Node::new(key.into(), value.into());
        
        // Get a key reference
        let key_ref = node.key_ref();

        // Box node to fix its pointer
        let boxed_node = Box::new(node);

        // Relink in place
        let old = if let Some(existing) = self.map.get(&key_ref) {
            // Reborrow to satisfy borrow chekcer
            let existing = unsafe { &*(existing.deref() as *const _) };

            // Remove existing link
            self.link_remove(existing);

            // We must explicitly remove the element first so that
            // the KeyRef is properly updated. Also allows us to move
            // out the node early
            Some(self.map.remove(&key_ref).unwrap().value)
        } else {
            None
        };

        (old, key_ref, boxed_node)
    }

    fn insert_at_end(&mut self, key: impl Into<String>, value: impl Into<Tag>, tail: bool) -> Option<Tag> {
        // prepare insert
        let (old, key_ref, mut boxed_node) = self.prepare_insert(key.into(), value.into());

        // insert in linked list
        self.link_push(boxed_node.as_mut(), tail);

        // insert in map
        self.map.insert(key_ref, boxed_node);

        old
    }

    /// Insert a [`Tag`] under the given key into this [`TagMap`]. If there was a tag previously
    /// stored with the same key, then that tag is moved out and returned.
    ///
    /// # Order
    /// This method appends all keys at the end of the iteration order, moving existing keys to the
    /// end. That is, reinserting `foo` and then `bar` into an empty map creates an iteration order
    /// of `foo` and then `bar`, and inserting a new value for `foo` will change the order into `bar`
    /// then `foo`.
    pub fn insert_at_tail(&mut self, key: impl Into<String>, value: impl Into<Tag>) -> Option<Tag> {
        self.insert_at_end(key, value, true)
    }

    /// Insert a [`Tag`] under the given key into this [`TagMap`]. If there was a tag previously
    /// stored with the same key, then that tag is moved out and returned.
    ///
    /// # Order
    /// This method prepends all keys at the start of the iteration order, moving existing keys to the
    /// start. That is, reinserting `foo` and then `bar` into an empty map creates an iteration order
    /// of `bar` and then `foo`, and inserting a new value for `foo` will change the order into `foo`
    /// then `bar`.
    pub fn insert_at_head(&mut self, key: impl Into<String>, value: impl Into<Tag>) -> Option<Tag> {
        self.insert_at_end(key, value, false)
    }

    pub fn insert_after<T>(&mut self, key: impl Into<String>, value: impl Into<Tag>, target: &T) -> Option<Tag>
    where
        for<'a> &'a T : Into<String>
    {
        let Some(target) = self.get_node_mut(&KeyRef::tmp(&target.into())) else {
            return self.insert_at_end(key, value, true);
        };

        let prev = target as *mut Node;
        let next = target.next;

        // prepare insert
        let (old, key_ref, mut boxed_node) = self.prepare_insert(key.into(), value.into());
        
        // insert in linked list
        self.link_insert_between(boxed_node.as_mut(), prev, next);

        // insert in map
        self.map.insert(key_ref, boxed_node);

        old
    }

    pub fn insert_before<T>(&mut self, key: impl Into<String>, value: impl Into<Tag>, target: &T) -> Option<Tag>
    where
        for<'a> &'a T : Into<String>
    {
        let Some(target) = self.get_node_mut(&KeyRef::tmp(&target.into())) else {
            return self.insert_at_end(key, value, true);
        };

        let prev = target.prev;
        let next = target as *mut Node;

        // prepare insert
        let (old, key_ref, mut boxed_node) = self.prepare_insert(key.into(), value.into());
        
        // insert in linked list
        self.link_insert_between(boxed_node.as_mut(), prev, next);

        // insert in map
        self.map.insert(key_ref, boxed_node);

        old
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

    fn node_iter(&self) -> NodeIterator {
        NodeIterator {
            head: self.head,
            tail: self.tail,
            end: false
        }
    }

    /// Returns an iterator that iterates over references to all keys in this [`TagMap`].
    /// The iterator returns keys in the current iteration order of the map.
    pub fn keys(&self) -> Keys<'_> {
        Keys {
            iter: self.node_iter(),
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all values (tags) in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn values(&self) -> Values<'_> {
        Values {
            iter: self.node_iter(),
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all values (tags) in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn values_mut(&mut self) -> ValuesMut<'_> {
        ValuesMut {
            iter: self.node_iter(),
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all values (tags) in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn value_refs(&self) -> ValueRefs<'_> {
        ValueRefs {
            iter: self.values()
        }
    }

    /// Returns an iterator that iterates over references to all values (tags) in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn value_refs_mut(&mut self) -> ValueRefsMut<'_> {
        ValueRefsMut {
            iter: self.values_mut()
        }
    }

    /// Returns an iterator that iterates over references to all keys and values in this [`TagMap`], in pairs.
    /// The iterator returns elements in the current iteration order of the map.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            iter: self.node_iter(),
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all keys and values in this [`TagMap`], in pairs.
    /// The iterator returns elements in the current iteration order of the map.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            iter: self.node_iter(),
            lifetime: PhantomData,
        }
    }

    /// Returns an iterator that iterates over references to all keys and values in this [`TagMap`], in pairs.
    /// The iterator returns elements in the current iteration order of the map.
    pub fn refs(&self) -> Refs<'_> {
        Refs {
            iter: self.iter()
        }
    }

    /// Returns an iterator that iterates over references to all keys and values in this [`TagMap`], in pairs.
    /// The iterator returns elements in the current iteration order of the map.
    pub fn refs_mut(&mut self) -> RefsMut<'_> {
        RefsMut {
            iter: self.iter_mut()
        }
    }

    /// Moves this map into an iterator that iterates over all keys in this [`TagMap`].
    /// The iterator returns keys in the current iteration order of the map.
    pub fn into_keys(self) -> IntoKeys {
        IntoKeys {
            iter: self.node_iter(),
            _owner: self,
        }
    }

    /// Moves this map into an iterator that iterates over all values in this [`TagMap`].
    /// The iterator returns values in the current iteration order of the map.
    pub fn into_values(self) -> IntoValues {
        IntoValues {
            iter: self.node_iter(),
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

    pub fn head(&self) -> Option<(&String, &Tag)> {
        if self.head.is_null() {
            return None;
        }

        let node = unsafe { &*self.head };
        Some((node.key.as_ref(), &node.value))
    }

    pub fn head_mut(&mut self) -> Option<(&String, &mut Tag)> {
        if self.head.is_null() {
            return None;
        }

        let node = unsafe { &mut *self.head };
        Some((node.key.as_ref(), &mut node.value))
    }

    pub fn head_ref(&self) -> Option<(&String, TagRef<'_>)> {
        self.head().map(|(k, v)| (k, v.into()))
    }

    pub fn head_ref_mut(&mut self) -> Option<(&String, TagRefMut<'_>)> {
        self.head_mut().map(|(k, v)| (k, v.into()))
    }

    pub fn tail(&self) -> Option<(&String, &Tag)> {
        if self.tail.is_null() {
            return None;
        }

        let node = unsafe { &*self.tail };
        Some((node.key.as_ref(), &node.value))
    }

    pub fn tail_mut(&mut self) -> Option<(&String, &mut Tag)> {
        if self.tail.is_null() {
            return None;
        }

        let node = unsafe { &mut *self.tail };
        Some((node.key.as_ref(), &mut node.value))
    }

    pub fn tail_ref(&self) -> Option<(&String, TagRef<'_>)> {
        self.tail().map(|(k, v)| (k, v.into()))
    }

    pub fn tail_ref_mut(&mut self) -> Option<(&String, TagRefMut<'_>)> {
        self.tail_mut().map(|(k, v)| (k, v.into()))
    }

    pub fn next<T>(&self, key: &T) -> Option<(&String, &Tag)>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.get_node(&KeyRef::tmp(&key.into()))?;
        if node.next.is_null() {
            return None;
        }

        let node = unsafe { &*node.next };
        Some((node.key.as_ref(), &node.value))
    }

    pub fn next_mut<T>(&mut self, key: &T) -> Option<(&String, &mut Tag)>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.get_node_mut(&KeyRef::tmp(&key.into()))?;
        if node.next.is_null() {
            return None;
        }

        let node = unsafe { &mut *node.next };
        Some((node.key.as_ref(), &mut node.value))
    }

    pub fn next_ref<T>(&self, key: &T) -> Option<(&String, TagRef<'_>)>
    where
        for<'a> &'a T: Into<String>,
    {
        self.next(key).map(|(k, v)| (k, v.into()))
    }

    pub fn next_ref_mut<T>(&mut self, key: &T) -> Option<(&String, TagRefMut<'_>)>
    where
        for<'a> &'a T: Into<String>,
    {
        self.next_mut(key).map(|(k, v)| (k, v.into()))
    }

    pub fn prev<T>(&self, key: &T) -> Option<(&String, &Tag)>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.get_node(&KeyRef::tmp(&key.into()))?;
        if node.prev.is_null() {
            return None;
        }

        let node = unsafe { &*node.prev };
        Some((node.key.as_ref(), &node.value))
    }

    pub fn prev_mut<T>(&mut self, key: &T) -> Option<(&String, &mut Tag)>
    where
        for<'a> &'a T: Into<String>,
    {
        let node = self.get_node_mut(&KeyRef::tmp(&key.into()))?;
        if node.prev.is_null() {
            return None;
        }

        let node = unsafe { &mut *node.prev };
        Some((node.key.as_ref(), &mut node.value))
    }

    pub fn prev_ref<T>(&self, key: &T) -> Option<(&String, TagRef<'_>)>
    where
        for<'a> &'a T: Into<String>,
    {
        self.prev(key).map(|(k, v)| (k, v.into()))
    }

    pub fn prev_ref_mut<T>(&mut self, key: &T) -> Option<(&String, TagRefMut<'_>)>
    where
        for<'a> &'a T: Into<String>,
    {
        self.prev_mut(key).map(|(k, v)| (k, v.into()))
    }

    unsafe fn extract_node_order(&self) -> Vec<*mut Node> {
        let mut order = Vec::new();

        let mut cur = self.head;
        while !cur.is_null() {
            order.push(cur);

            cur = unsafe { (*cur).next };
        }

        order
    }

    unsafe fn reconstruct_node_order(&mut self, order: Vec<*mut Node>) {
        debug_assert!(order.len() == self.len());
        
        self.head = null_mut();
        self.tail = null_mut();
        
        unsafe {
            let mut last = None;
            for elem in order {
                if let Some(last) = last {
                    (*elem).prev = last;
                    (*last).next = elem;
                } else {
                    (*elem).prev = null_mut();
                    self.head = elem;
                }

                last = Some(elem);
            }

            if let Some(last) = last {
                self.tail = last;
                (*last).next = null_mut();
            }
        }
    }

    pub fn sort(&mut self) {
        self.sort_keys_by_cached_key(|s| s);
    }

    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        for<'a> F : FnMut((&'a String, &'a Tag), (&'a String, &'a Tag)) -> Ordering
    {
        unsafe {
            let mut order = self.extract_node_order();
            order.sort_by(|l, r| compare(((**l).key.deref(), &(**l).value), ((**r).key.deref(), &(**r).value)));
            self.reconstruct_node_order(order);
        }
    }

    pub fn sort_keys_by<F>(&mut self, mut compare: F)
    where
        for<'a> F : FnMut(&'a String, &'a String) -> Ordering
    {
        self.sort_by(|(l, _), (r, _)| compare(l, r));
    }

    pub fn sort_values_by<F>(&mut self, mut compare: F)
    where
        for <'a> F : FnMut(&'a Tag, &'a Tag) -> Ordering
    {
        self.sort_by(|(_, l), (_, r)| compare(l, r));
    }

    pub fn sort_by_key<F, O>(&mut self, mut f: F)
    where
        for<'a> F : FnMut((&'a String, &'a Tag)) -> &'a O,
        O : Ord
    {
        unsafe {
            let mut order = self.extract_node_order();
            order.sort_by_key(|v| f(((**v).key.deref(), &(**v).value)));
            self.reconstruct_node_order(order);
        }
    }

    pub fn sort_keys_by_key<F, O>(&mut self, mut f: F)
    where
        for<'a> F : FnMut(&'a String) -> &'a O,
        O : Ord
    {
        self.sort_by_key(|(v, _)| f(v));
    }

    pub fn sort_values_by_key<F, O>(&mut self, mut f: F)
    where
        for<'a> F : FnMut(&'a Tag) -> &'a O,
        O : Ord
    {
        self.sort_by_key(|(_, v)| f(v));
    }

    pub fn sort_by_cached_key<F, O>(&mut self, mut f: F)
    where
        for<'a> F : FnMut((&'a String, &'a Tag)) -> &'a O,
        O : Ord
    {
        unsafe {
            let mut order = self.extract_node_order();
            order.sort_by_cached_key(|v| f(((**v).key.deref(), &(**v).value)));
            self.reconstruct_node_order(order);
        }
    }

    pub fn sort_keys_by_cached_key<F, O>(&mut self, mut f: F)
    where
        for<'a> F : FnMut(&'a String) -> &'a O,
        O : Ord
    {
        self.sort_by_cached_key(|(v, _)| f(v));
    }

    pub fn sort_values_by_cached_key<F, O>(&mut self, mut f: F)
    where
        for<'a> F : FnMut(&'a Tag) -> &'a O,
        O : Ord
    {
        self.sort_by_cached_key(|(_, v)| f(v));
    }

    pub fn move_to_head<T>(&mut self, key: &T) -> bool
    where
        for<'a> &'a T: Into<String>
    {
        let Some(node) = self.get_node_mut(&KeyRef::tmp(&key.into())) else {
            return false;
        };
        
        // Reborrow to regain mutable access to self
        let node = unsafe { &mut *(node as *mut Node) };

        self.link_remove(node);
        self.link_push(node, false);

        true
    }

    pub fn move_to_tail<T>(&mut self, key: &T) -> bool
    where
        for<'a> &'a T: Into<String>
    {
        let Some(node) = self.get_node_mut(&KeyRef::tmp(&key.into())) else {
            return false;
        };
        
        // Reborrow to regain mutable access to self
        let node = unsafe { &mut *(node as *mut Node) };

        self.link_remove(node);
        self.link_push(node, true);

        true
    }

    pub fn move_after<T1, T2>(&mut self, key: &T1, target: &T2) -> bool
    where
        for<'a> &'a T1: Into<String>,
        for<'a> &'a T2: Into<String>,
    {
        let Some(node) = self.get_node_mut(&KeyRef::tmp(&key.into())) else {
            return false;
        };

        // Reborrow to regain mutable access to self
        let node = unsafe { &mut *(node as *mut Node) };

        let Some(target) = self.get_node_mut(&KeyRef::tmp(&target.into())) else {
            return false;
        };

        let prev = target as *mut Node;
        let next = target.next;

        self.link_remove(node);
        self.link_insert_between(node, prev, next);

        true
    }

    pub fn move_before<T1, T2>(&mut self, key: &T1, target: &T2) -> bool
    where
        for<'a> &'a T1: Into<String>,
        for<'a> &'a T2: Into<String>,
    {
        let Some(node) = self.get_node_mut(&KeyRef::tmp(&key.into())) else {
            return false;
        };

        // Reborrow to regain mutable access to self
        let node = unsafe { &mut *(node as *mut Node) };

        let Some(target) = self.get_node_mut(&KeyRef::tmp(&target.into())) else {
            return false;
        };

        let prev = target.prev;
        let next = target as *mut Node;

        self.link_remove(node);
        self.link_insert_between(node, prev, next);

        true
    }

    pub fn remove_head(&mut self) -> Option<(String, Tag)> {
        if self.head.is_null() {
            return None;
        }

        let head = self.head;
        let key = unsafe { (*head).key_ref() };
        let node = self.remove_node(&key).unwrap();

        Some((*node.key, node.value))
    }

    pub fn remove_tail(&mut self) -> Option<(String, Tag)> {
        if self.tail.is_null() {
            return None;
        }

        let tail = self.tail;
        let key = unsafe { (*tail).key_ref() };
        let node = self.remove_node(&key).unwrap();

        Some((*node.key, node.value))
    }

    // TODO
    // - Better docs
    // - Extract to more generic structure?
    // - Double ended iterators

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

impl Display for TagMap {
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
            iter: self.node_iter(),
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
        Self::from_iter(value)
    }
}

impl<S, T> From<BTreeMap<S, T>> for TagMap where S : Into<String>, T : Into<Tag> {
    fn from(value: BTreeMap<S, T>) -> Self {
        Self::from_iter(value)
    }
}

impl<S, T> FromIterator<(S, T)> for TagMap where S : Into<String>, T : Into<Tag> {
    fn from_iter<U: IntoIterator<Item = (S, T)>>(iter: U) -> Self {
        let mut map = Self::new();
        for (k, v) in iter {
            map.insert_at_tail(k, v);
        }
        map
    }
}


/// Generic iterator implementation for nodes.
struct NodeIterator {
    head: *mut Node,
    tail: *mut Node,
    end: bool
}

impl NodeIterator {
    fn next(&mut self) -> Option<*mut Node> {
        if self.head.is_null() || self.end {
            return None;
        }

        if self.head == self.tail {
            self.end = true;
            return Some(self.head);
        }

        unsafe {
            let next = self.head;
            self.head = (*next).next;

            Some(next)
        }
    }

    fn next_back(&mut self) -> Option<*mut Node> {
        if self.tail.is_null() || self.end {
            return None;
        }

        if self.head == self.tail {
            self.end = true;
            return Some(self.tail);
        }
        
        unsafe {
            let prev = self.tail;
            self.tail = (*prev).prev;

            Some(prev)
        }
    }
}


pub struct Values<'a> {
    iter: NodeIterator,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a Tag;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next()?;

            Some(&(*next).value)
        }
    }
}

impl<'a> DoubleEndedIterator for Values<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next_back()?;

            Some(&(*next).value)
        }
    }
}


pub struct ValuesMut<'a> {
    iter: NodeIterator,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for ValuesMut<'a> {
    type Item = &'a mut Tag;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next()?;

            Some(&mut (*next).value)
        }
    }
}

impl<'a> DoubleEndedIterator for ValuesMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next_back()?;

            Some(&mut (*next).value)
        }
    }
}


pub struct ValueRefs<'a> {
    iter: Values<'a>
}

impl<'a> Iterator for ValueRefs<'a> {
    type Item = TagRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.iter.next()?;
        Some(n.into())
    }
}


pub struct ValueRefsMut<'a> {
    iter: ValuesMut<'a>
}

impl<'a> Iterator for ValueRefsMut<'a> {
    type Item = TagRefMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.iter.next()?;
        Some(n.into())
    }
}


pub struct Keys<'a> {
    iter: NodeIterator,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next()?;

            Some(&*(*next).key)
        }
    }
}

impl<'a> DoubleEndedIterator for Keys<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next_back()?;

            Some(&*(*next).key)
        }
    }
}

pub struct Iter<'a> {
    iter: NodeIterator,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Tag);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next()?;

            Some((&*(*next).key, &(*next).value))
        }
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next_back()?;

            Some((&*(*next).key, &(*next).value))
        }
    }
}

pub struct IterMut<'a> {
    iter: NodeIterator,
    lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a String, &'a mut Tag);

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next()?;

            Some((&*(*next).key, &mut (*next).value))
        }
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next_back()?;

            Some((&*(*next).key, &mut (*next).value))
        }
    }
}


pub struct Refs<'a> {
    iter: Iter<'a>
}

impl<'a> Iterator for Refs<'a> {
    type Item = (&'a String, TagRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.iter.next()?;
        Some((k, v.into()))
    }
}


pub struct RefsMut<'a> {
    iter: IterMut<'a>
}

impl<'a> Iterator for RefsMut<'a> {
    type Item = (&'a String, TagRefMut<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = self.iter.next()?;
        Some((k, v.into()))
    }
}



const REPLACE_TAG: Tag = Tag::Byte(0);
const REPLACE_KEY: String = String::new();

pub struct IntoValues {
    iter: NodeIterator,
    _owner: TagMap, // must keep ownership over pointer owner so the pointers don't get dropped yet
}

impl Iterator for IntoValues {
    type Item = Tag;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next()?;
            let next = &mut (*next).value as *mut Tag;

            Some(replace(next, REPLACE_TAG))
        }
    }
}

impl DoubleEndedIterator for IntoValues {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next_back()?;
            let next = &mut (*next).value as *mut Tag;

            Some(replace(next, REPLACE_TAG))
        }
    }
}

pub struct IntoKeys {
    iter: NodeIterator,
    _owner: TagMap, // must keep ownership over pointer owner so the pointers don't get dropped yet
}

impl Iterator for IntoKeys {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next()?;
            let next = (*next).key.as_mut() as *mut String;

            Some(replace(next, REPLACE_KEY))
        }
    }
}

impl DoubleEndedIterator for IntoKeys {
    fn next_back(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = self.iter.next_back()?;
            let next = (*next).key.as_mut() as *mut String;

            Some(replace(next, REPLACE_KEY))
        }
    }
}

pub struct IntoIter {
    iter: NodeIterator,
    _owner: TagMap, // must keep ownership over pointer owner so the pointers don't get dropped yet
}

impl Iterator for IntoIter {
    type Item = (String, Tag);

    fn next(&mut self) -> Option<Self::Item> {

        unsafe {
            let next = self.iter.next()?;
            let nextk = (*next).key.as_mut() as *mut String;
            let nextv = &mut (*next).value as *mut Tag;

            Some((replace(nextk, REPLACE_KEY), replace(nextv, REPLACE_TAG)))
        }
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {

        unsafe {
            let next = self.iter.next_back()?;
            let nextk = (*next).key.as_mut() as *mut String;
            let nextv = &mut (*next).value as *mut Tag;

            Some((replace(nextk, REPLACE_KEY), replace(nextv, REPLACE_TAG)))
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
