//a Documentation
/*!

# Markup module

This module provides common types for managing markup languages. It
provides a standard error type that utilizes a file/stream [Span]
trait, and then provides markup [Event]s for parsing or writing markup
streams.

!*/

//a Imports
mod error;
mod event;
mod traits;

//a Exports
pub use error::{Error, Result};
pub use event::{ContentType, Event, EventType};
pub use traits::Span;
