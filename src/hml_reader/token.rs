//a Imports
use std::collections::VecDeque;

use crate::{Posn, Span};

//a Token
//tp Token
/// [Token] represents a single item in an HML document
/// This will be an entity that effects the parse state of the parser
/// Hence it includes all of attr="string with spaces"
///
/// Missing are whether characters is escapable or not
///
/// and processing instruction
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    /// ; stuff up to newline
    Comment,
    /// ###<tag>[{] Tag open - with depth (number of #) and true if boxed
    TagOpen,
    /// ###<tag>} Tag close - with depth (number of #)
    TagClose,
    /// attribute [<string>:]<string>=<quoted string>
    Attribute,
    /// Quoted string of raw characters
    RawCharacters,
    /// Quoted string which needs interpretation (escapes handled)
    Characters,
    /// End of file
    EndOfFile,
    /// Whitespace that should in general be ignored
    Whitespace,
}

#[derive(Debug, Clone)]
pub struct Token<P>
where
    P: Posn,
{
    span: Span<P>,
    tt: TokenType,
    contents: VecDeque<String>,
    depth: usize,
    boxed: bool,
}

//ip Token
impl<P> Token<P>
where
    P: Posn,
{
    //fi new
    fn new(span: Span<P>, tt: TokenType, depth: usize, boxed: bool) -> Self {
        let contents = VecDeque::new();
        Self {
            span,
            tt,
            contents,
            depth,
            boxed,
        }
    }

    //cp add_string
    pub fn add_string(mut self, s: String) -> Self {
        self.contents.push_back(s);
        self
    }

    //fp open_boxed
    pub fn open_boxed(span: Span<P>, ns: String, name: String, depth: usize) -> Self {
        Self::new(span, TokenType::TagOpen, depth, true)
            .add_string(ns)
            .add_string(name)
    }

    //fp open
    pub fn open(span: Span<P>, ns: String, name: String, depth: usize) -> Self {
        Self::new(span, TokenType::TagOpen, depth, false)
            .add_string(ns)
            .add_string(name)
    }

    //fp close
    pub fn close(span: Span<P>, ns: String, name: String, depth: usize) -> Self {
        Self::new(span, TokenType::TagClose, depth, false)
            .add_string(ns)
            .add_string(name)
    }

    //fp attribute
    pub fn attribute(span: Span<P>, ns: String, name: String, value: String) -> Self {
        Self::new(span, TokenType::Attribute, 0, false)
            .add_string(ns)
            .add_string(name)
            .add_string(value)
    }

    //fp comment
    /// Consumes the Vec<String>
    pub fn comment(span: Span<P>, strings: Vec<String>) -> Self {
        let mut t = Self::new(span, TokenType::Comment, 0, false);
        for s in strings {
            t = t.add_string(s);
        }
        t
    }

    //fp whitespace
    /// Whitespace that should in general be ignored
    pub fn whitespace(span: Span<P>) -> Self {
        Self::new(span, TokenType::Whitespace, 0, false)
    }

    //fp raw_characters
    pub fn raw_characters(span: Span<P>, s: String) -> Self {
        Self::new(span, TokenType::RawCharacters, 0, false).add_string(s)
    }

    //fp characters
    pub fn characters(span: Span<P>, s: String) -> Self {
        Self::new(span, TokenType::Characters, 0, false).add_string(s)
    }

    //fp eof
    pub fn eof(span: Span<P>) -> Self {
        Self::new(span, TokenType::EndOfFile, 0, false)
    }

    //mp token_type
    pub fn token_type(&self) -> TokenType {
        self.tt
    }

    //mp get_span
    pub fn get_span(&self) -> &Span<P> {
        &self.span
    }

    //mp get_depth
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    //mp get_boxed
    pub fn get_boxed(&self) -> bool {
        self.boxed
    }

    //mp contents
    pub fn contents(&self) -> &[String] {
        self.contents.as_slices().0
    }

    //mp take_contents
    pub fn take_contents(&mut self) -> VecDeque<String> {
        self.contents.split_off(0)
    }

    //mp is_eof
    pub fn is_eof(&self) -> bool {
        self.tt == TokenType::EndOfFile
    }

    //mp is_attribute
    pub fn is_attribute(&self) -> bool {
        self.tt == TokenType::Attribute
    }

    //mp is_whitespace
    pub fn is_whitespace(&self) -> bool {
        self.tt == TokenType::Whitespace
    }
}

//ip std::fmt::Display for Token
impl<P> std::fmt::Display for Token<P>
where
    P: Posn,
{
    //mp fmt - format a `Token` for display
    /// Display the `Token` in a human-readable form
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use TokenType::*;
        match self.tt {
            Comment => write!(f, "[{:?}]; ...", self.span),
            TagOpen => {
                if self.boxed {
                    write!(
                        f,
                        "[{:?}]#<{}>{}:{}{{",
                        self.span, self.depth, self.contents[0], self.contents[1]
                    )
                } else {
                    write!(
                        f,
                        "[{:?}]#<{}>{}:{}",
                        self.span, self.depth, self.contents[0], self.contents[1]
                    )
                }
            }
            TagClose => {
                write!(
                    f,
                    "[{:?}]#<{}>{}:{}}}",
                    self.span, self.depth, self.contents[0], self.contents[1]
                )
            }
            Attribute => {
                write!(
                    f,
                    "[{:?}]{}:{}={}",
                    self.span, self.contents[0], self.contents[1], self.contents[2]
                )
            }
            Characters => {
                write!(f, "[{:?}]chars ...", self.span)
            }
            RawCharacters => {
                write!(f, "[{:?}]rawchars ...", self.span)
            }
            Whitespace => {
                write!(f, "[{:?}]whitespace", self.span)
            }
            EndOfFile => write!(f, "[{:?}]<eof>", self.span),
        }
    }
}
