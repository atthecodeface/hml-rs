//a Imports
use lexer_rs::{PosnInCharStream, StreamCharSpan};

use super::{CloseTag, OpenTag, StackElement, Token, TokenType};
use crate::markup::{ContentType, Event};
use crate::names::NamespaceStack;
use crate::reader::{Reader, ReaderError};

//a Internal types
//ti TagExtra
#[derive(Debug)]
struct TagExtra {
    depth: usize,
    boxed: bool,
}
impl TagExtra {
    fn new(depth: usize, boxed: bool) -> Self {
        Self { depth, boxed }
    }
}

//a Public types: Parser and TokenFn
//tp Parser
/// A parser, using a file position provided
///
pub struct Parser<R: Reader> {
    version: usize,
    pending_eof: bool,
    start_emitted: bool,
    end_emitted: bool,
    finished: bool,
    tag_depth: usize,
    tag_stack: Vec<StackElement<R, TagExtra>>,
    pending_open_tag: Option<OpenTag<R::Position, TagExtra>>,
    pending_close_tag: Option<CloseTag<R::Position, TagExtra>>,
    pending_token: Option<Token<R::Position>>,
    start_element_building: bool,
    token_pos: R::Position,
}

// These only work for R:Reader but Rust cannot handle that cleanly yet in the type itself
type Result<R, T> = crate::reader::Result<T, <R as Reader>::Position, <R as Reader>::Error>;

/// crate::reader::Result<T, P, E> = std::result::Result<T, ReaderError<P, E>>;
pub type EventResult<R> = std::result::Result<
    Event<<R as Reader>::Position>,
    ReaderError<<R as Reader>::Position, <R as Reader>::Error>,
>;
pub type OptEventResult<R> = std::result::Result<
    Option<Event<<R as Reader>::Position>>,
    ReaderError<<R as Reader>::Position, <R as Reader>::Error>,
>;

//ip Default for Parser
impl<R: Reader> Default for Parser<R> {
    fn default() -> Self {
        Parser {
            version: 100,
            start_emitted: false,
            end_emitted: false,
            finished: false,
            tag_depth: 0,
            tag_stack: Vec::new(),
            pending_eof: false,
            pending_open_tag: None,
            pending_close_tag: None,
            pending_token: None,
            start_element_building: false,
            token_pos: R::Position::default(),
        }
    }
}

//ip Parser
impl<R: Reader> Parser<R> {
    //mp set_version
    /// Set the target XML version number - 100 for 1.00, or 110 for
    /// 1.10
    #[inline]
    pub fn set_version(mut self, version: usize) -> Self {
        self.version = version;
        self
    }

    //mi pop_tag_stack
    /// Pops the tag stack and returns an Event of an end of that element
    fn pop_tag_stack(
        &mut self,
        ns_stack: &mut NamespaceStack,
        span: &StreamCharSpan<R::Position>,
    ) -> OptEventResult<R> {
        assert!(!self.tag_stack.is_empty());
        let (e, depth) = self.tag_stack.pop().unwrap().as_end_element(ns_stack, span);
        self.tag_depth = depth;
        Ok(Some(e))
    }

    //mi handle_pending_eof
    fn handle_pending_eof(&mut self, ns_stack: &mut NamespaceStack) -> OptEventResult<R> {
        if self.tag_stack.is_empty() {
            self.end_emitted = true;
            Ok(None)
        } else {
            let span = StreamCharSpan::new_at(&self.token_pos);
            self.pop_tag_stack(ns_stack, &span)
        }
    }

    //mi handle_close_tag
    /// A close tag closes all elements whose tag depth is > 0
    ///
    /// If the tag depth is 0 then the close tag should match the top of the tag stack
    fn handle_close_tag(
        &mut self,
        ns_stack: &mut NamespaceStack,
        close_tag: CloseTag<R::Position, TagExtra>,
    ) -> OptEventResult<R> {
        // If there are tags that are close the current element at the top of the stack
        if self.tag_depth > 0 {
            let span = StreamCharSpan::new_at(close_tag.span().start());
            self.pending_close_tag = Some(close_tag);
            self.pop_tag_stack(ns_stack, &span)
        } else {
            // should validate close_tag matches the StackElement at the top of the tag stack
            self.pop_tag_stack(ns_stack, close_tag.span())
        }
    }

    //mi handle_open_tag
    /// If the OpenTag has a depth <= the current then close the top of the tag stack
    ///
    /// If the OpenTag has a depth == current+1 then open it up
    ///
    /// If the OpenTag has a depth > current+1 then it has too much depth
    fn handle_open_tag(
        &mut self,
        ns_stack: &mut NamespaceStack,
        open_tag: OpenTag<R::Position, TagExtra>,
    ) -> OptEventResult<R> {
        if open_tag.extra.depth <= self.tag_depth {
            let span = StreamCharSpan::new_at(open_tag.span().start());
            self.pending_open_tag = Some(open_tag);
            self.pop_tag_stack(ns_stack, &span)
        } else if open_tag.extra.depth == self.tag_depth + 1 {
            // open the new element
            let boxed = open_tag.extra.boxed;
            self.tag_stack
                .push(StackElement::new(ns_stack, self.tag_depth, open_tag));
            self.start_element_building = true;
            self.tag_depth += 1;
            if boxed {
                self.tag_depth = 0;
            }
            Ok(None)
        } else {
            // tag with too much depth
            ReaderError::unexpected_tag_indent(*open_tag.span(), self.tag_depth + 1)
        }
    }

