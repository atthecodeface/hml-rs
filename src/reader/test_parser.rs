
//a Test
#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::{Namespace, NamespaceStack, Tag, Name};
    use crate::reader::{Parser, Lexer, ReaderError};
    use crate::string::Reader as StringReader;
    use crate::string::Position as StringPosition;
    use crate::string::Error as StringError;
    type Event = crate::Event<crate::reader::Span<StringPosition>>;
    #[derive(Debug)]
    enum Expectation<'a> {
        StD(usize),
        StE(&'a str, &'a str, &'a [(&'a str, &'a str, &'a str)]),
        EndE,
        EndD,
        Ignore,
    }
    struct ExpectationState<'a> {
        expectations : &'a [ Expectation<'a> ],
        index : usize,
        stack : Vec<Name>,
    }
    impl <'a> ExpectationState<'a> {
        fn new(expectations:&'a [Expectation <'a>]) -> Self {
            Self { expectations, index:0, stack:Vec::new() }
        }
        fn match_open(&mut self, ns_stack:&NamespaceStack, tag:Tag, uri:&str, name:&str, attrs:&[(&str, &str, &str)]) -> bool {
            if ns_stack.borrow_name(tag.name.name) != name {
                false
            } else if ns_stack.borrow_uri(tag.name.uri) != uri {
                false
            } else  {
                self.stack.push(tag.name);
                let tag_attrs = tag.attributes.take();
                if tag_attrs.len() != attrs.len() {
                    false
                } else {
                    for (e,a) in attrs.iter().zip(tag_attrs) {
                        println!("Check attrs {:?} {:?} {} {}",e,a, ns_stack.borrow_uri(a.name.uri), ns_stack.borrow_name(a.name.name));
                        if ns_stack.borrow_uri(a.name.uri)   != e.0   { return false; }
                        if ns_stack.borrow_name(a.name.name) != e.1 { return false; }
                        if a.value != e.2 { return false; }
                    }
                    true
                }
            }
        }
        fn match_close(&mut self, _ns_stack:&NamespaceStack, name:&Name ) -> bool {
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
        fn check_expectation(&mut self, ns_stack:&NamespaceStack, t:Result<Event, ReaderError<StringPosition, StringError>>) -> Result<(), String> {
            self.index += 1;
            if self.index > self.expectations.len() {
                return Err(format!("Ran out of expectations, got {:?}",t));
            }
            let (failure_string, pass) = {
                match self.expectations[self.index-1] {
                    Expectation::StD(v) => {
                        ( format!("Expected a StartDocument {}, got {:?}", v, t),
                          t.is_ok() && t.unwrap().as_start_document() == Some(v)
                        )
                    }
                    Expectation::EndD => {
                        ( format!("Expected an EndDocument, got {:?}", t),
                          t.is_ok() && t.unwrap().is_end_document()
                        )
                    },
                    Expectation::StE(ns,name,attrs) => {
                        ( format!("Expected a StartElement {}:{} with attrs {:?}, got {:?}", ns,name,attrs, t),
                          if t.is_err() {
                              false
                          } else {
                              let t = t.unwrap();
                              if let Some(tag) = t.as_start_element() {
                                  self.match_open(ns_stack, tag, ns, name, attrs)
                              } else {
                                  false
                              }
                          }
                        )
                    }
                    Expectation::EndE => {
                        ( format!("Expected an EndElement got {:?}", t),
                          if t.is_err() {
                              false
                          } else {
                              let t = t.unwrap();
                              if let Some(name) = t.as_end_element() {
                                  self.match_close(ns_stack, name)
                              } else {
                                  false
                              }
                          }
                        )
                    }
                    Expectation::Ignore => {
                        ( String::new(), true )
                    },
                }
            };
            if pass {
                Ok( () )
            } else {
                Err(failure_string)
            }
        }
    }

    fn test_string(s:&str, exp:&[Expectation]) {
        let mut expectation = ExpectationState::new(exp);
        let mut namespace = Namespace::new(true);
        let mut namespace_stack = NamespaceStack::new(&mut namespace);
        namespace_stack.add_null_ns();
        let mut reader = StringReader::new(s);
        let mut lexer  = Lexer::new();
        let mut parser : Parser::<StringReader>  = Parser::new();
        let mut errors = Vec::new();
        loop {
            let t = parser.next_event(&mut namespace_stack, || lexer.next_token(&mut reader));
            // println!("{:?}", t);
            let eof = t.is_ok() && t.as_ref().unwrap().is_end_document();
            match expectation.check_expectation(&namespace_stack, t) {
                Err(x) => { errors.push(x); },
                _ => (),
            }
            if eof { break ; }
        }
        for e in &errors {
            println!("FAIL: {}",e);
        }
        assert!(errors.is_empty());
    }
    #[test]
    fn test_blah() {
        use Expectation::*;
        test_string( "#svg ##line ##text",
                       &[StD(100),
                             StE("", "svg", &[]),
                             StE("", "line", &[]),
                             EndE,
                             StE("", "text", &[]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_blah2() {
        use Expectation::*;
        test_string( "#svg ##box{ ##box}",
                       &[StD(100),
                             StE("", "svg", &[]),
                             StE("", "box", &[]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_blah3() {
        use Expectation::*;
        test_string( "#svg ##box{ #line ##box}",
                       &[StD(100),
                             StE("", "svg", &[]),
                             StE("", "box", &[]),
                             StE("", "line", &[]),
                             EndE,
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_blah4() {
        use Expectation::*;
        test_string( "#svg ##box{ #line #line ##box}",
                       &[StD(100),
                             StE("", "svg", &[]),
                             StE("", "box", &[]),
                             StE("", "line", &[]),
                             EndE,
                             StE("", "line", &[]),
                             EndE,
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_blah5() {
        use Expectation::*;
        test_string( "#svg ##box{ #innerbox{ #line #innerbox} ##box}",
                       &[StD(100),
                             StE("", "svg", &[]),
                             StE("", "box", &[]),
                             StE("", "innerbox", &[]),
                             StE("", "line", &[]),
                             EndE,
                             EndE,
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_attr1() {
        use Expectation::*;
        test_string( "#svg a='1' ##line b='2'",
                       &[StD(100),
                             StE("", "svg",  &[ ("", "a", "1"), ]),
                             StE("", "line", &[ ("", "b", "2"), ]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_attr2() {
        use Expectation::*;
        test_string( "#svg a='1' b='2' ##line ",
                       &[StD(100),
                             StE("", "svg",  &[ ("", "a", "1"), ("", "b", "2"), ]),
                             StE("", "line", &[ ]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_attr3() {
        use Expectation::*;
        test_string( "#svg ##box{ a='1' b='2' ##box} ##line ",
                       &[StD(100),
                             StE("", "svg",  &[ ]),
                             StE("", "box",  &[ ("", "a", "1"), ("", "b", "2"), ]),
                             EndE,
                             StE("", "line", &[ ]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_ns() {
        use Expectation::*;
        test_string( "#svg ##box{ xmlns='https://fred' ##box} ##line ",
                       &[StD(100),
                             StE("", "svg",  &[ ]),
                             StE("https://fred", "box",  &[ ("http://www.w3.org/2000/xmlns/", "xmlns", "https://fred")]),
                             EndE,
                             StE("", "line", &[ ]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_ns2() {
        use Expectation::*;
        test_string( "#svg ##box{ xmlns='https://fred' b='2' ##box} ##line ",
                       &[StD(100),
                             StE("", "svg",  &[ ]),
                             StE("https://fred", "box",  &[ ("http://www.w3.org/2000/xmlns/", "xmlns", "https://fred"), ("https://fred", "b", "2"), ]),
                             EndE,
                             StE("", "line", &[ ]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
    #[test]
    fn test_ns3() {
        use Expectation::*;
        test_string( "#svg ##box{ xmlns:blob='https://fred' b='2' ##box} ##line ",
                       &[StD(100),
                             StE("", "svg",  &[ ]),
                             StE("", "box",  &[ ("http://www.w3.org/2000/xmlns/", "blob", "https://fred"), ("", "b", "2"), ]),
                             EndE,
                             StE("", "line", &[ ]),
                             EndE,
                             EndE,
                             EndD
                       ] );
    }
}

