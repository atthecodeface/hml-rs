//a Imports

//a Test
#[cfg(test)]
mod tests {
    // use super::super::types::*;
    use crate::string::Reader;
    use crate::hml_reader::Lexer;
    #[test]
    fn test_blah() {
        let buf = r#####"; This is a comment
   ; with more comment
   #banana #fred:tob{ r='2' r"Raw string" ##"Stuff "  and more "##"#####;
        let mut reader = Reader::new(buf);
        let mut lexer  = Lexer::new();
        loop {
            let t = lexer.next_token(&mut reader);
            assert_eq!( t.is_err(), false, "T should not be an error : {:?}", t);
            let token = t.unwrap();
            println!("{}", token);
            if token.is_eof() {break;}
        }
        // assert!(false);
    }
}
