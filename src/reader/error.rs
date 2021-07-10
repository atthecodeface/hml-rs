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

@file    types.rs
@brief   Types used throughout the reader files
 */

//a Imports
use crate::{MarkupError, MarkupResult};
use super::{Reader, Position, Span, Error};

//a Result
//tp Result
/// The [Result] type is a result with an error type of [crate::Error]
// pub type Result<T, P:Position, E:std::error::Error +'static> = std::result::Result<T, Error<P, E>>;
pub type Result<T, P, E> = std::result::Result<T, ReaderError<P, E>>;

//a ReaderError
//tp ReaderError
/// [ReaderError] represents an error from the UTF-8 character reader,
/// either an IO error from the reader or a malformed UTF-8 encoded
/// set of bytes.
#[derive(Debug)]
pub enum ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    /// An error from the underlying reader
    ReaderError(Span<P>, E),
    /// A markup error
    MarkupError(Span<P>, MarkupError),
    /// An unexpected character
    UnexpectedCharacter(Span<P>, char),
    /// Expected a depth of N or N+1
    UnexpectedTagIndent(Span<P>, usize),
    /// Iterated beyond the end of the reader stream
    BeyondEndOfTokens,
    /// Attribute provided where an attribute was not expected
    UnexpectedAttribute(Span<P>, String),
    /// Newline in a quoted string
    UnexpectedNewlineInQuotedString(Span<P>),
    /// Expected an '=' for an attribute but got something else
    ExpectedEquals(Span<P>, char),
    /// EOF when it was not expected
    UnexpectedEOF(Span<P>),
}

//ip ReaderError
impl <P, E> ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    pub fn of_reader<T, R>(reader:&R, reader_error:E) -> Result<T, P, E>
    where R:Reader<Position = P, Error = E>
    {
        let span = Span::new_at(reader.borrow_pos());
        Err(Self::ReaderError(span, reader_error))
    }
    pub fn unexpected_eof<T>(start:&P, end:&P) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedEOF(span))
    }
    pub fn unexpected_character<T>(start:&P, end:&P, ch:char) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedCharacter(span, ch))
    }
    pub fn unexpected_newline_in_string<T>(start:&P, end:&P) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedNewlineInQuotedString(span))
    }
    pub fn expected_equals<T>(start:&P, end:&P, ch:char) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::ExpectedEquals(span, ch))
    }
    pub fn no_more_events<T>() -> Result<T, P, E>{
        Err(Self::BeyondEndOfTokens)
    }
    pub fn unexpected_tag_indent<T>(span:Span<P>, depth:usize) -> Result<T, P, E> {
        Err(Self::UnexpectedTagIndent(span, depth))
    }
    pub fn unexpected_attribute<T>(span:Span<P>, prefx:&str, name:&str) -> Result<T, P, E> {
        let name = format!("{}:{}", prefx, name);
        Err(Self::UnexpectedAttribute(span, name))
    }
    pub fn of_markup_error(span:Span<P>, e:MarkupError) -> Self {
        Self::MarkupError(span, e)
    }
    pub fn of_markup_result<T>(span:Span<P>, r:MarkupResult<T>) -> Result<T, P, E> {
        match r {
            Ok(t) => Ok(t),
            Err(e) => Err(Self::of_markup_error(span, e)),
        }
    }
}

impl <P, E> Error for ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    type Position = P;
    /// Write the error without the span
    fn write_without_span(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::ReaderError(_, err) => err.write_without_span(f),
            Self::MarkupError(_, err) => write!(f, "{}", err),
            Self::UnexpectedCharacter(_, ch)    => write!(f, "Unexpected character '{}'", ch),
            Self::UnexpectedTagIndent(_, depth) => write!(f, "Expected a tag indent of at most {}", depth),
            Self::UnexpectedAttribute(_, name)  => write!(f, "Found attribute when not expected {}", name),
            Self::UnexpectedEOF(_) => write!(f, "Unexpected end-of-file"),
            Self::UnexpectedNewlineInQuotedString(_) => write!(f, "Unexpected newline in quoted string"),
            Self::ExpectedEquals(_, ch) => write!(f,"Expected '=' but found '{}'", ch),
            Self::BeyondEndOfTokens => write!(f, "Attempt to parse beyond end of tokens, probably a bug"),
        }
    }
    /// Borrow a span if it has one
    fn borrow_span(&self) -> Option<&Span<Self::Position>> {
        match self {
            Self::ReaderError(span, e) => {
                e.borrow_span().or(Some(span))
            },
            Self::MarkupError(span, _) => Some(span),
            Self::UnexpectedCharacter(span, _) => Some(span),
            Self::UnexpectedTagIndent(span, _) => Some(span),
            Self::UnexpectedAttribute(span, _) => Some(span),
            Self::UnexpectedEOF(span) => Some(span),
            Self::UnexpectedNewlineInQuotedString(span) => Some(span),
            Self::ExpectedEquals(span, _) => Some(span),
            Self::BeyondEndOfTokens => None,
        }
    }
}

//ip std::fmt::Display for ReaderError
impl <P, E> std::fmt::Display for ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    //mp fmt - format a `Error` for display
    /// Display the `Error` in a human-readable form
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.write_without_span(f)?;
        if let Some(span) = self.borrow_span() {
            write!(f, " at {}", span)
        } else {
            Ok(())
        }
    }
}

//ip std::error::Error for ReaderError
impl <P, E> std::error::Error for ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReaderError(_,e) => Some(e),
            _ => None,
        }
    }
}
