//a Documentation
//! This module provides a tokenizer for HML documents. HML documents are UTF-8 encoded.
//!
//! An HML token can be a tag, such as `###banana`, which may be
//! an explicit boxing open tag e.g. `##fruit{` or the equivalent close
//! tag `##fruit}`
//!
//! The token may be a comment - any HMLH line whose first non-whitespace character is a semicolon
//! makes the rest of the line after the semicolon a comment
//!
//! Note that XML does not support comments containing '--', nor those ending with a single '-'.
//!
//! quoted strings - want " -> ", r" -> ", ###" -> "###, and r####" -> "####; no newlines if no #; raw means don't unescape
//!
//! QuotedString :=   '"'     <EscapedContent without newlines> '"'
//!               |  'r"'     <Content without newlines> '"'
//!               |  '#"{M}'  <EscapedContent> '"{M}#'
//!               |  'r#"{M}' <Content> '"{M}#'
//!
//!
//! whitespace is a quoted string that is just whitespace
//!
//! The token may be characters - a quoted string - which starts with either a single or double quote character.
//! Quoted strings using one quote character to delineate it, in which case the contents are escaped, and must not contain newlines
//! Alternatively quoted strings may start with three quote characters, in which case they can be boxed, and the terminate at the
//! next occurrence of the same three quote characters
//!
//! A token may be a attribute - which is of the form [<name_space>:]<name>=<quoted string>
//!
//! A token may be a processing instruction introduction, e.g. '###?name'. It cannot be boxed (as PI have no content)
//!

//a Imports
use super::utils::*;
use super::Token;
use crate::reader;
use crate::reader::Reader;

//a Result
//ti Result
type Result<R, T> = reader::Result<T, <R as Reader>::Position, <R as Reader>::Error>;

//a Lexer
/// `Lexer` is a tokenizer for HMLH documents
///
/// Main method is `next_token` which accepts an `Read` instance
///
//tp Lexer
pub struct Lexer<R: Reader> {
    read_ahead: Option<Option<char>>,
    token_start: R::Position,
}

//ip Default for Lexer
impl<R: Reader> Default for Lexer<R> {
    fn default() -> Self {
        Lexer {
            read_ahead: None,
            token_start: R::Position::default(),
        }
    }
}

//ip Lexer
impl<R: Reader> Lexer<R> {
    //mi peek_char - peek at the next character
    /// Peek character
    fn peek_char(&mut self, reader: &mut R) -> Result<R, Option<char>> {
        match self.read_ahead {
            Some(x) => Ok(x),
            None => match reader.next_char() {
                Ok(ch) => {
                    self.read_ahead = Some(ch);
                    Ok(ch)
                }
                Err(error) => {
                    let span = crate::reader::Span::new_at(reader.borrow_pos());
                    Err(crate::reader::ReaderError::ReaderError { span, error })
                }
            },
        }
    }

    //mi peek_char_no_eof - peek at the next character, with an error if it is EOF
    /// Peek character - EOF not permitted
    fn peek_char_no_eof(&mut self, reader: &mut R) -> Result<R, char> {
        let ch = self.peek_char(reader)?;
        if let Some(ch) = ch {
            Ok(ch)
        } else {
            // assume eof for now
            self.unexpected_eof(reader)
        }
    }

    //mi drop_peek
    /// Drop the peeked character
    fn drop_peek(&mut self) {
        assert!(self.read_ahead.is_some());
        self.read_ahead = None;
    }

    //mi get_char - get the next character
    /// Get character
    fn get_char(&mut self, reader: &mut R) -> Result<R, Option<char>> {
        match self.read_ahead {
            Some(x) => {
                self.read_ahead = None;
                Ok(x)
            }
            None => match reader.next_char() {
                Ok(ch) => Ok(ch),
                Err(error) => {
                    let span = crate::reader::Span::new_at(reader.borrow_pos());
                    Err(crate::reader::ReaderError::ReaderError { span, error })
                }
            },
        }
    }

    //mi get_char_no_eof - get the next character, with an error if it is EOF
    /// Get character - EOF not permitted
    fn get_char_no_eof(&mut self, reader: &mut R) -> Result<R, char> {
        let ch = self.get_char(reader)?;
        if let Some(ch) = ch {
            Ok(ch)
        } else {
            // assume eof for now
            self.unexpected_eof(reader)
        }
    }

    //mi unget_char - return a character to the (single char) readahead buffer
    /// Unget a character - put it into the readahead
    fn unget_char(&mut self, char: Option<char>) {
        self.read_ahead = Some(char);
    }

    //mi skip_whitespace - get up to first non-whitespace character
    /// Read characters until EOF or non-whitespace
    /// If non-whitespace, then unget it back into the readahead
    fn skip_whitespace(&mut self, reader: &mut R, include_nl: bool) -> Result<R, ()> {
        loop {
            let ch = self.get_char(reader)?;
            if let Some(c) = ch {
                if !c.is_whitespace() {
                    self.unget_char(ch);
                    break;
                }
                if c == '\n' && !include_nl {
                    self.unget_char(ch);
                    break;
                }
            } else {
                self.unget_char(ch);
                break;
            }
        }
        Ok(())
    }

