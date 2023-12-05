//a Imports
use crate::{HmlError, HmlResult};
use crate::{Posn, Span};

use super::utils::*;
use super::Token;

//a Lexer functions
//fi parse_comment_line
fn parse_comment_line<L, P>(
    lexer: &L,
    mut acc: Vec<(P, P)>,
    posn: &P,
    _n: usize,
    ch: char,
) -> (Vec<(P, P)>, Option<P>)
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let (posn, _) = lexer.do_while(*posn, ch, &|_, ch| (!is_newline(ch)) && ch.is_whitespace());
    let Some(ch) = lexer.peek_at(&posn) else {
        return (acc, None);
    };
    if ch != ';' {
        return (acc, None);
    }
    // Skip past the ';'
    // the comment line is from posn to the newline (or EOF)
    let comment_start = lexer.consumed_char(posn, ch);
    let Some(ch) = lexer.peek_at(&posn) else {
        acc.push((comment_start, comment_start));
        return (acc, None);
    };
    let (comment_end, _) = lexer.do_while(comment_start, ch, &|_, ch| !is_newline(ch));
    acc.push((comment_start, comment_end));
    if lexer.peek_at(&comment_end).is_some() {
        let posn = lexer.consumed_char(comment_end, '\n');
        (acc, Some(posn))
    } else {
        (acc, Some(comment_end))
    }
}

//fi parse_comment
fn parse_comment<L, P>(lexer: &L, posn: P, ch: char) -> HmlResult<Option<(P, Token<P>)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let (comment_end, opt_start_n_span) = lexer.fold(posn, ch, vec![], &parse_comment_line);
    let Some((comment_start, _n, spans)) = opt_start_n_span else {
        return Ok(None);
    };
    let comment_strings: Vec<String> = spans
        .into_iter()
        .map(|(s_p, e_p)| lexer.get_text(s_p, e_p).to_string())
        .collect();
    Ok(Some((
        comment_end,
        Token::comment(Span::new(comment_start, comment_end), comment_strings),
    )))
}

//fi parse_whitespace
fn parse_whitespace<L, P>(lexer: &L, posn: P, ch: char) -> HmlResult<Option<(P, Token<P>)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let (end, Some((start, _n))) = lexer.do_while(posn, ch, &|_, ch| ch.is_whitespace()) else {
        return Ok(None);
    };
    let token = Token::whitespace(Span::new(start, end));
    Ok(Some((end, token)))
}

//fi parse_name
fn parse_name<L, P>(lexer: &L, posn: P) -> HmlResult<Option<(P, String)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let Some(ch) = lexer.peek_at(&posn) else {
        return Ok(None);
    };
    let (end, Some((start, _n))) = lexer.do_while(posn, ch, &|n, ch| {
        (n == 0) && (is_name_start(ch)) || ((n > 0) && (is_name(ch)))
    }) else {
        return Ok(None);
    };
    let s = lexer.get_text(start, end).to_string();
    Ok(Some((end, s)))
}

//fi parse_namespace_name
fn parse_namespace_name<L, P>(lexer: &L, posn: P) -> HmlResult<Option<(P, String, String)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let Some((end_name, name)) = parse_name(lexer, posn)? else {
        return Ok(None);
    };
    match lexer.peek_at(&end_name) {
        Some(':') => {
            let posn = lexer.consumed_char(end_name, ':');
            let Some((end_name, name2)) = parse_name(lexer, posn)? else {
                return Ok(None);
            };
            Ok(Some((end_name, name, name2)))
        }
        _ => Ok(Some((end_name, "".into(), name))),
    }
}

