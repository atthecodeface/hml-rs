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
use crate::{MarkupResult, Name, NamespaceStack};

//a Attribute
//tp Attribute
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Attribute {
    /// Name and optional namespace
    pub name: Name,

    /// Attribute value.
    pub value: String
}

impl Attribute {
    pub fn new(ns_stack:&mut NamespaceStack, prefix:&str, name:&str, value:String) -> MarkupResult<Self> {
        if ns_stack.uses_xmlns() {
            if prefix == "" && name == "xmlns" {
                println!("Add ns '' to be {}",value);
                ns_stack.add_ns( "", &value );
                let name = Name::new(ns_stack, name, name)?;
                return Ok(Self { name, value });
            } else if prefix == "xmlns" {
                println!("Add ns {} to be value {}", name, value);
                ns_stack.add_ns( name, &value );
            }
        }
        let name = Name::new(ns_stack, prefix, name)?;
        Ok(Self { name, value })
    }
}

//tp Attributes
#[derive(Debug)]
pub struct Attributes {
    //
    attributes: Vec<Attribute>
}

//ip Attributes
impl Attributes {
    //fp new
    pub fn new() -> Self {
        Self { attributes : Vec::new() }
    }
    //mp is_empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
    pub fn add(&mut self, ns_stack:&mut NamespaceStack, prefix:&str, name:&str, value:String) -> MarkupResult<()> {
        self.attributes.push(Attribute::new(ns_stack, prefix, name, value)?);
        Ok(())
    }
    pub fn steal(&mut self, v:&mut Self) {
        self.attributes.append(&mut v.attributes);
    }
    pub fn take(self) -> Vec<Attribute>  {
        self.attributes
    }
    pub fn borrow(&self) -> &Vec<Attribute>  {
        &self.attributes
    }
}
