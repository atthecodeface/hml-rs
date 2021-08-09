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

@file    tag.rs
@brief   Markup tag structure and associated construction methods
 */

//a Imports
use super::{Attributes, Name, NamespaceStack};

//a Tag
//tp Tag
#[derive(Debug)]
pub struct Tag {
    /// Name with prefix *and URI from namespace stack*
    ///
    /// Note that the Name depends on the Namespace attributes within the tag
    pub name: Name,

    /// Attributes for the tag, including any local namespaces for the tag
    pub attributes: Attributes,
}

//ip Tag
impl Tag {
    pub fn new(
        ns_stack: &mut NamespaceStack,
        ns: &str,
        name: &str,
        attributes: Attributes,
    ) -> crate::markup::Result<Self> {
        let name = Name::new(ns_stack, ns, name)?;
        Ok(Self { name, attributes })
    }
}
