mod error;
mod span;
mod traits;

pub use error::{ReaderError, Result};
pub use span::Span;
pub use traits::{Character, Error, Position, Reader};
