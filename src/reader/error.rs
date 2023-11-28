//a Imports
use super::{Position, Reader, Span};
use thiserror::Error;

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
#[derive(Debug, Error)]
pub enum ReaderError<P, E>
where
    P: Position,
    E: std::fmt::Debug,
{
    /// An error from the underlying reader
    #[error("reader error {error:?}")]
    ReaderError { span: Span<P>, error: E },
    /// A markup error
    #[error("markup error {error}")]
    MarkupError {
        span: Span<P>,
        error: crate::markup::Error,
    },
    /// An unexpected character
    #[error("Unexpected character '{ch}'")]
    UnexpectedCharacter { span: Span<P>, ch: char },
    /// Expected a depth of N or N+1
    #[error("Expected a tag indent of at most {depth}")]
    UnexpectedTagIndent { span: Span<P>, depth: usize },
    /// Iterated beyond the end of the reader stream
    #[error("Attempt to parse beyond end of tokens, probably a bug")]
    BeyondEndOfTokens,
    /// Attribute provided where an attribute was not expected
    #[error("Found attribute when not expected {attr}")]
    UnexpectedAttribute { span: Span<P>, attr: String },
    /// Newline in a quoted string
    #[error("Unexpected newline in quoted string")]
    UnexpectedNewlineInQuotedString { span: Span<P> },
    /// Expected an '=' for an attribute but got something else
    #[error("Expected equals, but found '{ch}'")]
    ExpectedEquals { span: Span<P>, ch: char },
    /// EOF when it was not expected
    #[error("Unexpected EOF")]
    UnexpectedEOF { span: Span<P> },
}

//ip ReaderError
impl<P, E> ReaderError<P, E>
where
    P: Position,
    E: std::fmt::Debug,
{
    //fp of_reader
    /// Create a given error with a [Span] of just the current reader
    /// position
    pub fn of_reader<T, R>(reader: &R, reader_error: E) -> Result<T, P, E>
    where
        R: Reader<Position = P, Error = E>,
    {
        let span = Span::new_at(reader.borrow_pos());
        Err(Self::ReaderError {
            span,
            error: reader_error,
        })
    }

    //fp unexpected_eof
    /// Return an unexpected_eof error at the specified positions
    pub fn unexpected_eof<T>(start: &P, end: &P) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedEOF { span })
    }

    //fp unexpected_character
    /// Return an unexpected_character error at the specified positions
    pub fn unexpected_character<T>(start: &P, end: &P, ch: char) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedCharacter { span, ch })
    }

    //fp unexpected_newline_in_string
    /// Return an unexpected newline error
    pub fn unexpected_newline_in_string<T>(start: &P, end: &P) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedNewlineInQuotedString { span })
    }

    //fp expected_equals
    /// Return an error indicating an expected character, but got a different character
    pub fn expected_equals<T>(start: &P, end: &P, ch: char) -> Result<T, P, E> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::ExpectedEquals { span, ch })
    }

    //fp no_more_events
    /// Return an error indicating a read beyond the end of the stream
    pub fn no_more_events<T>() -> Result<T, P, E> {
        Err(Self::BeyondEndOfTokens)
    }

    //fp unexpected_tag_indent
    /// Return an unexpected_tag_indent error over the specified span
    pub fn unexpected_tag_indent<T>(span: Span<P>, depth: usize) -> Result<T, P, E> {
        Err(Self::UnexpectedTagIndent { span, depth })
    }

    //fp unexpected_attribute
    /// Return an unexpected_attribute eror over the given span
    pub fn unexpected_attribute<T>(span: Span<P>, prefx: &str, name: &str) -> Result<T, P, E> {
        let attr = format!("{}:{}", prefx, name);
        Err(Self::UnexpectedAttribute { span, attr })
    }

    //fp of_markup_error
    /// Return a ReaderError of a MarkupError over a certain span
    pub fn of_markup_error(span: Span<P>, error: crate::markup::Error) -> Self {
        Self::MarkupError { span, error }
    }

    //fp of_markup_error
    /// Map a markup result over a span to a ReaderError result
    pub fn of_markup_result<T>(span: Span<P>, r: crate::markup::Result<T>) -> Result<T, P, E> {
        match r {
            Ok(t) => Ok(t),
            Err(e) => Err(Self::of_markup_error(span, e)),
        }
    }
}

impl<P, E> ReaderError<P, E>
where
    P: Position,
    E: std::fmt::Debug,
{
    /// Borrow a span if it has one
    fn borrow_span(&self) -> Option<&Span<P>> {
        match self {
            Self::ReaderError { span, .. } => Some(span),
            Self::MarkupError { span, .. } => Some(span),
            Self::UnexpectedCharacter { span, .. } => Some(span),
            Self::UnexpectedTagIndent { span, .. } => Some(span),
            Self::UnexpectedAttribute { span, .. } => Some(span),
            Self::UnexpectedEOF { span, .. } => Some(span),
            Self::UnexpectedNewlineInQuotedString { span, .. } => Some(span),
            Self::ExpectedEquals { span, .. } => Some(span),
            Self::BeyondEndOfTokens => None,
        }
    }
}
