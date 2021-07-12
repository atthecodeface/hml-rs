
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
