use crate::{ReaderPosition, ReaderChar, Reader};

#[derive(Copy, Clone, Debug)]
pub struct StringPos {byte:usize, line_num:usize, char_num:usize}
impl StringPos {
    fn new(byte:usize, line_num:usize, char_num:usize) -> Self {
        Self { byte, line_num, char_num }
    }
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
impl ReaderPosition for StringPos {
    fn none() -> Self { Self::new(0,1,1) }
}
impl std::fmt::Display for StringPos {
    //mp fmt
    /// Format for humans
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line_num, self.char_num)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct StringChr (Option<char>);
impl ReaderChar for StringChr {
    fn is_eof(&self)     -> bool { self.0.is_none() }
    fn is_not_rdy(&self) -> bool { false }
    fn as_char(&self)    -> Option<char> { self.0 }
}
impl std::fmt::Display for StringChr {
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

#[derive(Debug)]
pub struct StringReader<'a> {
    s     : &'a str,
    chars : std::str::Chars<'a>,
    line_starts : Vec<usize>,
    n     : StringPos,
}
impl <'a> StringReader<'a> {
    pub fn new(s:&'a str) -> Self {
        let mut line_starts = Vec::new();
        line_starts.push(0);
        for (b,c) in s.char_indices() {
            if c == '\n' {
                line_starts.push(b);
            }
        }
        Self { s, chars:s.chars(), line_starts, n:StringPos::none() }
    }
    fn fmt_line(&self, f: &mut std::fmt::Formatter, line_num:usize)  -> std::fmt::Result {
        use std::fmt::Write;
        let ofs = self.line_starts[line_num-1];
        let bytes = &(self.s.as_bytes()[ofs..]);
        let s = {unsafe {std::str::from_utf8_unchecked(bytes) }};
        for c in s.chars() {
            if c == '\n' {
                f.write_char(c)?;
            } else {
                break;
            }
        }
        Ok(())
    }
}
impl <'a> Reader for StringReader<'a> {
    type Position = StringPos;
    type Char     = StringChr;
    type Error    = std::io::Error;

    fn next_char(&mut self) -> std::result::Result<StringChr, std::io::Error> {
        match self.chars.next() {
            Some(ch) => {
                self.n.move_by_char(ch);
                Ok(StringChr(Some(ch)))
            },
            None => Ok(StringChr(None)),
        }
    }
    fn borrow_pos(&self) -> &Self::Position {
        &self.n
    }
    //   --> src/reader/test_lexer.rs:54:9
    //    |
    // 54 |..       self.byte == ch.len_utf8();
    //    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^
    //    |
    //    = note: `#[warn(unused_must_use)]` on by default
    fn fmt_context(&self, f: &mut std::fmt::Formatter, start:&StringPos, end:&StringPos) -> std::fmt::Result {
        use std::fmt::Write;
        if start.line_num == end.line_num {
            let mut num_chars = end.char_num - start.char_num;
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
            for _ in 1..start.char_num { f.write_char(' ')?; }
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


