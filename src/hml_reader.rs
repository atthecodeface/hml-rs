//a Documentation
/*!

# HML reader module

This module provides a HML (human markup language) parser which allows
for reading of HML files to markup events and content.

This uses the lexer_rs module to generate HML tokens, which are parsed
with the HML [Parser].

An HML [Parser] instance requires a namespace, and stack of
namespaces, to operate as it runs over an HML document. It's
'next_event' method is invoked with a callback that provides HML lexer
tokens, and it returns an HML document event.

The HML document events are similar to those of the reader in xml_rs.

```text
 let mut namespace = Namespace::new(false);
 let mut namespace_stack = NamespaceStack::new(&mut namespace);
 match parser.next_event(&mut namespace_stack, || lexer_iter.next()) {
  ...
 }
```

The lexer can be created for a String using:
```text
 let lexer_string = lexer_rs::LexerOfString::default().set_text(text);
 let lexer = lexer_string.lexer();
 let lexer_parsers = hml_rs::hml_reader::parse_fns();
 let mut lexer_iter = lexer.iter(&lexer_parsers);
```

Any type that supports the lexer_rs::Lexer and the
lexer_rs::CharStream trait can be used instead of 'lexer'; the trait
requires a 'Position' in the stream to be specified (which must
support lexer_rs::PosnInCharStream) - this can be a simple usize
(which is just a byte offset into a str, always pointing at a UTF8
boundary), or lexer_rs::LineColumn (for example) which tracks the line
and column in addition to the byte offset that 'usize' tracks.

If the lexer_rs::LexerOfString is used with LineColumn as the position
then the lexer_rs::FmtContext trait can be used to display errors with
full context.

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
