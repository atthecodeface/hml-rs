mod traits;
mod error;
mod span;

mod token;
mod lexer;
mod parser;
mod builder;

mod test_lexer;
mod test_parser;

pub(self) use error::{Error, Result};

pub(self) use token::{Token, TokenType};
pub(self) use builder::{OpenTag, CloseTag, StackElement};
pub(self) use span::Span;

pub use lexer::Lexer;
pub use parser::Parser;
pub use traits::{Position, Character, Reader};

