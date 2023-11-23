/*a Copyright

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

@file    namespace.rs
@brief   A markup namespace
 */

/*a Imports
*/
use super::{NSMap, NSNameId, NSPrefixId, NSUriId};
use std::collections::HashSet;

//a Namespace
//tt NamespaceDisplay
pub trait NamespaceDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

//tp Namespace
/// [Namespace] is a set of namespace prefixes and URIs, and mappings
/// between them, that have been encountered or are defaults.
///
/// It also contains a stack of active mappings
///
/// More complex implementations will move to using BTree for the prefixes
pub struct Namespace {
    xmlns: bool,
    /// ALl the
    prefixes: Vec<String>,
    uris: Vec<String>,
    names: Vec<String>,
    mappings: HashSet<NSMap>,
}

//ip Namespace
impl Namespace {
    //fp new
    /// Create a new Namespace object
    pub fn new(xmlns: bool) -> Self {
        let uris = Vec::new();
        let prefixes = Vec::new();
        let names = Vec::new();
        let mappings = HashSet::new();
        Self {
            xmlns,
            uris,
            prefixes,
            names,
            mappings,
        }
    }

    //mp uses_xmlns
    /// Returns true if the [Namespace] was constructed indicating it
    /// should provide the standard XMLNS
    pub fn uses_xmlns(&self) -> bool {
        self.xmlns
    }

    //mp find_name
    /// Find a name within the [Namespace]; return a None if not
    /// found, or Some(NSNameId) if it is. An empty name *is* an
    /// NSNameId::None
    pub fn find_name(&mut self, name: &str) -> Option<NSNameId> {
        if name == "" {
            Some(NSNameId::none())
        } else {
            for (i, p) in self.names.iter().enumerate() {
                if *p == *name {
                    return Some(NSNameId::new(i));
                }
            }
            None
        }
    }

    //mp find_prefix
    /// Find a prefix within the [Namespace]; return a None if not
    /// found, or Some(NSPrefixId) if it is. An empty name *is* an
    /// NSPrefixId::None
    pub fn find_prefix(&mut self, prefix: &str) -> Option<NSPrefixId> {
        if prefix == "" {
            Some(NSPrefixId::none())
        } else {
            for (i, p) in self.prefixes.iter().enumerate() {
                if *p == *prefix {
                    return Some(NSPrefixId::new(i));
                }
            }
            None
        }
    }

    //mp find_uri
    /// Find a URI within the [Namespace]; return a None if not
    /// found, or Some(NSUriId) if it is. An empty name *is* an
    /// NSUriId::None
    pub fn find_uri(&mut self, uri: &str) -> Option<NSUriId> {
        if uri == "" {
            Some(NSUriId::none())
        } else {
            for (i, p) in self.uris.iter().enumerate() {
                if *p == *uri {
                    return Some(NSUriId::new(i));
                }
            }
            None
        }
    }

    //mp find_or_add_name
    /// Find a name within the Namespace; if it is not found then add it
    pub(crate) fn find_or_add_name(&mut self, name: &str) -> NSNameId {
        if let Some(id) = self.find_name(name) {
            id
        } else {
            let n = self.names.len();
            self.names.push(name.into());
            NSNameId::new(n)
        }
    }

    //mp find_or_add_prefix
    /// Find a prefix within the Namespace; if it is not found then add it
    pub(crate) fn find_or_add_prefix(&mut self, prefix: &str) -> NSPrefixId {
        if let Some(id) = self.find_prefix(prefix) {
            id
        } else {
            let n = self.prefixes.len();
            self.prefixes.push(prefix.into());
            NSPrefixId::new(n)
        }
    }

    //mp find_or_add_uri
    /// Find a URI within the Namespace; if it is not found then add it
    fn find_or_add_uri(&mut self, uri: &str) -> NSUriId {
        if let Some(id) = self.find_uri(uri) {
            id
        } else {
            let n = self.uris.len();
            self.uris.push(uri.into());
            NSUriId::new(n)
        }
    }

    //ap name_str
    /// Borrow the `str` of a [NSNameId] within the [Namespace]
    pub fn name_str<'a>(&'a self, name: NSNameId, default: &'a str) -> &'a str {
        if name.is_none() {
            default
        } else {
            &self.names[name.get().unwrap()]
        }
    }

    //mp prefix_str
    /// Borrow the `str` of a [NSPrefixId] within the [Namespace]
    pub fn prefix_str<'a>(&'a self, prefix: NSPrefixId, default: &'a str) -> &'a str {
        if prefix.is_none() {
            default
        } else {
            &self.prefixes[prefix.get().unwrap()]
        }
    }

    //mp uri_str
    /// Borrow the `str` of a [NSUriId] within the [Namespace]
    pub fn uri_str<'a>(&'a self, uri: NSUriId, default: &'a str) -> &'a str {
        if uri.is_none() {
            default
        } else {
            &self.uris[uri.get().unwrap()]
        }
    }

    //mp add_mapping
    /// Add a mapping from a prefix to a URI
    pub fn add_mapping(&mut self, prefix: &str, uri: &str) -> NSMap {
        let p_id = self.find_or_add_prefix(prefix);
        let u_id = self.find_or_add_uri(uri);
        self.add_mapping_by_id(p_id, u_id)
    }

    //mp add_mapping_by_id
    /// Add a mapping from a prefix to a URI
    pub fn add_mapping_by_id(&mut self, prefix_id: NSPrefixId, uri_id: NSUriId) -> NSMap {
        let ns_map = NSMap::new(prefix_id, uri_id);
        if !self.mappings.contains(&ns_map) {
            self.mappings.insert(ns_map);
        }
        ns_map
    }

    //zz All done
}

//a Test
#[cfg(test)]
mod test {}
