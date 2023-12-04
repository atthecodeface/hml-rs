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
mod utils;

mod test_lexer;
mod test_parser;

use builder::{CloseTag, OpenTag, StackElement};
use token::{Token, TokenType};

pub use lexer::Lexer;
pub use parser::Parser;
