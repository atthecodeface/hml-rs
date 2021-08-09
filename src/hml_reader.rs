//a Documentation
/*!

# HML reader module

This module provides a HML (human markup language) reader which allows
for reading of HML files to markup events and content.

!*/

mod builder;
mod lexer;
mod parser;
mod token;

mod test_lexer;
mod test_parser;

pub(self) use builder::{CloseTag, OpenTag, StackElement};
pub(self) use token::{Token, TokenType};

pub use lexer::Lexer;
pub use parser::Parser;
