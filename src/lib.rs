pub mod names;
pub mod markup;

// pub use names::{NSNameId, NSPrefixId, NSUriId, NSMap};
// pub use names::{Namespace, NamespaceStack, Name, Attribute, Attributes, Tag};

// pub use markup::Span as StreamSpan;
// pub use markup::{MarkupError, MarkupResult};
// pub use markup::{Event, EventType};

// Expose reader::{Position, Character, Error, Reader, Lexer, Parser, Span, ReaderError, Result}
pub mod reader;

mod implementations;
pub use implementations::string;

/*
mod types;
mod utils;

pub mod writer;
 */
