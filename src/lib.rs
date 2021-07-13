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

@file    lib.rs
@brief   Markup library
 */

//a Documentation
#![warn(missing_docs)]
#![warn(missing_doc_code_examples)]
/*!

# Markup library

This library provides for markup language stream reading and writing.

It

!*/

pub mod escape;

// Expose names::{NSNameId, NSPrefixId, NSUriId, NSMap};
//        names::{Namespace, NamespaceStack, Name, Attribute, Attributes, Tag};
pub mod names;

// Expose markup::Span, Error, Result, Event, EventType
pub mod markup;

// Expose reader::{Position, Character, Error, Reader, Lexer, Parser, Span, ReaderError, Result}
pub mod reader;

mod implementations;
pub use implementations::string;

/*
mod types;
mod utils;

pub mod writer;
 */
