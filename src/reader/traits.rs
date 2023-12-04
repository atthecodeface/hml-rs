//a Character trait
//tt Reader
/// The trait required of a Reader for its use with markup language readers
pub trait Reader: std::fmt::Debug + lexer_rs::FmtContext<Self::Position> {
    /// The type of the position within a stream for the reader, used by its Error and Span
    type Position: lexer_rs::PosnInCharStream;
    /// The type of errors created by the reader
    type Error: std::fmt::Debug; // Error<Position = Self::Position>;
    /// Return the next character from the stream - this can indicate
    /// end of file or an actual character.
    fn next_char(&mut self) -> std::result::Result<Option<char>, Self::Error>;
    /// Borrow the position of the reader's next character
    fn borrow_pos(&self) -> &Self::Position;
}