    //mi read_line - read up to newline, for (e.g.) comments
    /// Read the string from the current char to a newline, leaving that out
    fn read_line(&mut self, reader: &mut R) -> Result<R, String> {
        let mut s = String::new();
        loop {
            let ch = self.get_char(reader)?;
            if let Some(c) = ch {
                if is_newline(c) {
                    break;
                }
                s.push(c);
            } else {
                self.unget_char(ch);
                break;
            }
        }
        Ok(s)
    }

    //mp next_token
    /// Tries to read the next token from the buffer, returning an Ok(Token) on success
    ///
    /// # Errors
    ///
    /// Can return an IO error from the underlying stream, or a UTF-8 encoding error.
    ///
    /// Additionally it may return an error for characters that are illegal within the token stream
    pub fn next_token(&mut self, reader: &mut R) -> Result<R, Token<R::Position>> {
        self.skip_whitespace(reader, true)?;
        self.token_start = *reader.borrow_pos();
        let ch = self.peek_char(reader)?;
        if let Some(ch) = ch {
            let mut span = reader::Span::new_at(reader.borrow_pos());
            if ch == ';' {
                self.get_char(reader)?; // drop the semicolon
                let mut comment_strings = Vec::new();
                loop {
                    comment_strings.push(self.read_line(reader)?);
                    span = span.end_at(reader.borrow_pos());
                    self.skip_whitespace(reader, false)?;
                    if self.peek_char(reader)? != Some(';') {
                        break;
                    }
                    self.get_char(reader)?;
                }
                return Ok(Token::comment(span, comment_strings));
            } else if is_hash(ch) {
                let hash_count = self.read_hash_sequence(reader)?;
                let ch = self.peek_char(reader)?.unwrap();
                if is_quote(ch) {
                    self.drop_peek();
                    let s = self.read_quoted_string(reader, ch, hash_count, false)?;
                    span = span.end_at(reader.borrow_pos());
                    return Ok(Token::characters(span, s));
                } else {
                    return self.read_tag(reader, span, hash_count);
                }
            } else if is_quote(ch) {
                self.drop_peek();
                let s = self.read_quoted_string(reader, ch, 0, false)?;
                span = span.end_at(reader.borrow_pos());
                return Ok(Token::characters(span, s));
            } else if ch == 'r' {
                self.drop_peek();
                if let Some(ch) = self.peek_char(reader)? {
                    if is_hash(ch) {
                        let hash_count = self.read_hash_sequence(reader)?;
                        let ch = self.peek_char(reader)?.unwrap();
                        if is_quote(ch) {
                            self.drop_peek();
                            let s = self.read_quoted_string(reader, ch, hash_count, true)?;
                            span = span.end_at(reader.borrow_pos());
                            return Ok(Token::raw_characters(span, s));
                        } else {
                            return self.unexpected_character(reader, ch);
                        }
                    } else if is_quote(ch) {
                        self.drop_peek();
                        let s = self.read_quoted_string(reader, ch, 0, true)?;
                        span = span.end_at(reader.borrow_pos());
                        return Ok(Token::raw_characters(span, s));
                    }
                }
                return self.read_attribute(reader, span, Some('r'));
            } else if is_name_start(ch) {
                return self.read_attribute(reader, span, None);
            }
            self.unexpected_character(reader, ch)
        } else {
            Ok(Token::eof(reader::Span::new_at(reader.borrow_pos())))
        }
    }

    //mi unexpected_eof
    fn unexpected_eof<T>(&self, reader: &R) -> Result<R, T> {
        reader::ReaderError::unexpected_eof(&self.token_start, reader.borrow_pos())
    }

    //mi unexpected_character
    fn unexpected_character<T>(&self, reader: &R, ch: char) -> Result<R, T> {
        reader::ReaderError::unexpected_character(&self.token_start, reader.borrow_pos(), ch)
    }

    //mi unexpected_newline_in_string
    fn unexpected_newline_in_string<T>(&self, reader: &R) -> Result<R, T> {
        reader::ReaderError::unexpected_newline_in_string(&self.token_start, reader.borrow_pos())
    }

    //mi expected_equals
    fn expected_equals<T>(&self, reader: &R, ch: char) -> Result<R, T> {
        reader::ReaderError::expected_equals(&self.token_start, reader.borrow_pos(), ch)
    }

    //mi read_name - read a name, cursor should be pointing at a is_name_start character
    // at end, cursor pointing at first non-name character or EOF
    fn read_name(&mut self, reader: &mut R, initial_ch: Option<char>) -> Result<R, String> {
        let mut s = String::new();
        let ch = {
            if let Some(ch) = initial_ch {
                ch
            } else {
                self.get_char_no_eof(reader)?
            }
        };
        if !is_name_start(ch) {
            return self.unexpected_character(reader, ch);
        }
        s.push(ch);
        loop {
            let ch = self.get_char(reader)?;
            match ch {
                Some(c) if is_name(c) => {
                    s.push(c);
                }
                _ => {
                    self.unget_char(ch);
                    break;
                }
            }
        }
        self.token_start = *reader.borrow_pos();
        Ok(s)
    }