    //mi handle_token
    fn handle_token(
        &mut self,
        ns_stack: &mut NamespaceStack,
        mut token: Token<R::Position>,
    ) -> OptEventResult<R> {
        if self.start_element_building && !token.is_attribute() {
            self.start_element_building = false;
            self.pending_token = Some(token);
            Ok(Some(
                self.tag_stack
                    .last_mut()
                    .unwrap()
                    .as_start_element(ns_stack)?,
            ))
        } else {
            self.token_pos = *token.get_span().end();
            match token.token_type() {
                TokenType::Comment => {
                    let mut lengths = Vec::new();
                    let mut s = String::new();
                    for (i, c) in token.take_contents().into_iter().enumerate() {
                        lengths.push(c.len());
                        if i > 0 {
                            s.push('\n');
                        }
                        s += &c;
                    }
                    Ok(Some(Event::comment(*token.get_span(), s, lengths)))
                }
                TokenType::TagOpen => {
                    let span = *token.get_span();
                    let mut args = token.take_contents();
                    let prefix = args.pop_front().unwrap();
                    let name = args.pop_front().unwrap();
                    self.pending_open_tag = Some(OpenTag::new(
                        span,
                        prefix,
                        name,
                        TagExtra::new(token.get_depth(), token.get_boxed()),
                    ));
                    Ok(None)
                }
                TokenType::TagClose => {
                    let span = *token.get_span();
                    let mut args = token.take_contents();
                    let prefix = args.pop_front().unwrap();
                    let name = args.pop_front().unwrap();
                    let close_tag = ReaderError::of_markup_result(
                        span,
                        CloseTag::new(
                            span,
                            ns_stack,
                            &prefix,
                            &name,
                            TagExtra::new(token.get_depth(), false),
                        ),
                    )?;
                    self.pending_close_tag = Some(close_tag);
                    Ok(None)
                }
                TokenType::Attribute => {
                    let span = *token.get_span();
                    let mut args = token.take_contents();
                    let prefix = args.pop_front().unwrap();
                    let name = args.pop_front().unwrap();
                    let value = args.pop_front().unwrap();
                    if self.start_element_building {
                        self.tag_stack
                            .last_mut()
                            .unwrap()
                            .add_attribute(span, ns_stack, &prefix, &name, value)?;
                        Ok(None)
                    } else {
                        ReaderError::unexpected_attribute(span, &prefix, &name)
                    }
                }
                TokenType::Characters => {
                    let mut data = token.take_contents();
                    let data = data.pop_front().unwrap();
                    Ok(Some(Event::content(
                        *token.get_span(),
                        ContentType::Interpretable,
                        data,
                    )))
                }
                TokenType::RawCharacters => {
                    let mut data = token.take_contents();
                    let data = data.pop_front().unwrap();
                    Ok(Some(Event::content(
                        *token.get_span(),
                        ContentType::Raw,
                        data,
                    )))
                }
                TokenType::Whitespace => Ok(None),
                TokenType::EndOfFile => {
                    self.pending_eof = true;
                    Ok(None)
                }
            }
        }
    }

    //mp next_event
    /// next_event
    pub fn next_event<T>(
        &mut self,
        ns_stack: &mut NamespaceStack,
        mut get_token: T,
    ) -> EventResult<R>
    where
        T: FnMut() -> Result<R, Token<R::Position>>,
    {
        loop {
            if !self.start_emitted {
                self.start_emitted = true;
                let span = StreamCharSpan::new_at(&self.token_pos);
                return Ok(Event::start_document(span, self.version));
            } else if self.finished {
                return ReaderError::no_more_events();
            } else if self.end_emitted {
                self.finished = true;
                let span = StreamCharSpan::new_at(&self.token_pos);
                return Ok(Event::end_document(span));
            }
            if let Some(event) = {
                if self.pending_eof {
                    self.handle_pending_eof(ns_stack)
                } else if let Some(close_tag) = self.pending_close_tag.take() {
                    self.handle_close_tag(ns_stack, close_tag)
                } else if let Some(open_tag) = self.pending_open_tag.take() {
                    self.handle_open_tag(ns_stack, open_tag)
                } else if let Some(token) = self.pending_token.take() {
                    self.handle_token(ns_stack, token)
                } else {
                    let token = get_token()?;
                    self.handle_token(ns_stack, token)
                }
            }? {
                return Ok(event);
            }
        }
    }
}
