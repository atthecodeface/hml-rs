//a Test infrastructure
#[cfg(test)]
#[allow(dead_code)]
mod test_infrastructure {
    //a Imports
    use crate::hml_reader::Parser;
    use crate::markup::ContentType;
    use crate::names::{Name, Namespace, NamespaceStack, Tag};
    use crate::HmlError;
    type StringPosition = lexer_rs::StreamCharPos<lexer_rs::LineColumn>;

    use lexer_rs::{Lexer, LexerOfString, LineColumn, StreamCharPos};

    type LexerPos = StreamCharPos<LineColumn>;
    type StringError = HmlError<LexerPos>;
    type StringLexer = LexerOfString<LexerPos, crate::hml_reader::Token<LexerPos>, StringError>;
    type Event = crate::markup::Event<LexerPos>;

    //a Expectation
    //tp Expectation
    #[derive(Debug)]
    pub enum Expectation<'a> {
        StD(usize),
        StE(&'a str, &'a str, &'a [(&'a str, &'a str, &'a str)]),
        Content(ContentType, &'a str),
        EndE,
        EndD,
        Ignore,
    }

    //tp ExpectationState
    pub struct ExpectationState<'a> {
        expectations: &'a [Expectation<'a>],
        index: usize,
        stack: Vec<Name>,
    }

    //ip ExpectationState
    impl<'a> ExpectationState<'a> {
        //fp new
        fn new(expectations: &'a [Expectation<'a>]) -> Self {
            Self {
                expectations,
                index: 0,
                stack: Vec::new(),
            }
        }

        //fp match_open
        fn match_open(
            &mut self,
            ns_stack: &NamespaceStack,
            tag: Tag,
            uri: &str,
            name: &str,
            attrs: &[(&str, &str, &str)],
        ) -> bool {
            if ns_stack.name_str(tag.name.name) != name {
                false
            } else if ns_stack.uri_str(tag.name.uri) != uri {
                false
            } else {
                self.stack.push(tag.name);
                let tag_attrs = tag.attributes.take();
                if tag_attrs.len() != attrs.len() {
                    false
                } else {
                    for (e, a) in attrs.iter().zip(tag_attrs) {
                        println!(
                            "Check attrs {:?} {:?} {} {}",
                            e,
                            a,
                            ns_stack.uri_str(a.name.uri),
                            ns_stack.name_str(a.name.name)
                        );
                        if ns_stack.uri_str(a.name.uri) != e.0 {
                            return false;
                        }
                        if ns_stack.name_str(a.name.name) != e.1 {
                            return false;
                        }
                        if a.value != e.2 {
                            return false;
                        }
                    }
                    true
                }
            }
        }

        //fp match_close
        fn match_close(&mut self, _ns_stack: &NamespaceStack, name: &Name) -> bool {
            if let Some(stack_name) = self.stack.pop() {
                if *name == stack_name {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }

        //fp check_expectation
        fn check_expectation(
            &mut self,
            ns_stack: &NamespaceStack,
            t: Result<Event, StringError>,
        ) -> Result<(), String> {
            self.index += 1;
            if self.index > self.expectations.len() {
                return Err(format!("Ran out of expectations, got {:?}", t));
            }
            let (failure_string, pass) = {
                match self.expectations[self.index - 1] {
                    Expectation::StD(v) => (
                        format!("Expected a StartDocument {}, got {:?}", v, t),
                        t.is_ok() && t.unwrap().as_start_document() == Some(v),
                    ),
                    Expectation::EndD => (
                        format!("Expected an EndDocument, got {:?}", t),
                        t.is_ok() && t.unwrap().is_end_document(),
                    ),
                    Expectation::StE(ns, name, attrs) => (
                        format!(
                            "Expected a StartElement {}:{} with attrs {:?}, got {:?}",
                            ns, name, attrs, t
                        ),
                        if t.is_err() {
                            false
                        } else {
                            let t = t.unwrap();
                            if let Some(tag) = t.as_start_element() {
                                self.match_open(ns_stack, tag, ns, name, attrs)
                            } else {
                                false
                            }
                        },
                    ),
                    Expectation::EndE => (
                        format!("Expected an EndElement got {:?}", t),
                        if t.is_err() {
                            false
                        } else {
                            let t = t.unwrap();
                            if let Some(name) = t.as_end_element() {
                                self.match_close(ns_stack, name)
                            } else {
                                false
                            }
                        },
                    ),
                    Expectation::Content(et, es) => (
                        format!("Expected a Content {:?} got {:?}", et, t),
                        if t.is_err() {
                            false
                        } else {
                            let t = t.unwrap();
                            if let Some((ct, cs)) = t.as_content() {
                                if et != ct {
                                    false
                                } else if es != cs {
                                    false
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        },
                    ),
                    Expectation::Ignore => (String::new(), true),
                }
            };
            if pass {
                Ok(())
            } else {
                Err(failure_string)
            }
        }
    }

    //a Functions for test
    //fp test_string
    pub fn test_string(text: &str, exp: &[Expectation]) {
        let mut expectation = ExpectationState::new(exp);
        let mut namespace = Namespace::new(true);
        let mut namespace_stack = NamespaceStack::new(&mut namespace);
        namespace_stack.add_null_ns();
        let lexer_string = StringLexer::default().set_text(text);
        let lexer = lexer_string.lexer();
        let lexer_parsers = crate::hml_reader::parse_fns();
        let mut lexer_iter = lexer.iter(&lexer_parsers);
        let mut parser: Parser<lexer_rs::StreamCharPos<lexer_rs::LineColumn>> = Parser::default();
        let mut errors = Vec::new();
        loop {
            let t = parser.next_event(&mut namespace_stack, || lexer_iter.next());
            // println!("{:?}", t);
            let eof = t.is_ok() && t.as_ref().unwrap().is_end_document();
            let is_err = t.is_err();
            match expectation.check_expectation(&namespace_stack, t) {
                Err(x) => {
                    errors.push(x);
                    if is_err {
                        break;
                    }
                }
                _ => (),
            }
            if eof {
                break;
            }
        }
        println!("{}", text);
        for e in &errors {
            println!("FAIL: {}", e);
        }
        assert!(errors.is_empty());
    }

    //zz All done
}

//a Tests
#[cfg(test)]
#[allow(dead_code)]
mod tests {
    //a Imports from test_infrastructure
    use super::test_infrastructure::test_string;
    use super::test_infrastructure::Expectation::{Content, EndD, EndE, StD, StE};
    use crate::markup::ContentType;

    //a Structure tests
    #[test]
    fn test_structure() {
        test_string(
            "#svg ##line ##text",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE("", "line", &[]),
                EndE,
                StE("", "text", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_structure1() {
        // Note that #r can introduce a raw string or it may be a tag
        test_string(
            "#svg ##rect ##text",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE("", "rect", &[]),
                EndE,
                StE("", "text", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_structure2() {
        test_string(
            "#svg ##box{ ##box}",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE("", "box", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_structure3() {
        test_string(
            "#svg ##box{ #line ##box}",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE("", "box", &[]),
                StE("", "line", &[]),
                EndE,
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_structure4() {
        test_string(
            "#svg ##box{ #line #line ##box}",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE("", "box", &[]),
                StE("", "line", &[]),
                EndE,
                StE("", "line", &[]),
                EndE,
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_structure5() {
        test_string(
            "#svg ##box{ #innerbox{ #line #innerbox} ##box}",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE("", "box", &[]),
                StE("", "innerbox", &[]),
                StE("", "line", &[]),
                EndE,
                EndE,
                EndE,
                EndE,
                EndD,
            ],
        );
    }

    //a Attribute tests
    #[test]
    fn test_attr1() {
        test_string(
            "#svg a='1' ##line b='2'",
            &[
                StD(100),
                StE("", "svg", &[("", "a", "1")]),
                StE("", "line", &[("", "b", "2")]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_attr2() {
        test_string(
            "#svg a='1' b='2' ##line ",
            &[
                StD(100),
                StE("", "svg", &[("", "a", "1"), ("", "b", "2")]),
                StE("", "line", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_attr3() {
        test_string(
            "#svg ##box{ a='1' b='2' ##box} ##line ",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE("", "box", &[("", "a", "1"), ("", "b", "2")]),
                EndE,
                StE("", "line", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }

    //a Namespace tests
    #[test]
    fn test_ns() {
        test_string(
            "#svg ##box{ xmlns='https://fred' ##box} ##line ",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE(
                    "https://fred",
                    "box",
                    &[("http://www.w3.org/2000/xmlns/", "xmlns", "https://fred")],
                ),
                EndE,
                StE("", "line", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_ns2() {
        test_string(
            "#svg ##box{ xmlns='https://fred' b='2' ##box} ##line ",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE(
                    "https://fred",
                    "box",
                    &[
                        ("http://www.w3.org/2000/xmlns/", "xmlns", "https://fred"),
                        ("https://fred", "b", "2"),
                    ],
                ),
                EndE,
                StE("", "line", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_ns3() {
        test_string(
            "#svg ##box{ xmlns:blob='https://fred' b='2' ##box} ##line ",
            &[
                StD(100),
                StE("", "svg", &[]),
                StE(
                    "",
                    "box",
                    &[
                        ("http://www.w3.org/2000/xmlns/", "blob", "https://fred"),
                        ("", "b", "2"),
                    ],
                ),
                EndE,
                StE("", "line", &[]),
                EndE,
                EndE,
                EndD,
            ],
        );
    }
    //a Content tests
    #[test]
    fn test_content0() {
        test_string(
            r###"#svg "banana" "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Interpretable, "banana"),
                EndE,
                EndD,
            ],
        );
    }
    #[test]
    fn test_content1() {
        test_string(
            r###"#svg ##"banana"## "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Interpretable, "banana"),
                EndE,
                EndD,
            ],
        );
        test_string(
            r###"#svg ##"banana
"## "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Interpretable, "banana\n"),
                EndE,
                EndD,
            ],
        );
        test_string(
            r###"#svg ##'banana
'## "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Interpretable, "banana\n"),
                EndE,
                EndD,
            ],
        );
        test_string(
            r###"#svg r'banana' "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Raw, "banana"),
                EndE,
                EndD,
            ],
        );
        test_string(
            r###"#svg r#'banana'# "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Raw, "banana"),
                EndE,
                EndD,
            ],
        );
        test_string(
            r###"#svg r#'banana
'# "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Raw, "banana\n"),
                EndE,
                EndD,
            ],
        );
        test_string(
            r###"#svg r##'banana'#' '## "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Raw, "banana'#' "),
                EndE,
                EndD,
            ],
        );
        test_string(
            r###"#svg r##'banana'#''## "###,
            &[
                StD(100),
                StE("", "svg", &[]),
                Content(ContentType::Raw, "banana'#'"),
                EndE,
                EndD,
            ],
        );
    }
}