//fi parse_tag
fn parse_tag<L, P>(lexer: &L, posn: P, ch: char) -> HmlResult<Option<(P, Token<P>)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let (hash_end, Some((start, hash_count))) = lexer.do_while(posn, ch, &|_, ch| is_hash(ch))
    else {
        return Ok(None);
    };

    let Some((end_name, ns, name)) = parse_namespace_name(lexer, hash_end)? else {
        let span = Span::new(start, hash_end);
        return Err(HmlError::ExpectedTagName { span });
    };
    let opt_ch = lexer.peek_at(&end_name);
    let (end_posn, result) = {
        match opt_ch {
            Some('{') => {
                let end_posn = lexer.consumed_char(end_name, '{');
                let span = Span::new(start, end_posn);
                (end_posn, Token::open_boxed(span, ns, name, hash_count))
            }
            Some('}') => {
                let end_posn = lexer.consumed_char(end_name, '}');
                let span = Span::new(start, end_posn);
                (end_posn, Token::close(span, ns, name, hash_count))
            }
            _ => {
                let span = Span::new(start, end_name);
                (end_name, Token::open(span, ns, name, hash_count))
            }
        }
    };
    // expect whitespace or EOF after ##tag (and optional brace)
    if let Some(ch) = lexer.peek_at(&end_posn) {
        if !ch.is_whitespace() {
            let span = Span::new(start, end_posn);
            return Err(HmlError::ExpectedWhitespaceAfterTag { span });
        }
    }
    Ok(Some((end_posn, result)))
}

//fi parse_quoted_string
/// reads a quoted string, given the stream cursor is pointing just beyond the opening quote character
///
/// The hash_count is the number of hashes in front of this quote
/// character - zero means no newlines are permitted inside the string
///
/// The string completes with the *next* quote character that is followed by hash_count hashes
///
/// The resultant string is the contents between the quote characters;
/// the end position includes the hash characters
fn parse_quoted_string<L, P>(
    lexer: &L,
    mut posn: P,
    quote_ch: char,
    hash_count: usize,
    _raw: bool,
) -> HmlResult<Option<(P, String)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let start = posn;
    let mut last_posn;
    loop {
        let Some(ch) = lexer.peek_at(&posn) else {
            return Ok(None);
        };
        if is_newline(ch) && hash_count == 0 {
            // return self.unexpected_newline_in_string(reader);
            return Ok(None);
        }
        last_posn = posn;
        posn = lexer.consumed_char(posn, ch);
        if ch == quote_ch {
            let mut i = 0;
            let mut hash_posn = posn;
            while i < hash_count {
                let Some(ch) = lexer.peek_at(&hash_posn) else {
                    return Ok(None);
                };
                if !is_hash(ch) {
                    break;
                }
                i += 1;
                hash_posn = lexer.consumed_char(hash_posn, ch);
            }
            if i == hash_count {
                let s = lexer.get_text(start, last_posn).to_string();
                return Ok(Some((hash_posn, s)));
            }
            posn = hash_posn;
        }
    }
}

//mi parse_string
/// Reads a string, possibly a quoted string, given the stream cursor is pointing at the opening character.
///
/// The string must start with a quote character or a different non-whitespace character
/// If it starts with a non-whitespace character then the string goes up to EOF or or whitespace
/// If it starts with a quote character then it is a quoted string
fn parse_string<L, P>(lexer: &L, posn: P) -> HmlResult<Option<(P, String)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let Some(ch) = lexer.peek_at(&posn) else {
        return Ok(None);
    };
    if is_quote(ch) {
        // || is_hash(ch) {
        let posn = lexer.consumed_char(posn, ch);
        parse_quoted_string(lexer, posn, ch, 0, false)
    } else if ch.is_whitespace() {
        Ok(None)
    } else {
        let (end_posn, _) = lexer.do_while(posn, ch, &|_, ch| !ch.is_whitespace());
        Ok(Some((end_posn, lexer.get_text(posn, end_posn).to_string())))
    }
}

//fi parse_attribute
fn parse_attribute<L, P>(lexer: &L, posn: P, _ch: char) -> HmlResult<Option<(P, Token<P>)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let start = posn;
    let Some((end_name, ns, name)) = parse_namespace_name(lexer, posn)? else {
        return Ok(None);
    };
    // expect = after name
    let Some(ch) = lexer.peek_at(&end_name) else {
        let span = Span::new(start, end_name);
        return Err(HmlError::UnexpectedEOF { span });
    };
    if ch != '=' {
        let span = Span::new(start, end_name);
        return Err(HmlError::ExpectedEquals { span, ch });
    }
    let posn = lexer.consumed_char(end_name, ch);
    let Some((end_posn, value)) = parse_string(lexer, posn)? else {
        let span = Span::new_at(&posn);
        // FIXME
        return Err(HmlError::ExpectedEquals { span, ch });
    };
    let span = Span::new(start, end_posn);
    Ok(Some((end_posn, Token::attribute(span, ns, name, value))))
}

