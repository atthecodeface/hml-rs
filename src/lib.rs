// Namespaces
//
// The Namespaces type contains sets of namespace URIs and
// identifiers, mappings between them, and stacks of valid mappings
// that change as a document is parsed.
//
mod error;
mod traits;
mod markup;

mod reader;

pub use error::HmlError;
pub use traits::{ReaderPosition, ReaderChar, Reader, StreamSpan};

pub use markup::{NSNameId, NSPrefixId, NSUriId, NSMap};
pub use markup::{Namespace, NamespaceStack, Name, Attribute, Attributes, Tag, Event};

pub use reader::{StringReader, StringPos, Lexer, Parser, Span};
pub use reader::Error as ReaderError;

/*
mod types;
mod utils;

pub mod writer;
 */
