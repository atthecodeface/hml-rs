/*a Imports
*/
use std::collections::{HashSet};
use crate::{NSNameId, NSPrefixId, NSUriId, NSMap};

//a Namespace
//tt NamespaceDisplay
pub trait NamespaceDisplay {
    fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result;
}

//tp Namespace
/// [Namespace] is a set of namespace prefixes and URIs, and mappings
/// between them, that have been encountered or are defaults.
///
/// It also contains a stack of active mappings
///
/// More complex implementations will move to using BTree for the prefixes
pub struct Namespace {
    xmlns : bool,
     /// ALl the
    prefixes : Vec<String>,
    uris     : Vec<String>,
    names    : Vec<String>,
    mappings : HashSet<NSMap>,
}

impl Namespace {
    //fp new
    /// Create a new Namespace object
    pub fn new(xmlns:bool) -> Self {
        let uris = Vec::new();
        let prefixes = Vec::new();
        let names = Vec::new();
        let mappings = HashSet::new();
        Self { xmlns, uris, prefixes, names, mappings }
    }

    //mp uses_xmlns
    pub fn uses_xmlns(&self) -> bool {
        self.xmlns
    }

    //mp find_name
    pub fn find_name(&mut self, name:&str) -> Option<NSNameId> {
        if name == "" {
            Some(NSNameId::none())
        } else {
            for (i,p) in self.names.iter().enumerate() {
                if *p == *name { return Some(NSNameId::new(i)); }
            }
            None
        }
    }

    //mp find_prefix
    pub fn find_prefix(&mut self, prefix:&str) -> Option<NSPrefixId> {
        if prefix == "" {
            Some(NSPrefixId::none())
        } else {
            for (i,p) in self.prefixes.iter().enumerate() {
                if *p == *prefix { return Some(NSPrefixId::new(i)); }
            }
            None
        }
    }

    //mp find_uri
    pub fn find_uri(&mut self, uri:&str) -> Option<NSUriId> {
        if uri == "" {
            Some(NSUriId::none())
        } else {
            for (i,p) in self.uris.iter().enumerate() {
                if *p == *uri { return Some(NSUriId::new(i)); }
            }
            None
        }
    }

    //mp find_or_add_name
    pub(crate) fn find_or_add_name(&mut self, name:&str) -> NSNameId {
        if let Some(id) = self.find_name(name) {
            id
        } else {
            let n = self.names.len();
            self.names.push(name.into());
            NSNameId::new(n)
        }
    }

    //mp find_or_add_prefix
    pub(crate) fn find_or_add_prefix(&mut self, prefix:&str) -> NSPrefixId {
        if let Some(id) = self.find_prefix(prefix) {
            id
        } else {
            let n = self.prefixes.len();
            self.prefixes.push(prefix.into());
            NSPrefixId::new(n)
        }
    }

    //mp find_or_add_uri
    fn find_or_add_uri(&mut self, uri:&str) -> NSUriId {
        if let Some(id) = self.find_uri(uri) {
            id
        } else {
            let n = self.uris.len();
            self.uris.push(uri.into());
            NSUriId::new(n)
        }
    }

    //mp borrow_name_str
    pub fn borrow_name_str(&self, name:NSNameId) -> &str {
        if name.is_none() {
            ""
        } else {
            &self.names[name.get().unwrap()]
        }
    }

    //mp borrow_prefix_str
    pub fn borrow_prefix_str(&self, prefix:NSPrefixId) -> &str {
        if prefix.is_none() {
            ""
        } else {
            &self.prefixes[prefix.get().unwrap()]
        }
    }

    //mp borrow_uri_str
    pub fn borrow_uri_str(&self, uri:NSUriId) -> &str {
        if uri.is_none() {
            ""
        } else {
            &self.uris[uri.get().unwrap()]
        }
    }

    //mp add_mapping
    pub fn add_mapping(&mut self, prefix:&str, uri:&str) -> NSMap {
        let p_id = self.find_or_add_prefix(prefix);
        let u_id = self.find_or_add_uri(uri);
        self.add_mapping_by_id(p_id, u_id)
    }

    //mp add_mapping_by_id
    pub fn add_mapping_by_id(&mut self, prefix_id:NSPrefixId, uri_id:NSUriId) -> NSMap {
        let ns_map = NSMap::new(prefix_id, uri_id);
        if !self.mappings.contains(&ns_map) {
            self.mappings.insert(ns_map);
        }
        ns_map
    }
}

//a Test
#[cfg(test)]
mod test {
}
