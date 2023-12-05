//a Documentation
/*!

# HML reader module

This module provides a HML (human markup language) reader which allows
for reading of HML files to markup events and content.

!*/

mod builder;
mod parser;
mod token;
mod utils;

mod test_parser;

use builder::{CloseTag, OpenTag, StackElement};
use token::{Token, TokenType};

mod lexer_parsers;
pub use lexer_parsers::parse_fns;
pub use parser::Parser;
