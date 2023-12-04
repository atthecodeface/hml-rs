//a Documentation
/*!

# String as a [Reader]

This module provides a [Reader] implementation for `String`.

!*/

//a Imports
use crate::reader;

//a Position
//tp Position
/// Position within a string
///
/// This is used to derive a context for errors, for example; a Span
/// is between two Positions.
use lexer_rs::PosnInCharStream;
pub type Position = lexer_rs::StreamCharPos<lexer_rs::LineColumn>;

//a Character
//tp Character
/// A character as returned by the reader; this can be none or EOF as
/// well as a UTF8 character
#[derive(Copy, Clone, Debug)]
pub struct Character(Option<char>);

//ip Reader::Character for Character
impl reader::Character for Character {
    fn is_eof(&self) -> bool {
        self.0.is_none()
    }
    fn is_not_rdy(&self) -> bool {
        false
    }
    fn as_char(&self) -> Option<char> {
        self.0
    }
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
/// Error returned by a string
///
/// A String cannot generate errors - there is no underlying I/O for
/// example; a String is indeed guaranteed by Rust to be a sequence of
/// unicode code points.
///
/// Reading beyond the end of the String is not even an error - it provides EOF.
#[derive(Debug)]
pub struct Error();

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
    s: &'a str,
    chars: std::str::Chars<'a>,
    line_starts: Vec<usize>,
    n: Position,
}

//ip Reader
impl<'a> Reader<'a> {
    //fp new
    /// Create a new [Reader] by borrowing the `str` to read over
    pub fn new(s: &'a str) -> Self {
        let mut line_starts = Vec::new();
        line_starts.push(0);
        for (b, c) in s.char_indices() {
            if c == '\n' {
                line_starts.push(b + 1);
            }
        }
        use crate::reader::Position;
        Self {
            s,
            chars: s.chars(),
            line_starts,
            n: Position::none(),
        }
    }

    //fp of_file
    /// Create a new [Reader] by borrowing the `str` to read over
    pub fn of_file(f: &mut dyn std::io::Read, contents: &'a mut String) -> std::io::Result<Self> {
        f.read_to_string(contents)?;
        Ok(Self::new(contents))
    }

    //zz All done
}

//ip reader::Reader for Reader
impl<'a> reader::Reader for Reader<'a> {
    type Position = Position;
    type Char = Character;
    type Error = Error;

    fn next_char(&mut self) -> std::result::Result<Character, Self::Error> {
        match self.chars.next() {
            Some(ch) => {
                self.n = self.n.move_by_char(ch);
                Ok(Character(Some(ch)))
            }
            None => Ok(Character(None)),
        }
    }
    fn borrow_pos(&self) -> &Self::Position {
        &self.n
    }
}

//ip lexer_rs::FmtContext<Position> for Reader<'a>
impl<'a> lexer_rs::FmtContext<Position> for Reader<'a> {
    //fi fmt_line
    /// Output a single line of text to a formatter given a line number
    fn fmt_line(&self, f: &mut dyn std::fmt::Write, line_num: usize) -> std::fmt::Result {
        let ofs = self.line_starts[line_num - 1];
        let bytes = &(self.s.as_bytes()[ofs..]);
        let s = { unsafe { std::str::from_utf8_unchecked(bytes) } };
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
    fn line_length(&self, line_num: usize) -> usize {
        let ofs1 = self.line_starts[line_num - 1];
        let ofs2 = self.line_starts[line_num];
        ofs2 - ofs1
    }
}
