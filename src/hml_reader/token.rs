//a Imports
use crate::reader::{Position, Span};
use std::collections::VecDeque;

//a Token
//tp Token
/// [Token] represents a single item in an HML document
/// This will be an entity that effects the parse state of the parser
/// Hence it includes all of attr="string with spaces"
///
/// Missing are whether characters is escapable or not
///
/// and processing instruction
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenType {
    /// ; stuff up to newline
    Comment,
    /// ###<tag>[{] Tag open - with depth (number of #) and true if boxed
    TagOpen,
    /// ###<tag>} Tag close - with depth (number of #)
    TagClose,
    /// attribute [<string>:]<string>=<quoted string>
    Attribute,
    /// Quoted string of raw characters
    RawCharacters,
    /// Quoted string which needs interpretation (escapes handled)
    Characters,
    /// End of file
    EndOfFile,
    /// Whitespace that should in general be ignored
    Whitespace,
}

#[derive(Debug, Clone)]
pub struct Token<P: Position> {
    span: Span<P>,
    tt: TokenType,
    contents: VecDeque<String>,
    depth: usize,
    boxed: bool,
}

//ip Token
impl<P: Position> Token<P> {
    //fi new
    fn new(span: Span<P>, tt: TokenType, depth: usize, boxed: bool) -> Self {
        let contents = VecDeque::new();
        Self {
            span,
            tt,
            contents,
            depth,
            boxed,
        }
    }

    //cp add_string
    pub fn add_string(mut self, s: String) -> Self {
        self.contents.push_back(s);
        self
    }

    //fp open_boxed
    pub fn open_boxed(span: Span<P>, ns: String, name: String, depth: usize) -> Self {
        Self::new(span, TokenType::TagOpen, depth, true)
            .add_string(ns)
            .add_string(name)
    }

    //fp open
    pub fn open(span: Span<P>, ns: String, name: String, depth: usize) -> Self {
        Self::new(span, TokenType::TagOpen, depth, false)
            .add_string(ns)
            .add_string(name)
    }

    //fp close
    pub fn close(span: Span<P>, ns: String, name: String, depth: usize) -> Self {
        Self::new(span, TokenType::TagClose, depth, false)
            .add_string(ns)
            .add_string(name)
    }

    //fp attribute
    pub fn attribute(span: Span<P>, ns: String, name: String, value: String) -> Self {
        Self::new(span, TokenType::Attribute, 0, false)
            .add_string(ns)
            .add_string(name)
            .add_string(value)
    }

    //fp comment
    /// Consumes the Vec<String>
    pub fn comment(span: Span<P>, strings: Vec<String>) -> Self {
        let mut t = Self::new(span, TokenType::Comment, 0, false);
        for s in strings {
            t = t.add_string(s);
        }
        t
    }

    //fp whitespace
    /// Whitespace that should in general be ignored
    pub fn whitespace(span: Span<P>) -> Self {
        Self::new(span, TokenType::Whitespace, 0, false)
    }

    //fp raw_characters
    pub fn raw_characters(span: Span<P>, s: String) -> Self {
        Self::new(span, TokenType::RawCharacters, 0, false).add_string(s)
    }

    //fp characters
    pub fn characters(span: Span<P>, s: String) -> Self {
        Self::new(span, TokenType::Characters, 0, false).add_string(s)
    }

    //fp eof
    pub fn eof(span: Span<P>) -> Self {
        Self::new(span, TokenType::EndOfFile, 0, false)
    }

    //mp token_type
    pub fn token_type(&self) -> TokenType {
        self.tt
    }

    //mp get_span
    pub fn get_span(&self) -> &Span<P> {
        &self.span
    }

    //mp get_depth
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    //mp get_boxed
    pub fn get_boxed(&self) -> bool {
        self.boxed
    }

    //mp contents
    pub fn contents(&self) -> &[String] {
        self.contents.as_slices().0
    }

    //mp take_contents
    pub fn take_contents(&mut self) -> VecDeque<String> {
        self.contents.split_off(0)
    }

    //mp is_eof
    pub fn is_eof(&self) -> bool {
        self.tt == TokenType::EndOfFile
    }

    //mp is_attribute
    pub fn is_attribute(&self) -> bool {
        self.tt == TokenType::Attribute
    }

    //mp is_whitespace
    pub fn is_whitespace(&self) -> bool {
        self.tt == TokenType::Whitespace
    }
}

