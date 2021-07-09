mod traits;
mod markup;

pub use traits::{StreamSpan};

pub use markup::{MarkupError, MarkupResult};
pub use markup::{NSNameId, NSPrefixId, NSUriId, NSMap};
pub use markup::{Namespace, NamespaceStack, Name, Attribute, Attributes, Tag};
pub use markup::{Event, EventType};

// Expose reader::{Position, Character, Reader, Lexer, Parser, Span, Error}
pub mod reader;

mod implementations;
pub use implementations::string;
// pub mod implementations::file;

/*
mod types;
mod utils;

pub mod writer;
 */
