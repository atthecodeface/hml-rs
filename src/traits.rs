//a Imports

//a ReaderPosition trait
//tt ReaderPosition
pub trait ReaderPosition : Clone + Copy + std::fmt::Debug + std::fmt::Display {
    fn none() -> Self;
}

//tt ReaderChar
pub trait ReaderChar : Clone + Copy + std::fmt::Debug + std::fmt::Display {
    fn is_eof(&self)     -> bool;
    fn is_not_rdy(&self) -> bool;
    fn as_char(&self)    -> Option<char>;
}

//tt Reader
pub trait Reader  : std::fmt::Debug {
    type Position : ReaderPosition;
    type Char     : ReaderChar;
    type Error    : std::error::Error + 'static;
    fn next_char(&mut self) -> std::result::Result<Self::Char, Self::Error>;
    fn borrow_pos(&self) -> &Self::Position;
    fn fmt_context(&self, f: &mut std::fmt::Formatter, start:&Self::Position, end:&Self::Position) -> std::fmt::Result ;
}


//tt StreamSpan
/// Used for reading and writing
pub trait StreamSpan : Clone + std::fmt::Debug {
}
