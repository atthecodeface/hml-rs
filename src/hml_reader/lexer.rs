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

@file    lexer.rs
@brief   Lexical analyzer creating tokens from utf8
 */

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
//! quoted strings - want " -> ", r" -> ", ###" -> "###, and r####" -> "####; no newlines if no #; raw means don't interpret ampersands nor handle escapes (CDATA)
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
use crate::reader::{Reader, Position, Character};
use crate::reader::{ReaderError, Span};
use super::{Token};
type Result<R, T> = crate::reader::Result<T, <R as Reader>::Position, <R as Reader>::Error>;

//a Utils
//fp is_quote
/// Returns true if the UTF character is either a single or double quote
#[inline]
fn is_newline(ch:char) -> bool { ch == '\n' }
#[inline]
fn is_hash(ch:char) -> bool { ch == '#' }
#[inline]
pub fn is_quote(ch:char) -> bool { ch=='"' || ch=='\'' }

//fp is_name_start
/// Returns true if the UTF character is a colon, underscore, alphabetic, or UTF name character
pub fn is_name_start(ch:char) -> bool {
    let ch = ch as u32;
    match ch {
        // ?? 58 => {true}, // colon
        95 => {true}, // underscore
        _  => { ((ch>=65) && (ch<=90))       ||    // A-Z
                    ((ch>=97) && (ch<=122))     ||   // a-z
                    ((ch>=0xc0) && (ch<=0xd6)) ||
                    ((ch>=0xd8) && (ch<=0xf6)) ||
                    ((ch>=0xf8) && (ch<=0x2ff)) ||
                    ((ch>=0x370) && (ch<=0x37d)) ||
                    ((ch>=0x37f) && (ch<=0x1fff)) ||
                    ((ch>=0x200c) && (ch<=0x200d)) ||
                    ((ch>=0x2070) && (ch<=0x218f)) ||
                    ((ch>=0x2c00) && (ch<=0x2fef)) ||
                    ((ch>=0x3001) && (ch<=0xd7ff)) ||
                    ((ch>=0xf900) && (ch<=0xfdcf)) ||
                    ((ch>=0xfdf0) && (ch<=0xfffd)) ||
                    ((ch>=0x10000) && (ch<=0xeffff))  }
    }
}

//fp is_name
/// Returns true if the UTF character is a name character or a
/// continuation of a name character that adds -, ., digits, and other
/// UTF characters
pub fn is_name(ch:char) -> bool {
    if is_name_start(ch) {
        true
    } else {
        let ch = ch as u32;
        ((ch==45) || (ch==46) || (ch==0xb7)) || // - .
            ((ch>=48) && (ch<=57)) || // 0-9
            ((ch>=0x399) && (ch<=0x36f)) ||
            ((ch>=0x203f) && (ch<=0x2040))
    }
}


//a Lexer
/// `Lexer` is a tokenizer for HMLH documents
///
/// Main method is `next_token` which accepts an `Read` instance
///
//tp Lexer
pub struct Lexer<R:Reader> {
    read_ahead : Option<R::Char>,
    token_start: R::Position,
}

//ip Lexer
impl <R:Reader> Lexer<R> {

    //fp new -
    /// Returns a new lexer with default state.
    pub fn new() -> Self {
        Lexer {
            read_ahead: None,
            token_start: R::Position::none(),
        }
    }

    //mi peek_char - peek at the next character
    /// Peek character
    fn peek_char(&mut self, reader:&mut R) -> Result<R, R::Char> {
        match self.read_ahead {
            Some(x) => {
                Ok(x)
            },
            None => {
                match reader.next_char() {
                    Ok(ch) => {
                        self.read_ahead = Some(ch);
                        Ok(ch)
                    }
                    Err(e) => ReaderError::of_reader(reader, e)
                }
            },
        }
    }

    //mi peek_char_no_eof - peek at the next character, with an error if it is EOF
    /// Peek character - EOF not permitted
    fn peek_char_no_eof(&mut self, reader:&mut R) -> Result<R, char> {
        let ch = self.peek_char(reader)?;
        if let Some(ch) = ch.as_char() {
            Ok(ch)
        } else { // assume eof for now
            self.unexpected_eof(reader)
        }
    }

    //mi get_char - get the next character
    /// Get character
    fn get_char(&mut self, reader:&mut R) -> Result<R, R::Char> {
        match self.read_ahead {
            Some(x) => {
                self.read_ahead = None;
                Ok(x)
            }
            None => {
                match reader.next_char() {
                    Ok(ch) => {
                        Ok(ch)
                    }
                    Err(e) => ReaderError::of_reader(reader, e)
                }
            }
        }
    }

    //mi get_char - get the next character, with an error if it is EOF
    /// Get character - EOF not permitted
    fn get_char_no_eof(&mut self, reader:&mut R) -> Result<R, char> {
        let ch = self.get_char(reader)?;
        if let Some(ch) = ch.as_char() {
            Ok(ch)
        } else { // assume eof for now
            self.unexpected_eof(reader)
        }
    }