//ip std::fmt::Display for Token
impl<P: Position> std::fmt::Display for Token<P> {
    //mp fmt - format a `Token` for display
    /// Display the `Token` in a human-readable form
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use TokenType::*;
        match self.tt {
            Comment => write!(f, "[{}]; ...", self.span),
            TagOpen => {
                if self.boxed {
                    write!(
                        f,
                        "[{}]#<{}>{}:{}{{",
                        self.span, self.depth, self.contents[0], self.contents[1]
                    )
                } else {
                    write!(
                        f,
                        "[{}]#<{}>{}:{}",
                        self.span, self.depth, self.contents[0], self.contents[1]
                    )
                }
            }
            TagClose => {
                write!(
                    f,
                    "[{}]#<{}>{}:{}}}",
                    self.span, self.depth, self.contents[0], self.contents[1]
                )
            }
            Attribute => {
                write!(
                    f,
                    "[{}]{}:{}={}",
                    self.span, self.contents[0], self.contents[1], self.contents[2]
                )
            }
            Characters => {
                write!(f, "[{}]chars ...", self.span)
            }
            RawCharacters => {
                write!(f, "[{}]rawchars ...", self.span)
            }
            Whitespace => {
                write!(f, "[{}]whitespace", self.span)
            }
            EndOfFile => write!(f, "[{}]<eof>", self.span),
        }
    }
}

//a Lexer functions
//fi parse_comment_line
fn parse_comment_line<L, P>(
    lexer: &L,
    mut acc: Vec<(P, P)>,
    posn: &P,
    n: usize,
    mut ch: char,
) -> (Vec<(P, P)>, Option<P>)
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
{
    let (posn, _) = lexer.do_while(*posn, ch, &|_, ch| (ch != '\n') && ch.is_whitespace());
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
    let (comment_end, _) = lexer.do_while(comment_start, ch, &|_, ch| ch != '\n');
    acc.push((comment_start, comment_end));
    if lexer.peek_at(&comment_end).is_some() {
        let posn = lexer.consumed_char(comment_end, '\n');
        (acc, Some(posn))
    } else {
        (acc, Some(comment_end))
    }
}

//fi parse_comment
fn parse_comment<L, P>(
    lexer: &L,
    posn: P,
    ch: char,
) -> Result<Option<(P, Token<P>)>, lexer_rs::SimpleParseError<P>>
// ) -> Result<Option<(P, Token<P>)>, crate::reader::ReaderError<P, E>>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
    //     E: std::fmt::Debug,
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
        Token::comment(
            crate::reader::Span::between(&comment_start, &comment_end),
            comment_strings,
        ),
    )))
}

//fi parse_whitespace
fn parse_whitespace<L, P>(
    lexer: &L,
    posn: P,
    ch: char,
) -> Result<Option<(P, Token<P>)>, lexer_rs::SimpleParseError<P>>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
{
    let (end, Some((start, _n))) = lexer.do_while(posn, ch, &|_, ch| ch.is_whitespace()) else {
        return Ok(None);
    };
    let token = Token::whitespace(crate::reader::Span::between(&start, &end));
    Ok(Some((end, token)))
}

//ip impl Position for StreamCharPos
impl crate::reader::Position for lexer_rs::StreamCharPos<lexer_rs::LineColumn> {
    fn none() -> Self {
        Self::default()
    }
}

