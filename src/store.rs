// Copyright (c) 2026 Sebastian Ibanez

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use crate::error::Error;

/// Stores key-value pairs.
#[derive(Debug, Clone)]
pub struct Store {
    map: HashMap<String, Item>,
}

impl Store {
    /// Initialize new `Store`.
    pub fn new() -> Store {
        Store {
            map: HashMap::new(),
        }
    }

    /// Insert new key-value pair.
    /// Returns `Error::StoreInsertError` if pair cannot be inserted.
    pub fn add(&mut self, key: String, value: Item) -> Result<(), Error> {
        if self.map.get(&key).is_none() {
            println!(
                "[ADD] key: {}\n      value: {}",
                key,
                value.to_string().replace('\n', "\n             ")
            );
            self.map.insert(key.clone(), value.clone());
        }

        Ok(())
    }

    /// Update value in existing pair.
    pub fn get(&self, key: String) -> Option<&Item> {
        if let Some(value) = self.map.get(&key) {
            println!(
                "[GET] key: {}\n      value: {}",
                key,
                value.to_string().replace('\n', "\n             ")
            );
            return Some(&value);
        }
        None
    }

    /// Delete key-value pair.
    pub fn delete(&mut self, key: String) -> Result<(), Error> {
        if let Some(value) = self.map.remove(&key) {
            println!(
                "[DEL] key: {}\n      value: {}",
                key,
                value.to_string().replace('\n', "\n             ")
            );
            return Ok(());
        }

        Err(Error::KeyNotFoundError)
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    String(String),
    List(Vec<String>),
    Set(HashSet<String>),
    HashMap(HashMap<String, String>),
}
/*
    Item string representation:
    String  => abc
    List    => a,b,c
    Set     => [a,b,c]
    HashMap => akey:a,bkey:b,ckey:c
*/

impl Item {
    pub fn from_string(s: String) -> Item {
        // Set
        if s.starts_with('[') && s.ends_with(']') {
            let inner = &s[1..s.len() - 1];
            let set = inner.split(',').map(|v| v.trim().to_string()).collect();
            return Item::Set(set);
        }

        // HashMap (all segments must contain ':')
        if s.contains(',') && s.split(',').all(|seg| seg.contains(':')) {
            let map = s
                .split(',')
                .filter_map(|pair| {
                    let (k, v) = pair.split_once(':')?;
                    Some((k.trim().to_string(), v.trim().to_string()))
                })
                .collect();
            return Item::HashMap(map);
        }

        // List
        if s.contains(',') {
            let list = s.split(',').map(|v| v.trim().to_string()).collect();
            return Item::List(list);
        }

        Item::String(s)
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::String(s) => write!(f, "{s}"),
            Item::List(items) => {
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{item}")?;
                }
                Ok(())
            }
            Item::Set(set) => {
                write!(f, "[")?;
                for (i, member) in set.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{member}")?;
                }
                write!(f, "]")
            }
            Item::HashMap(map) => {
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{k}:{v}")?;
                }
                Ok(())
            }
        }
    }
}
