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

@file    markup.rs
@brief   Markup module
 */

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