//fi parse_name
use crate::hml_reader::lexer::{is_name, is_name_start, is_quote};
fn parse_name<L, P>(
    lexer: &L,
    posn: P,
) -> Result<Option<(P, String)>, lexer_rs::SimpleParseError<P>>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
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
fn parse_namespace_name<L, P>(
    lexer: &L,
    posn: P,
) -> Result<Option<(P, String, String)>, lexer_rs::SimpleParseError<P>>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
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
fn parse_tag<L, P>(
    lexer: &L,
    posn: P,
    ch: char,
) -> Result<Option<(P, Token<P>)>, lexer_rs::SimpleParseError<P>>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
{
    let (hash_end, Some((start, hash_count))) = lexer.do_while(posn, ch, &|_, ch| ch == '#') else {
        return Ok(None);
    };

    let Some((end_name, ns, name)) = parse_namespace_name(lexer, hash_end)? else {
        return Ok(None);
        // return Err("Expected name or quote after #'s");
    };
    let opt_ch = lexer.peek_at(&end_name);
    let (end_posn, result) = {
        match opt_ch {
            Some('{') => {
                let end_posn = lexer.consumed_char(end_name, '{');
                let span = crate::reader::Span::between(&start, &end_posn);
                (end_posn, Token::open_boxed(span, ns, name, hash_count))
            }
            Some('}') => {
                let end_posn = lexer.consumed_char(end_name, '}');
                let span = crate::reader::Span::between(&start, &end_posn);
                (end_posn, Token::close(span, ns, name, hash_count))
            }
            _ => {
                let span = crate::reader::Span::between(&start, &end_name);
                (end_name, Token::open(span, ns, name, hash_count))
            }
        }
    };
    // expect whitespace or EOF after ##tag (and optional brace)
    if let Some(ch) = lexer.peek_at(&end_posn) {
        if !ch.is_whitespace() {
            return Ok(None);
            // return Err("Expected whitespace after ##tag");
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
) -> Result<Option<(P, String)>, lexer_rs::SimpleParseError<P>>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
{
    let start = posn;
    let mut last_posn = start;
    loop {
        let Some(ch) = lexer.peek_at(&posn) else {
            return Ok(None);
        };
        if ch == '\n' && hash_count == 0 {
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
                if ch != '#' {
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

//fi parse_character_string
fn parse_character_string<L, P>(
    lexer: &L,
    posn: P,
    ch: char,
) -> Result<Option<(P, Token<P>)>, lexer_rs::SimpleParseError<P>>
where
    L: lexer_rs::Lexer<State = P> + lexer_rs::CharStream<P>,
    P: lexer_rs::PosnInCharStream + crate::reader::Position,
{
    let start = posn;
    let (posn, hash_count) = {
        if ch == '#' {
            let (hash_end, Some((_start, hash_count))) =
                lexer.do_while(posn, ch, &|_, ch| ch == '#')
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
    let Some((posn, quoted_string)) = parse_quoted_string(lexer, posn, ch, hash_count, false)?
    else {
        return Ok(None);
    };
    let span = crate::reader::Span::between(&start, &posn);
    Ok(Some((posn, Token::characters(span, quoted_string))))
}

//a Tests
//ft test_parse_comments
#[test]
fn test_parse_comments() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, SimpleParseError<Posn>>;
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
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, SimpleParseError<Posn>>;
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
    let (posn, mut token) = result.unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);
    assert!(l.peek_at(&posn).is_none());

    let l = TestLexer::new("     ");
    let posn = Posn::default();
    let result = l.parse(posn, &parsers);
    assert!(!result.is_err());
    let result = result.unwrap();
    assert!(result.is_some());
    let (posn, mut token) = result.unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);
    assert!(l.peek_at(&posn).is_none());
}

//ft test_parse_comment_and_whitespace
#[test]
fn test_parse_comment_and_whitespace() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, SimpleParseError<Posn>>;
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
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, SimpleParseError<Posn>>;
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

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 2);
    assert!(!token.get_boxed());
    assert_eq!(token.contents[0], "");
    assert_eq!(token.contents[1], "name");

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 3);
    assert!(!token.get_boxed());

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagClose);
    assert_eq!(token.get_depth(), 2);
    assert_eq!(token.contents[0], "");
    assert_eq!(token.contents[1], "close");

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 1);
    assert!(token.get_boxed());

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Whitespace);

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::TagOpen);
    assert_eq!(token.get_depth(), 2);
    assert!(!token.get_boxed());
    assert_eq!(token.contents[0], "ns");
    assert_eq!(token.contents[1], "name");

    assert!(l.peek_at(&posn).is_none());
}

//ft test_parse_character_string
#[test]
fn test_parse_character_string() {
    use lexer_rs::*;
    type Posn = StreamCharPos<LineColumn>;
    type TestLexer<'a> = LexerOfStr<'a, Posn, Token<Posn>, SimpleParseError<Posn>>;
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

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Characters);
    assert_eq!(token.contents[0], "a string");

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Characters);
    assert_eq!(token.contents[0], "Another string");

    let (posn, token) = l.parse(posn, &parsers).unwrap().unwrap();
    assert_eq!(token.token_type(), TokenType::Characters);
    assert_eq!(token.contents[0], "quoted string'### more");

    assert!(l.peek_at(&posn).is_none());
}
