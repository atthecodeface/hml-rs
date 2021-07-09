//a Imports

//a Test
#[cfg(test)]
mod tests {
    // use super::super::types::*;
    use crate::string::Reader;
    use crate::reader::Lexer;
    #[test]
    fn test_blah() {
        let buf = "; This is a comment\n   ; with more comment\n #banana #fred:tob{ r='2' \"\"\"Stuff \"\"  and more \"\"\"";
        let mut reader = Reader::new(buf);
        let mut lexer  = Lexer::new();
        loop {
            let t = lexer.next_token(&mut reader);
            assert_eq!( t.is_err(), false, "T should not be an error");
            let token = t.unwrap();
            println!("{}", token);
            if token.is_eof() {break;}
        }
        // assert!(false);
    }
}
