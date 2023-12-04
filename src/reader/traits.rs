//a Character trait
//tt Character
/// The trait required for a character returned by a Reader
///
/// A character can be a char or an end of file marker; it can also
/// (in the future) be a 'not ready' indication - where there data may
/// become ready in a later Reader get character call
pub trait Character: Clone + Copy + std::fmt::Debug + std::fmt::Display + 'static {
    /// Return true if the character is end-of-file
    fn is_eof(&self) -> bool;
    /// Return true if the character is not-ready
    fn is_not_rdy(&self) -> bool;
    /// Return Some(c) if the character corresponds to a real `char`
    fn as_char(&self) -> Option<char>;
}

//tt Reader
/// The trait required of a Reader for its use with markup language readers
pub trait Reader: std::fmt::Debug + lexer_rs::FmtContext<Self::Position> {
    /// The type of the position within a stream for the reader, used by its Error and Span
    type Position: lexer_rs::PosnInCharStream;
    /// The type of characters returned by the reader
    type Char: Character;
    /// The type of errors created by the reader
    type Error: std::fmt::Debug; // Error<Position = Self::Position>;
    /// Return the next character from the stream - this can indicate
    /// end of file or an actual character.
    fn next_char(&mut self) -> std::result::Result<Self::Char, Self::Error>;
    /// Borrow the position of the reader's next character
    fn borrow_pos(&self) -> &Self::Position;
}