//fi parse_character_string
fn parse_character_string<L, P>(lexer: &L, posn: P, ch: char) -> HmlResult<Option<(P, Token<P>)>, P>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: Posn,
{
    let start = posn;
    let raw = ch == 'r';
    let (ch, posn) = {
        if raw {
            let after_r = lexer.consumed_char(posn, ch);
            let Some(check_ch) = lexer.peek_at(&after_r) else {
                return Ok(None);
            };
            if !is_hash(check_ch) && !is_quote(check_ch) {
                return Ok(None);
            }
            (check_ch, after_r)
        } else {
            (ch, posn)
        }
    };
    let (posn, hash_count) = {
        if is_hash(ch) {
            let (hash_end, Some((_start, hash_count))) =
                lexer.do_while(posn, ch, &|_, ch| is_hash(ch))
            else {
                return Ok(None);
            };
            (hash_end, hash_count)
        } else {
            (posn, 0)
        }
    };
    let Some(ch) = lexer.peek_at(&posn) else {
        return Ok(None);
    };
    if !is_quote(ch) {
        return Ok(None);
    }
    let posn = lexer.consumed_char(posn, ch);
    let Some((posn, quoted_string)) = parse_quoted_string(lexer, posn, ch, hash_count, raw)? else {
        return Ok(None);
    };
    let span = Span::new(start, posn);
    if raw {
        Ok(Some((posn, Token::raw_characters(span, quoted_string))))
    } else {
        Ok(Some((posn, Token::characters(span, quoted_string))))
    }
}

//fp parse_fns
/// Return a Vec of parser functions that can be passed to a Lexer,
/// which provide a complete parsing of tokens for an HML reader
pub fn parse_fns<'parser, L, P>() -> Vec<lexer_rs::BoxDynLexerParseFn<'parser, L>>
where
    L: lexer_rs::Lexer<State = P, Token = Token<P>, Error = HmlError<P>>
        + lexer_rs::CharStream<P>
        + 'parser,
    P: Posn + 'parser,
{
    // Note ##r can introduce a string or can be a tag, and ### can introduce a string
    // Attempt to parse the string first
    vec![
        Box::new(parse_whitespace) as lexer_rs::BoxDynLexerParseFn<'parser, L>,
        Box::new(parse_comment) as lexer_rs::BoxDynLexerParseFn<'parser, L>,
        Box::new(parse_character_string) as lexer_rs::BoxDynLexerParseFn<'parser, L>,
        Box::new(parse_tag) as lexer_rs::BoxDynLexerParseFn<'parser, L>,
        Box::new(parse_attribute) as lexer_rs::BoxDynLexerParseFn<'parser, L>,
    ]
}

//a Tests
#[cfg(test)]
use super::TokenType;
//ft test_parse_comments
#[test]
fn test_parse_comments() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, HmlError<Posn>>;
    let parsers = [Box::new(parse_comment) as BoxDynLexerParseFn<TestLexer>];

    let l = TestLexer::new("not a comment");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(result.is_err());

    let l = TestLexer::new("; This is a comment");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(!result.is_err());
    let result = result.unwrap();
    assert!(result.is_some());
    let (posn, mut token) = result.unwrap();
    assert_eq!(token.token_type(), TokenType::Comment);
    assert!(l.peek_at(&posn).is_none());
    assert_eq!(token.take_contents()[0], " This is a comment");

    let l = TestLexer::new("    ; This is a comment");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(!result.is_err());
    let result = result.unwrap();
    assert!(result.is_some());
    let (posn, mut token) = result.unwrap();
    assert_eq!(token.token_type(), TokenType::Comment);
    assert!(l.peek_at(&posn).is_none());
    assert_eq!(token.take_contents()[0], " This is a comment");

    let l = TestLexer::new(
        r#"    ; This is a multi-line comment
      ; with a second line
"#,
    );
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(!result.is_err());
    let result = result.unwrap();
    assert!(result.is_some());
    let (posn, mut token) = result.unwrap();
    assert_eq!(token.token_type(), TokenType::Comment);
    assert!(l.peek_at(&posn).is_none());
    let contents = token.take_contents();
    assert_eq!(contents[0], " This is a multi-line comment");
    assert_eq!(contents[1], " with a second line");
}

