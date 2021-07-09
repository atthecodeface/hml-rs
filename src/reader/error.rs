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
use super::{Reader, Span};

//a Result
//tp Result
/// The [Result] type is a result with an error type of [crate::Error]
pub type Result<R, T> = std::result::Result<T, Error<R>>;

//a Error
//tp Error
/// [Error] represents an error from the UTF-8 character reader,
/// either an IO error from the reader or a malformed UTF-8 encoded
/// set of bytes.
#[derive(Debug)]
pub enum Error<R:Reader> {
    /// A UTF8 error
    Utf8Error(R::Position, R::Error),
    MarkupError(Span<R::Position>, MarkupError),
    UnexpectedCharacter(Span<R::Position>, char),
    /// Expected a depth of N or N+1
    UnexpectedTagIndent(Span<R::Position>, usize),
    BeyondEndOfTokens,
    UnexpectedAttribute(Span<R::Position>, String),
    UnexpectedEOF(Span<R::Position>),
}

//ip Error
impl <R:Reader> Error<R> {
    pub fn of_reader<T>(reader:&R, reader_error:R::Error) -> Result<R, T> {
        let posn = reader.borrow_pos();
        Err(Self::Utf8Error(*posn, reader_error))
    }
    pub fn unexpected_eof<T>(start:&R::Position, end:&R::Position) -> Result<R, T> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedEOF(span))
    }
    pub fn unexpected_character<T>(start:&R::Position, end:&R::Position, ch:char) -> Result<R, T> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedCharacter(span, ch))
    }
    pub fn no_more_events<T>() -> Result<R, T> {
        Err(Self::BeyondEndOfTokens)
    }
    pub fn unexpected_tag_indent<T>(span:Span<R::Position>, depth:usize) -> Result<R, T> {
        Err(Self::UnexpectedTagIndent(span, depth))
    }
    pub fn unexpected_attribute<T>(span:Span<R::Position>, prefx:&str, name:&str) -> Result<R, T> {
        let name = format!("{}:{}", prefx, name);
        Err(Self::UnexpectedAttribute(span, name))
    }
    pub fn of_markup_error(span:Span<R::Position>, e:MarkupError) -> Self {
        Self::MarkupError(span, e)
    }
    pub fn of_markup_result<T>(span:Span<R::Position>, r:MarkupResult<T>) -> Result<R, T> {
        match r {
            Ok(t) => Ok(t),
            Err(e) => Err(Self::of_markup_error(span, e)),
        }
    }
}

//ip std::fmt::Display for Error
impl <R:Reader> std::fmt::Display for Error<R> {
    //mp fmt - format a `Error` for display
    /// Display the `Error` in a human-readable form
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Utf8Error(posn, err) => write!(f, "{}: {}", posn, err),
            Error::UnexpectedTagIndent(span, depth) => write!(f, "{}: Expected a tag indent of at most {}", span, depth),
            _ => Ok(()),
        }
    }
}

//ip std::error::Error for Error
impl <R:Reader> std::error::Error for Error<R> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Utf8Error(_,e) => Some(e),
            _ => None,
        }
    }
}
