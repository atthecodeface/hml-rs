//a Documentation
/*!

# Reader module

This module provides common types for sources of data for reading markup languages.

A source of data for a reader provides streams of characters, the
position of which within a stream can be determined. Error messages
can by generated by the markup language reader referring to these
positions; a [Span] is a stretch of characters between two such
positions.

!*/

//a Imports
mod error;
mod span;
mod traits;

//a Exports
pub use error::{ReaderError, Result};
pub use span::Span;
pub use traits::{Character, Error, Position, Reader};