//ft test_parse_whitespace
#[test]
fn test_parse_whitespace() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, HmlError<Posn>>;
    let parsers = [Box::new(parse_whitespace) as BoxDynLexerParseFn<TestLexer>];

    let l = TestLexer::new("not a comment");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(result.is_err());

    let l = TestLexer::new("     ");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(!result.is_err());
    let result = result.unwrap();
    assert!(result.is_some());
    let (posn, token) = result.unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);
    assert!(l.peek_at(&posn).is_none());

    let l = TestLexer::new("     ");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(!result.is_err());
    let result = result.unwrap();
    assert!(result.is_some());
    let (posn, token) = result.unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);
    assert!(l.peek_at(&posn).is_none());
}

//ft test_parse_comment_and_whitespace
#[test]
fn test_parse_comment_and_whitespace() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, HmlError<Posn>>;
    let parsers = [
        Box::new(parse_whitespace) as BoxDynLexerParseFn<TestLexer>,
        Box::new(parse_comment),
    ];

    let l = TestLexer::new("not a comment not whitespace");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(result.is_err());

    let l = TestLexer::new(
        "   ; This is a comment after some whitespace and whitespace afterwards\n   ",
    );
    let posn = Posn::default();
    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);
    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Comment);
    assert_eq!(
        token.take_contents()[0],
        " This is a comment after some whitespace and whitespace afterwards"
    );
    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    assert!(l.peek_at(&posn).is_none());
}

//ft test_parse_tag
#[test]
fn test_parse_tag() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, HmlError<Posn>>;
    let parsers = [
        Box::new(parse_whitespace) as BoxDynLexerParseFn<TestLexer>,
        Box::new(parse_tag),
    ];

    let l = TestLexer::new("not a tag");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(result.is_err());

    let l = TestLexer::new("##name ###second  ##close} #open_me{  ##ns:name");
    let posn = Posn::default();

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 2);
    assert!(!token.get_boxed());
    let contents = token.take_contents();
    assert_eq!(contents[0], "");
    assert_eq!(contents[1], "name");

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 3);
    assert!(!token.get_boxed());

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagClose);
    assert_eq!(token.get_depth(), 2);
    let contents = token.take_contents();
    assert_eq!(contents[0], "");
    assert_eq!(contents[1], "close");

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 1);
    assert!(token.get_boxed());

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 2);
    assert!(!token.get_boxed());
    let contents = token.take_contents();
    assert_eq!(contents[0], "ns");
    assert_eq!(contents[1], "name");

    assert!(l.peek_at(&posn).is_none());
}

//ft test_parse_character_string
#[test]
fn test_parse_character_string() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, HmlError<Posn>>;
    let parsers = [
        Box::new(parse_whitespace) as BoxDynLexerParseFn<TestLexer>,
        Box::new(parse_character_string),
    ];

    let l = TestLexer::new("not a tag");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(result.is_err());

    let l = TestLexer::new("'a string'\"Another string\"####'quoted string'### more'####");
    let posn = Posn::default();

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Characters);
    let contents = token.take_contents();
    assert_eq!(contents[0], "a string");

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Characters);
    let contents = token.take_contents();
    assert_eq!(contents[0], "Another string");

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Characters);
    let contents = token.take_contents();
    assert_eq!(contents[0], "quoted string'### more");

    assert!(l.peek_at(&posn).is_none());
}

//ft test_parse_attribute
#[test]
fn test_parse_attribute() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, HmlError<Posn>>;
    let parsers = [
        Box::new(parse_whitespace) as BoxDynLexerParseFn<TestLexer>,
        Box::new(parse_attribute),
    ];

    let l = TestLexer::new("not a tag");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(result.is_err());

    let l = TestLexer::new("not=1 fred='this'");
    let posn = Posn::default();

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Attribute);
    let contents = token.take_contents();
    assert_eq!(contents[0], "");
    assert_eq!(contents[1], "not");
    assert_eq!(contents[2], "1");

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, mut token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Attribute);
    let contents = token.take_contents();
    assert_eq!(contents[0], "");
    assert_eq!(contents[1], "fred");
    assert_eq!(contents[2], "this");
}
