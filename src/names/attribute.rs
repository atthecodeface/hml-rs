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

@file    attribute.rs
@brief   Markup attribute types
 */

//a Imports
use super::{Name, NamespaceStack};

//a Attribute
//tp Attribute
/// An [Attribute] has a [Name] and a [String] value.
///
/// They correspond to attributes in markup tags
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Attribute {
    /// Name and optional namespace
    pub name: Name,

    /// Attribute value.
    pub value: String,
}

impl Attribute {
    //fp new
    /// Create a new [Attribute] using the [NamespaceStack] to resolve the name
    pub fn new(
        ns_stack: &mut NamespaceStack,
        prefix: &str,
        name: &str,
        value: String,
    ) -> crate::markup::Result<Self> {
        if ns_stack.uses_xmlns() {
            if prefix == "" && name == "xmlns" {
                println!("Add ns '' to be {}", value);
                ns_stack.add_ns("", &value);
                let name = Name::new(ns_stack, name, name)?;
                return Ok(Self { name, value });
            } else if prefix == "xmlns" {
                println!("Add ns {} to be value {}", name, value);
                ns_stack.add_ns(name, &value);
            }
        }
        let name = Name::new(ns_stack, prefix, name)?;
        Ok(Self { name, value })
    }

    //zz All done
}

//a Attributes
//tp Attributes
/// A list of attributes in the order in which they appear in the
/// markup stream
#[derive(Debug)]
pub struct Attributes {
    //
    attributes: Vec<Attribute>,
}

//ip Attributes
impl Attributes {
    //fp new
    /// Create a new list of [Attribute]
    pub fn new() -> Self {
        Self {
            attributes: Vec::new(),
        }
    }
    //mp is_empty
    /// Returns true if the [Attributes] list is empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    //mp add
    /// Add a prefix/name and value to the [Attributes] list, using
    /// the [NamespaceStack] to resolve the prefix into a URI
    pub fn add(
        &mut self,
        ns_stack: &mut NamespaceStack,
        prefix: &str,
        name: &str,
        value: String,
    ) -> crate::markup::Result<()> {
        self.attributes
            .push(Attribute::new(ns_stack, prefix, name, value)?);
        Ok(())
    }

    //mp steal
    /// Take all the attributes away from another [Attributes] and add them to this
    pub fn steal(&mut self, v: &mut Self) {
        self.attributes.append(&mut v.attributes);
    }

    //mp take
    /// Deconstruct this list of [Attribute] to a `Vec<Attribute>`
    pub fn take(self) -> Vec<Attribute> {
        self.attributes
    }

    //mp borrow
    /// Borrow the [Attribute] vec
    pub fn borrow(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    //zz All done
}
