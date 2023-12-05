//a Imports
use crate::{Posn, Span};
use thiserror::Error;

//a Result
//tp Result
/// The [Result] type is a result with an error type of [crate::Error]
// pub type Result<T, P:Position, E:std::error::Error +'static> = std::result::Result<T, Error<P, E>>;
pub type HmlResult<T, P> = std::result::Result<T, HmlError<P>>;
pub type MarkupResult<T> = std::result::Result<T, MarkupError>;

//a MarkupError
//tp MarkupError
#[derive(Debug, Error)]
pub enum MarkupError {
    /// An empty name was provided, which
    /// is illegal
    #[error("empty name")]
    EmptyName {},
    /// Use of an unmapped prefix / namespace
    #[error("unmapped_prefix {prefix}")]
    UnmappedPrefix {
        /// Prefix
        prefix: String,
    },
    /// Indicates a bad name (such as a:b:c)
    #[error("bad name {name}")]
    BadName {
        /// Name
        name: String,
    },
}
impl MarkupError {
    pub fn empty_name() -> Self {
        Self::EmptyName {}
    }
    pub fn unmapped_prefix(prefix: &str) -> Self {
        Self::UnmappedPrefix {
            prefix: prefix.to_string(),
        }
    }
    pub fn bad_name(name: &str) -> Self {
        Self::BadName {
            name: name.to_string(),
        }
    }
}

//a HmlError
//tp HmlError
/// [HmlError] represents an error from the UTF-8 character reader,
/// either an IO error from the reader or a malformed UTF-8 encoded
/// set of bytes.
#[derive(Debug, Error)]
pub enum HmlError<P>
where
    P: Posn,
{
    /// An IO error
    #[error("io error {source}")]
    IoError {
        /// Span of the error
        span: Span<P>,
        /// Error
        source: std::io::Error,
    },
    /// A markup error
    #[error("markup error {source}")]
    MarkupError {
        /// Span of the error
        span: Span<P>,
        /// Error
        source: MarkupError,
    },
    /// Expected a tag name after hashes
    #[error("Expected a tag name after hashes")]
    ExpectedTagName {
        /// Span of the error
        span: Span<P>,
    },
    /// Expect whitespace after a tag
    #[error("Expected whitespace after tag")]
    ExpectedWhitespaceAfterTag {
        /// Span of the error
        span: Span<P>,
    },
    /// An unexpected character
    #[error("Unexpected character '{ch}'")]
    UnexpectedCharacter {
        /// Span of the error
        span: Span<P>,
        /// Character
        ch: char,
    },
    /// Expected a depth of N or N+1
    #[error("Expected a tag indent of at most {depth}")]
    UnexpectedTagIndent {
        /// Span of the error
        span: Span<P>,
        /// Depth
        depth: usize,
    },
    /// Iterated beyond the end of the reader stream
    #[error("Attempt to parse beyond end of tokens, probably a bug")]
    BeyondEndOfTokens,
    /// Attribute provided where an attribute was not expected
    #[error("Found attribute when not expected {attr}")]
    UnexpectedAttribute {
        /// Span of the error
        span: Span<P>,
        /// Attribute
        attr: String,
    },
    /// Newline in a quoted string
    #[error("Unexpected newline in quoted string")]
    UnexpectedNewlineInQuotedString {
        /// Span of the error
        span: Span<P>,
    },
    /// Expected an '=' for an attribute but got something else
    #[error("Expected equals, but found '{ch}'")]
    ExpectedEquals {
        /// Span of the error
        span: Span<P>,
        /// Character
        ch: char,
    },
    /// EOF when it was not expected
    #[error("Unexpected EOF")]
    UnexpectedEOF {
        /// Span of the error
        span: Span<P>,
    },
}

//ip HmlError
impl<P> HmlError<P>
where
    P: Posn,
{
    //fp unexpected_eof
    /// Return an unexpected_eof error at the specified positions
    pub fn unexpected_eof<T>(start: &P, end: &P) -> HmlResult<T, P> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedEOF { span })
    }

    //fp unexpected_character
    /// Return an unexpected_character error at the specified positions
    pub fn unexpected_character<T>(start: &P, end: &P, ch: char) -> HmlResult<T, P> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedCharacter { span, ch })
    }

    //fp unexpected_newline_in_string
    /// Return an unexpected newline error
    pub fn unexpected_newline_in_string<T>(start: &P, end: &P) -> HmlResult<T, P> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::UnexpectedNewlineInQuotedString { span })
    }

    //fp expected_equals
    /// Return an error indicating an expected character, but got a different character
    pub fn expected_equals<T>(start: &P, end: &P, ch: char) -> HmlResult<T, P> {
        let span = Span::new_at(start).end_at(end);
        Err(Self::ExpectedEquals { span, ch })
    }

    //fp no_more_events
    /// Return an error indicating a read beyond the end of the stream
    pub fn no_more_events<T>() -> HmlResult<T, P> {
        Err(Self::BeyondEndOfTokens)
    }

    //fp unexpected_tag_indent
    /// Return an unexpected_tag_indent error over the specified span
    pub fn unexpected_tag_indent<T>(span: Span<P>, depth: usize) -> HmlResult<T, P> {
        Err(Self::UnexpectedTagIndent { span, depth })
    }

    //fp unexpected_attribute
    /// Return an unexpected_attribute eror over the given span
    pub fn unexpected_attribute<T>(span: Span<P>, prefx: &str, name: &str) -> HmlResult<T, P> {
        let attr = format!("{}:{}", prefx, name);
        Err(Self::UnexpectedAttribute { span, attr })
    }

    //fp io_error
    /// An IO error
    pub fn io_error(span: Span<P>, source: std::io::Error) -> Self {
        Self::IoError { span, source }
    }

    //fp of_io_result
    /// Map a io result over a span to a HmlError result
    pub fn of_io_result<T>(
        span: Span<P>,
        r: std::result::Result<T, std::io::Error>,
    ) -> HmlResult<T, P> {
        match r {
            Ok(t) => Ok(t),
            Err(e) => Err(Self::io_error(span, e)),
        }
    }
}

//ip HmlError<P>
impl<P> HmlError<P>
where
    P: Posn,
{
    /// Borrow a span if it has one
    pub fn borrow_span(&self) -> Option<&Span<P>> {
        match self {
            Self::IoError { span, .. } => Some(span),
            Self::UnexpectedCharacter { span, .. } => Some(span),
            Self::UnexpectedTagIndent { span, .. } => Some(span),
            Self::UnexpectedAttribute { span, .. } => Some(span),
            Self::UnexpectedEOF { span, .. } => Some(span),
            Self::UnexpectedNewlineInQuotedString { span, .. } => Some(span),
            Self::ExpectedEquals { span, .. } => Some(span),
            Self::BeyondEndOfTokens => None,
            _ => None,
        }
    }
    #[inline(always)]
    pub fn map_markup_error<T>(result: MarkupResult<T>, span: &Span<P>) -> HmlResult<T, P> {
        match result {
            Err(source) => Err(Self::MarkupError {
                source,
                span: *span,
            }),
            Ok(t) => Ok(t),
        }
    }
}

//ip HmlError<P>
impl<P> lexer_rs::LexerError<P> for HmlError<P>
where
    P: Posn,
{
    fn failed_to_parse(posn: P, ch: char) -> Self {
        let span = Span::new_at(&posn);
        Self::UnexpectedCharacter { span, ch }
    }
}
