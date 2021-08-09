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

@file    name.rs
@brief   Markup name (prefix => URI and name within namespace)
 */

//a Imports
use super::NamespaceStack;
use super::{NSNameId, NSPrefixId, NSUriId};
use crate::markup;

//a Name
//tp Name
/// A name within a markup stream consisting of ids for a prefix and
/// the actual name; the prefix can be the none id.
///
/// If the prefix is not none, then the URI id will correspond to the
/// id of a URI that the prefix has been declared to map to (given the
/// namespace stack)
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Name {
    /// A name prefix, e.g. `xsi` in `xsi:string`.
    pub prefix: NSPrefixId,

    /// A namespace URI, e.g. `http://www.w3.org/2000/xmlns/`.
    pub uri: NSUriId,

    /// A name
    pub name: NSNameId,
}

//ip Name
impl Name {
    //fp none
    /// Create a `None` [Name]
    pub fn none() -> Self {
        let prefix = NSPrefixId::none();
        let uri = NSUriId::none();
        let name = NSNameId::none();
        Self { prefix, uri, name }
    }

    //fp new_local
    /// Create a [Name] from a name with an prefix of None
    pub fn new_local(ns: &mut NamespaceStack, name: &str) -> markup::Result<Self> {
        if name.is_empty() {
            return crate::markup::Error::empty_name();
        }
        let prefix = NSPrefixId::none();
        let uri = NSUriId::none();
        let name = ns.add_name(name);
        Ok(Self { prefix, uri, name })
    }

    //fp new
    /// Create a [Name] fom a prefix string and a name
    ///
    /// If the name is illegal (empty) or the prefix is not mapped (an
    /// empty prefix can be mapped in the namespace by default, so an
    /// empty prefix is not illegal per s) then an error is returned.
    pub fn new(ns: &mut NamespaceStack, prefix: &str, name: &str) -> crate::markup::Result<Self> {
        if name.is_empty() {
            return crate::markup::Error::empty_name();
        }
        if let Some(p_id) = ns.find_prefix_id(prefix) {
            if let Some(uri) = ns.find_mapping(p_id) {
                let name = ns.add_name(name);
                Ok(Self {
                    prefix: p_id,
                    uri,
                    name,
                })
            } else {
                crate::markup::Error::unmapped_prefix(prefix)
            }
        } else {
            crate::markup::Error::unmapped_prefix(prefix)
        }
    }

    //fp from_str
    /// Create a [Name] fom a string given the current
    /// [NamespaceStack], returning an error if the namespace is
    /// unmapped or the input string is malformed
    ///
    /// A well-formed string is either <ns>:<name> or <name>; two ':'
    /// characters are illegal
    pub fn from_str(ns: &mut NamespaceStack, s: &str) -> crate::markup::Result<Self> {
        let mut it = s.split(':');
        match (it.next(), it.next(), it.next()) {
            (Some(prefix), Some(name), None) => Self::new(ns, prefix, name),
            (Some(name), None, None) => Self::new_local(ns, name),
            _ => crate::markup::Error::bad_name(s),
        }
    }

    //fp to_string
    /// Create a new `String` of the [Name]
    pub fn to_string(&self, ns: &NamespaceStack) -> String {
        if self.prefix.is_none() {
            format!("{}", ns.borrow_name(self.name))
        } else {
            format!(
                "{}:{}",
                ns.borrow_prefix(self.prefix),
                ns.borrow_name(self.name)
            )
        }
    }
}
