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
    /// A UTF8 error
    Utf8Error(P, E),
    MarkupError(Span<P>, MarkupError),
    UnexpectedCharacter(Span<P>, char),
    /// Expected a depth of N or N+1
    UnexpectedTagIndent(Span<P>, usize),
    BeyondEndOfTokens,
    UnexpectedAttribute(Span<P>, String),
    UnexpectedEOF(Span<P>),
}

//ip ReaderError
impl <P, E> ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    pub fn of_reader<T, R>(reader:&R, reader_error:E) -> Result<T, P, E>
    where R:Reader<Position = P, Error = E>
    {
        let posn = reader.borrow_pos();
        Err(Self::Utf8Error(*posn, reader_error))
    }
    pub fn unexpected_eof<T>(start:&P, end:&P) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedEOF(span))
    }
    pub fn unexpected_character<T>(start:&P, end:&P, ch:char) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedCharacter(span, ch))
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

//ip std::fmt::Display for ReaderError
impl <P, E> std::fmt::Display for ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    //mp fmt - format a `Error` for display
    /// Display the `Error` in a human-readable form
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Utf8Error(posn, err) => write!(f, "{}: {}", posn, err),
            Self::UnexpectedTagIndent(span, depth) => write!(f, "{}: Expected a tag indent of at most {}", span, depth),
            _ => Ok(()),
        }
    }
}

//ip std::error::Error for ReaderError
impl <P, E> std::error::Error for ReaderError<P, E>
where P:Position, E:Error<Position = P>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Utf8Error(_,e) => Some(e),
            _ => None,
        }
    }
}