    //mi unget_char - return a character to the (single char) readahead buffer
    /// Unget a character - put it into the readahead
    fn unget_char(&mut self, char:R::Char) -> () {
        self.read_ahead = Some(char);
    }

    //mi skip_whitespace - get up to first non-whitespace character
    /// Read characters until EOF or non-whitespace
    /// If non-whitespace, then unget it back into the readahead
    fn skip_whitespace(&mut self, reader:&mut R) -> Result<R, ()> {
        loop {
            let ch = self.get_char(reader)?;
            if let Some(c) = ch.as_char() {
                if !c.is_whitespace() {
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
    fn read_line(&mut self, reader:&mut R) -> Result<R, String> {
        let mut s = String::new();
        loop {
            let ch = self.get_char(reader)?;
            if let Some(c) = ch.as_char() {
                if is_newline(c) {
                    break;
                }
                s.push(c);
            } else {
                self.unget_char(ch);
                break;
            }
        }
        return Ok(s);
    }

    //mp next_token
    /// Tries to read the next token from the buffer, returning an Ok(Token) on success
    ///
    /// # Errors
    ///
    /// Can return an IO error from the underlying stream, or a UTF-8 encoding error.
    ///
    /// Additionally it may return an error for characters that are illegal within the token stream
    pub fn next_token(&mut self, reader:&mut R) -> Result<R, Token<R::Position>> {
        self.skip_whitespace(reader)?;
        self.token_start = *reader.borrow_pos();
        let ch = self.peek_char(reader)?;
        if let Some(ch) = ch.as_char() {
            if ch == ';' {
                let mut span = Span::new_at(reader.borrow_pos());
                self.get_char(reader)?; // drop the semicolon
                let mut comment_strings = Vec::new();
                loop {
                    comment_strings.push(self.read_line(reader)?);
                    span = span.end_at(reader.borrow_pos());
                    self.skip_whitespace(reader)?;
                    if self.peek_char(reader)?.as_char() != Some(';') {
                        break;
                    }
                    self.get_char(reader)?;
                }
                return Ok(Token::comment(span, comment_strings));
            } else if is_hash(ch) {
                return self.read_tag(reader);
            } else if ch == '\'' || ch=='"' {
                let mut span = Span::new_at(reader.borrow_pos());
                let s = self.read_quoted_string(reader)?;
                span = span.end_at(reader.borrow_pos());
                return Ok(Token::characters(span, s));
            } else if is_name_start(ch) {
                return self.read_attribute(reader);
            }
            return self.unexpected_character(reader, ch);
        } else {
            Ok(Token::eof(Span::new_at(reader.borrow_pos())))
        }
    }

    //mi unexpected_eof
    fn unexpected_eof<T> (&self, reader:&R) -> Result<R, T> {
        ReaderError::unexpected_eof(&self.token_start, reader.borrow_pos())
    }

    //mi unexpected_character
    fn unexpected_character<T> (&self, reader:&R, ch:char) -> Result<R, T> {
        ReaderError::unexpected_character(&self.token_start, reader.borrow_pos(), ch)
    }

    //mi unexpected_newline_in_string
    fn unexpected_newline_in_string<T> (&self, reader:&R) -> Result<R, T> {
        ReaderError::unexpected_newline_in_string(&self.token_start, reader.borrow_pos())
    }

    //mi expected_equals
    fn expected_equals<T> (&self, reader:&R, ch:char) -> Result<R, T> {
        ReaderError::expected_equals(&self.token_start, reader.borrow_pos(), ch)
    }

    //mi read_name - read a name, cursor should be pointing at a is_name_start character
    // at end, cursor pointing at first non-name character or EOF
    fn read_name(&mut self, reader:&mut R) -> Result<R, String> {
        let mut s = String::new();
        let ch = self.get_char_no_eof(reader)?;
        if !is_name_start(ch) {
            return self.unexpected_character(reader, ch);
        }
        s.push(ch);
        loop {
            let ch = self.get_char(reader)?;
            match ch.as_char() {
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
        return Ok(s);
    }

    //mi read_namespace_name
    // pointing at first character of name
    fn read_namespace_name(&mut self, reader:&mut R) -> Result<R, (String,String)> {
        let name = self.read_name(reader)?;
        let ch = self.peek_char(reader)?;
        match ch.as_char() {
            Some(':') => {
                self.get_char(reader)?;
                let name2 = self.read_name(reader)?;
                Ok((name, name2))
            },
            _ => {
                Ok(("".into(), name))
            },
        }
    }

    //mi read_tag - read a tag given cursor is at first #
    /// the stream cursor points at the first # in the tag,
    /// and this method reads the tag from that point
    ///
    /// a tag is #+ <namespace_name> [ { | } ] <whitespace>
    ///
    /// The result is a TagOpen or TagClose, with the depth set to the number of '#'s
    /// at the front of the tag, and the namespace_name set appropriately
    fn read_tag(&mut self, reader:&mut R) -> Result<R, Token<R::Position>> {
        let mut span = Span::new_at(reader.borrow_pos());
        let mut hash_count = 0;
        loop {
            let ch = self.peek_char_no_eof(reader)?;
            if !is_hash(ch) { break; }
            hash_count += 1;
            self.get_char(reader)?;
        }
        let (ns, name) = self.read_namespace_name(reader)?;
        let result = {
            match self.peek_char(reader)?.as_char() {
                Some('{') => {
                    self.get_char(reader)?;
                    span = span.end_at(reader.borrow_pos());
                    Token::open_boxed(span, ns, name, hash_count)
                },
                Some('}') => {
                    self.get_char(reader)?;
                    span = span.end_at(reader.borrow_pos());
                    Token::close(span, ns, name, hash_count)
                },
                _ => {
                    span = span.end_at(reader.borrow_pos());
                    Token::open(span, ns, name, hash_count)
                },
            }
        };
        match self.peek_char(reader)?.as_char() {
            Some(ch) => {
                if ch.is_whitespace() {
                    Ok(result)
                } else {
                    self.unexpected_character(reader, ch)
                }
            },
            _ => Ok(result),
        }
    }

    //mi read_string
    /// Reads a string, possibly a quoted string, given the stream cursor is pointing at the opening character.
    ///
    /// The string must start with a quote character or a different non-whitespace character
    /// If it starts with a non-whitespace character then the string goes up to EOF or or whitespace
    /// If it starts with a quote character then it is a quoted string
    pub fn read_string(&mut self, reader:&mut R) -> Result<R, String> {
        let ch = self.peek_char_no_eof(reader)?;
        if is_quote(ch) {
            self.read_quoted_string(reader)
        } else {
            let mut result = String::new();
            loop {
                let ch = self.get_char(reader)?;
                match ch.as_char() {
                    Some(c) => {
                        if c.is_whitespace() {
                            self.unget_char(ch);
                            break;
                        } else {
                            result.push(c);
                        }
                    }
                    _  => {
                        self.unget_char(ch);
                        break;
                    },
                }
            }
            Ok(result)
        }
    }

    //mi read_quoted_string
    /// reads a quoted string, given the stream cursor is pointing at the opening quote character
    /// an empty quoted string is two identical quote characters then a different character (or EOF)
    /// a triple quoted string starts with three identical quote characters and continues (including newlines)
    /// until the next three identical quote characters
    /// otherwise it is a single quoted string, which should handle escapes (only \\ => \, \" => ", \' => ', \n => newline?)
    pub fn read_quoted_string(&mut self, reader:&mut R) -> Result<R, String> {
        let mut result = String::new();
        let ch = self.get_char_no_eof(reader)?;
        let ch2 = self.get_char_no_eof(reader)?;
        if ch == ch2 {
            match self.peek_char(reader)?.as_char() {
                Some(ch3) => {
                    if ch3 == ch2 {
                        self.get_char(reader)?;
                        self.read_triple_quoted_string(reader,ch)
                    } else {
                        Ok(result) // empty string
                    }
                },
                _ => {
                    Ok(result) // empty string
                },
            }
        } else { // build single quoted string - no newlines permitted, copy raw up to next 'ch' character
            let mut new_ch = ch2;
            while new_ch != ch {
                if is_newline(ch) {
                    return self.unexpected_newline_in_string(reader);
                }
                result.push(new_ch);
                new_ch = self.get_char_no_eof(reader)?;
            }
            Ok(result)
        }
    }

    //mi read_triple_quoted_string
    /// read a triple quoted string, with the stream cursor pointing
    /// at first character of contents (after the triple quote) keeps
    /// reading characters and pushing them until the three
    /// consecutive quote characters are seen.
    fn read_triple_quoted_string(&mut self, reader:&mut R, quote_char:char) -> Result<R, String> {
        let mut result = String::new();
        let mut num_quotes = 0;
        while num_quotes<3 {
            let ch = self.get_char_no_eof(reader)?;
            if ch==quote_char {
                num_quotes += 1;
            } else if num_quotes>0 {
                for _ in 0..num_quotes {
                    result.push(quote_char);
                }
                num_quotes = 0;
                result.push(ch);
            } else {
                result.push(ch);
            }
        }
        Ok(result)
    }

    //mi read_attribute
    // Stream is pointing at first character of attribute
    fn read_attribute(&mut self, reader:&mut R) -> Result<R, Token<R::Position>> {
        let span = Span::new_at(reader.borrow_pos());
        let (ns,name) = self.read_namespace_name(reader)?;
        let ch   = self.get_char_no_eof(reader)?;
        if ch != '=' {
            return self.expected_equals(reader, ch);
        }
        let value = self.read_string(reader)?;
        let span = span.end_at(reader.borrow_pos());
        Ok(Token::attribute(span, ns, name, value))
    }

    //zz All done
}