    //mi read_namespace_name
    /// initial_ch is Some(first character) or None if reader pointing
    /// at first character of name
    fn read_namespace_name(
        &mut self,
        reader: &mut R,
        initial_ch: Option<char>,
    ) -> Result<R, (String, String)> {
        let name = self.read_name(reader, initial_ch)?;
        let ch = self.peek_char(reader)?;
        match ch {
            Some(':') => {
                self.drop_peek();
                let name2 = self.read_name(reader, None)?;
                Ok((name, name2))
            }
            _ => Ok(("".into(), name)),
        }
    }

    //mi read_hash_sequence - read a sequence of # characters and return its length
    fn read_hash_sequence(&mut self, reader: &mut R) -> Result<R, usize> {
        let mut hash_count = 0;
        loop {
            let ch = self.peek_char_no_eof(reader)?;
            if !is_hash(ch) {
                break;
            }
            hash_count += 1;
            self.drop_peek();
        }
        Ok(hash_count)
    }

    //mi read_tag - read a tag given cursor is at first #
    /// the stream cursor points at the first # in the tag,
    /// and this method reads the tag from that point
    ///
    /// a tag is #+ <namespace_name> [ { | } ] <whitespace>
    ///
    /// The result is a TagOpen or TagClose, with the depth set to the number of '#'s
    /// at the front of the tag, and the namespace_name set appropriately
    fn read_tag(
        &mut self,
        reader: &mut R,
        mut span: reader::Span<R::Position>,
        hash_count: usize,
    ) -> Result<R, Token<R::Position>> {
        let (ns, name) = self.read_namespace_name(reader, None)?;
        let result = {
            match self.peek_char(reader)? {
                Some('{') => {
                    self.drop_peek();
                    span = span.end_at(reader.borrow_pos());
                    Token::open_boxed(span, ns, name, hash_count)
                }
                Some('}') => {
                    self.drop_peek();
                    span = span.end_at(reader.borrow_pos());
                    Token::close(span, ns, name, hash_count)
                }
                _ => {
                    span = span.end_at(reader.borrow_pos());
                    Token::open(span, ns, name, hash_count)
                }
            }
        };
        match self.peek_char(reader)? {
            Some(ch) => {
                if ch.is_whitespace() {
                    Ok(result)
                } else {
                    self.unexpected_character(reader, ch)
                }
            }
            _ => Ok(result),
        }
    }

    //mi read_string
    /// Reads a string, possibly a quoted string, given the stream cursor is pointing at the opening character.
    ///
    /// The string must start with a quote character or a different non-whitespace character
    /// If it starts with a non-whitespace character then the string goes up to EOF or or whitespace
    /// If it starts with a quote character then it is a quoted string
    pub fn read_string(&mut self, reader: &mut R) -> Result<R, String> {
        let ch = self.peek_char_no_eof(reader)?;
        if is_quote(ch) {
            self.drop_peek();
            self.read_quoted_string(reader, ch, 0, false)
        } else {
            let mut result = String::new();
            loop {
                let ch = self.get_char(reader)?;
                match ch {
                    Some(c) => {
                        if c.is_whitespace() {
                            self.unget_char(ch);
                            break;
                        } else {
                            result.push(c);
                        }
                    }
                    _ => {
                        self.unget_char(ch);
                        break;
                    }
                }
            }
            Ok(result)
        }
    }

    //mi read_quoted_string
    /// reads a quoted string, given the stream cursor is pointing at the opening quote character
    ///
    pub fn read_quoted_string(
        &mut self,
        reader: &mut R,
        quote_ch: char,
        hash_count: usize,
        _raw: bool,
    ) -> Result<R, String> {
        let mut result = String::new();
        let mut ch = self.get_char_no_eof(reader)?;
        loop {
            while ch != quote_ch {
                if is_newline(ch) && hash_count == 0 {
                    return self.unexpected_newline_in_string(reader);
                }
                result.push(ch);
                ch = self.get_char_no_eof(reader)?;
            }
            // ch == quote_ch; check for hashes if required
            let mut i = 0;
            while i < hash_count {
                ch = self.get_char_no_eof(reader)?;
                if ch != '#' {
                    break;
                }
                i += 1;
            }
            if i == hash_count {
                break;
            }
            result.push(quote_ch);
            for _ in 0..i {
                result.push('#');
            }
        }
        Ok(result)
    }

    //mi read_attribute
    // Stream is pointing at first character of attribute
    fn read_attribute(
        &mut self,
        reader: &mut R,
        mut span: reader::Span<R::Position>,
        initial_ch: Option<char>,
    ) -> Result<R, Token<R::Position>> {
        let (ns, name) = self.read_namespace_name(reader, initial_ch)?;
        let ch = self.get_char_no_eof(reader)?;
        if ch != '=' {
            return self.expected_equals(reader, ch);
        }
        let value = self.read_string(reader)?;
        span = span.end_at(reader.borrow_pos());
        Ok(Token::attribute(span, ns, name, value))
    }

    //zz All done
}
