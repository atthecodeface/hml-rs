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

@file    string.rs
@brief   String reader implementation for Markup library
 */

//a Imports
use crate::reader;

//a Position
//tp Position
/// Position within a string
///
/// This is used to derive a context for errors, for example; a Span
/// is between two Positions.
#[derive(Copy, Clone, Debug)]
pub struct Position {
    /// Byte offset within the u8 forming the string
    byte:usize,
    /// Line number
    line_num:usize,
    /// Character offset within the line
    char_num:usize,
}

//ip Position
impl Position {
    //fp new
    /// Create a new Position
    fn new(byte:usize, line_num:usize, char_num:usize) -> Self {
        Self { byte, line_num, char_num }
    }

    //fp move_by_char
    /// Move on by a character
    fn move_by_char(&mut self, ch:char) {
        self.byte += ch.len_utf8();
        if ch == '\n' {
            self.line_num += 1;
            self.char_num = 1;
        } else {
            self.char_num += 1;
        }
    }
}

//ip Reader::Position for Position
impl reader::Position for Position {
    //fp none
    fn none() -> Self { Self::new(0,1,1) }
}

//ip Display for Position
impl std::fmt::Display for Position {
    //mp fmt
    /// Format for humans
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "line {} character {}", self.line_num, self.char_num)
    }
}

//a Character
//tp Character
/// A character as returned by the reader; this can be none or EOF as
/// well as a UTF8 character
#[derive(Copy, Clone, Debug)]
pub struct Character (Option<char>);

//ip Reader::Character for Character
impl reader::Character for Character {
    fn is_eof(&self)     -> bool { self.0.is_none() }
    fn is_not_rdy(&self) -> bool { false }
    fn as_char(&self)    -> Option<char> { self.0 }
}

//ip Display for Character
impl std::fmt::Display for Character {
    //mp fmt
    /// Format for humans
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::fmt::Write;
        match self.0 {
            Some(c) => f.write_char(c),
            None => write!(f, "None"),
        }
    }
}

//a Error
//tp Error
#[derive(Debug)]
pub struct Error ();

//ip reader::Error for Error
impl reader::Error for Error {
    type Position = Position;
    fn write_without_span(&self, _f:&mut dyn std::fmt::Write) -> std::fmt::Result {
        Ok(())
    }
    fn borrow_span(&self) -> Option<&reader::Span<Position>> {
        None
    }
}

//ip std::error::Error for Error
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

//ip std::fmt::Display for Error
impl std::fmt::Display for Error {
    //mp fmt - format a `Error` for display
    /// Display the `Error` in a human-readable form
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Ok(())
    }
}

//a Reader
//tp Reader
/// A markup Reader type that operates on strings
///
/// It borrows the string and creates a Vec of where within the string
/// each new line starts.
///
/// As such it has an overhead compared to the string itself, and for
/// handling markup of large data this type may consume more memory
/// than is desired.
///
/// When formatting a Span this will generate a `rustc`-like multiline
/// string highlighting the characters within the Span. This provides
/// for good error reporting.
//   --> src/reader/test_lexer.rs:54:9
//    |
// 54 |..       self.byte == ch.len_utf8();
//    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^
//    |
//    = note: `#[warn(unused_must_use)]` on by default
#[derive(Debug)]
pub struct Reader<'a> {
    s           : &'a str,
    chars       : std::str::Chars<'a>,
    line_starts : Vec<usize>,
    n           : Position,
}

//ip Reader
impl <'a> Reader<'a> {
    //fp new
    /// Create a new [Reader] by borrowing the `str` to read over
    pub fn new(s:&'a str) -> Self {
        let mut line_starts = Vec::new();
        line_starts.push(0);
        for (b,c) in s.char_indices() {
            if c == '\n' {
                line_starts.push(b);
            }
        }
        use crate::reader::Position;
        Self { s, chars:s.chars(), line_starts, n:Position::none() }
    }

    //fp of_file
    /// Create a new [Reader] by borrowing the `str` to read over
    pub fn of_file(f:&mut dyn std::io::Read, contents:&'a mut String) -> std::io::Result<Self> {
        f.read_to_string(contents)?;
        Ok(Self::new(contents))
    }

    //fi fmt_line
    /// Output a single line of text to a formatter given a line number
    fn fmt_line(&self, f:&mut dyn std::fmt::Write, line_num:usize)  -> std::fmt::Result {
        let ofs = self.line_starts[line_num-1];
        let bytes = &(self.s.as_bytes()[ofs..]);
        let s = {unsafe {std::str::from_utf8_unchecked(bytes) }};
        for c in s.chars() {
            if c == '\n' {
                break;
            } else {
                f.write_char(c)?;
            }
        }
        Ok(())
    }

    //fi line_length
    /// Find line length
    fn line_length(&self, line_num:usize)  -> usize {
        let ofs1 = self.line_starts[line_num-1];
        let ofs2 = self.line_starts[line_num];
        ofs2 - ofs1
    }

    //zz All done
}

//ip reader::Reader for Reader
impl <'a> reader::Reader for Reader<'a> {
    type Position = Position;
    type Char     = Character;
    type Error    = Error;

    fn next_char(&mut self) -> std::result::Result<Character, Self::Error> {
        match self.chars.next() {
            Some(ch) => {
                self.n.move_by_char(ch);
                Ok(Character(Some(ch)))
            },
            None => Ok(Character(None)),
        }
    }
    fn borrow_pos(&self) -> &Self::Position {
        &self.n
    }

    fn fmt_context(&self, f: &mut dyn std::fmt::Write, start:&Position, end:&Position) -> std::fmt::Result {
        if start.line_num == end.line_num || (start.line_num+1 == end.line_num && end.char_num == 1){
            let mut num_chars = {
                if start.line_num == end.line_num {
                    end.char_num - start.char_num
                } else {
                    self.line_length(start.line_num)
                }
            };
            if num_chars == 0 { num_chars = 1; }
            if start.line_num > 1 {
                write!(f, "    |  ")?;
                self.fmt_line(f, start.line_num-1)?;
                write!(f, "\n")?;
            }
            write!(f, "{:4}|  ", start.line_num)?;
            self.fmt_line(f, start.line_num)?;
            write!(f, "\n")?;
            write!(f, "    |  ")?;
            for _ in 1..(start.char_num-1) { f.write_char(' ')?; }
            for _ in 0..num_chars { f.write_char('^')?; }
            write!(f, "\n")?;
            write!(f, "    |  ")?;
            write!(f, "\n")?;
            Ok(())
        } else {
            Ok(())
        }
    }
}


