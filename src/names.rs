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

@file    names.rs
@brief   Part of the markup library for names, name spaces, attributes and tags
 */

//a Imports
mod ids;
mod namespace;
mod namespace_stack;
mod name;
mod attribute;
mod tag;

//a Exports
pub use ids::{NSNameId, NSPrefixId, NSUriId, NSMap};
pub use namespace::Namespace;
pub use namespace_stack::NamespaceStack;
pub use name::Name;
pub use attribute::{Attribute, Attributes};
pub use tag::Tag;
