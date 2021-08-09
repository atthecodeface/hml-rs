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

@file    traits.rs
@brief   Reader traits for the markup library
 */

use super::Span;

//a Position trait
//tt Position
/// The trait required for a position within a stream for a Reader
pub trait Position: Clone + Copy + std::fmt::Debug + std::fmt::Display + 'static {
    /// A position that is unset
    fn none() -> Self;
}

//tt Character
/// The trait required for a character returned by a Reader
///
/// A character can be a char or an end of file marker; it can also
/// (in the future) be a 'not ready' indication - where there data may
/// become ready in a later Reader get character call
pub trait Character: Clone + Copy + std::fmt::Debug + std::fmt::Display + 'static {
    /// Return true if the character is end-of-file
    fn is_eof(&self) -> bool;
    /// Return true if the character is not-ready
    fn is_not_rdy(&self) -> bool;
    /// Return Some(c) if the character corresponds to a real `char`
    fn as_char(&self) -> Option<char>;
}

//tt Error
/// The trait required for the Reader 'Error' type
pub trait Error: std::error::Error + 'static {
    /// The type of a position used by the error for its Span's
    type Position: Position;
    /// Write the error without the span
    fn write_without_span(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result;
    /// Borrow a span if it has one
    fn borrow_span(&self) -> Option<&Span<Self::Position>>;
}

//tt Reader
/// The trait required of a Reader for its use with markup language readers
pub trait Reader: std::fmt::Debug {
    /// The type of the position within a stream for the reader, used by its Error and Span
    type Position: Position;
    /// The type of characters returned by the reader
    type Char: Character;
    /// The type of errors created by the reader
    type Error: Error<Position = Self::Position>;
    /// Return the next character from the stream - this can indicate
    /// end of file or an actual character.
    fn next_char(&mut self) -> std::result::Result<Self::Char, Self::Error>;
    /// Borrow the position of the reader's next character
    fn borrow_pos(&self) -> &Self::Position;
    /// Write with the formatter the context indicated by start and end
    ///
    /// This is used to display errors for users, using the context of
    /// the error provided by its span
    ///
    /// If end == start then the context is a single point in the reader stream
    ///
    /// If the reader has the stream contents for the span from the
    /// two positions then it may output that content
    ///
    /// If the reader does not have any stream contents then this can do nothing
    fn fmt_context(
        &self,
        f: &mut dyn std::fmt::Write,
        start: &Self::Position,
        end: &Self::Position,
    ) -> std::fmt::Result;
}
