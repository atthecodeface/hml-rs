/*a Copyright

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

@file    parser.rs
@brief   HML parser, part of the HML reader using its Lexer
 */

//a Imports
use crate::{Tag, Name, Attributes, Event, NamespaceStack};
use crate::{Reader, ReaderPosition};
use super::{Token, TokenType, Span, Error, Result};

//a Internal types
//tp OpenTag
#[derive(Copy, Clone, Debug)]
struct OpenTag<P:ReaderPosition> {
    span : Span<P>,
    name : Name,
    depth: usize,
    boxed : bool,
}

impl <P:ReaderPosition> OpenTag<P> {
    pub fn new(span:Span<P>, name:Name, depth:usize, boxed:bool) -> Self {
        Self { span, name, depth, boxed }
    }
}
//tp CloseTag
#[derive(Copy, Clone, Debug)]
struct CloseTag<P:ReaderPosition> {
    span : Span<P>,
    name : Name,
    depth: usize,
}

impl <P:ReaderPosition> CloseTag<P> {
    pub fn new(span:Span<P>, name:Name, depth:usize) -> Self {
        Self { span, name, depth }
    }
}

//ti StackElement
/// A [StackElement] is used to build and return elements as a document is parsed.
///
/// The [StackElement] is created when the opening tag is seen; the
/// tag's attributes are added to as the tag is built, and upon
/// completion of the tag being built a 'StartElement' event can be
/// issued.
///
/// The [StartElement] remains on the top of the stack as the content of the element is handled.
///
/// Note that the namespace of the [StackElement] is a new frame in
/// the namespace stack, and the resolution of the attributes *AND*
/// tag name for the element are within this stack frame; any
/// namespace attributes for the element are added to the frame
/// *BEFORE* resolution of the names for the element and its
/// non-namespace attributes
///
/// When the content completes an [EndElement] can be issued
#[derive(Debug)]
struct StackElement <P:ReaderPosition> {
    parent_depth : usize,
    open_tag     : OpenTag<P>,
    attributes   : Attributes,
}

//ii StackElement
impl <P:ReaderPosition> StackElement<P> {
    pub fn new(parent_depth:usize,
               open_tag:OpenTag<P>
    ) -> Self {
        let attributes = Attributes::new();
        StackElement {
            parent_depth, open_tag, attributes
        }
    }
    pub fn as_start_element(&mut self) -> Event<Span<P>> {
        let attributes = std::mem::replace(&mut self.attributes, Attributes::new());
        let tag = Tag::new(self.open_tag.name, attributes);
        Event::start_element( self.open_tag.span, tag )
    }
    pub fn as_end_element(&self, span:Span<P>) -> (Event<Span<P>>, usize) {
        ( Event::end_element( span, self.open_tag.name ),
          self.parent_depth )
    }
}

//a Public types: Parser and TokenFn
//tp Parser
/// A parser, using a file position provided
///
pub struct Parser <R:Reader>{
    pending_eof       : bool,
    start_emitted     : bool,
    end_emitted       : bool,
    finished          : bool,
    tag_depth         : usize,
    tag_stack         : Vec<StackElement<R::Position>>,
    pending_open_tag  : Option<OpenTag<R::Position>>,
    pending_close_tag : Option<CloseTag<R::Position>>,
    pending_token     : Option<Token<R::Position>>,
    start_element_building : bool,
    token_pos       : R::Position,
}

// These only work for R:Reader but Rust cannot handle that cleanly yet in the type itself
pub type EventResult<R>    = Result<R, Event<Span<<R as Reader>::Position>>>;
pub type OptEventResult<R> = Result<R, Option<Event<Span<<R as Reader>::Position>>>>;

//ip Parser
impl <R:Reader> Parser<R> {

    //fp new
    /// Returns a new lexer with default state.
    pub fn new() -> Self {
        Parser {
            start_emitted: false,
            end_emitted: false,
            finished: false,
            tag_depth : 0,
            tag_stack : Vec::new(),
            pending_eof : false,
            pending_open_tag  : None,
            pending_close_tag : None,
            pending_token     : None,
            start_element_building : false,
            token_pos              : R::Position::none(),
        }
    }

    //mi pop_tag_stack
    /// Pops the tag stack and returns an Event of an end of that element
    fn pop_tag_stack(&mut self, ns_stack: &mut NamespaceStack, span:Span<R::Position>) -> OptEventResult<R> {
        assert!(self.tag_stack.len()>0);
        ns_stack.pop_frame();
        let (e, depth) = self.tag_stack.pop().unwrap().as_end_element(span);
        self.tag_depth = depth;
        Ok(Some(e))
    }

    //mi handle_pending_eof
    fn handle_pending_eof(&mut self, ns_stack: &mut NamespaceStack) -> OptEventResult<R> {
        if self.tag_stack.len()>0 {
            let span = Span::new_at(&self.token_pos);
            self.pop_tag_stack(ns_stack, span)
        } else {
            self.end_emitted = true;
            Ok(None)
        }
    }

    //mi handle_close_tag
    /// A close tag closes all elements whose tag depth is > 0
    ///
    /// If the tag depth is 0 then the close tag should match the top of the tag stack
    fn handle_close_tag(&mut self, ns_stack: &mut NamespaceStack, close_tag:CloseTag<R::Position>) -> OptEventResult<R> {
        // If there are tags that are close the current element at the top of the stack
        if self.tag_depth > 0 {
            let span = Span::new_at(close_tag.span.start());
            self.pending_close_tag = Some(close_tag);
            self.pop_tag_stack(ns_stack, span)
        } else {
            // should validate close_tag matches the StackElement at the top of the tag stack
            self.pop_tag_stack(ns_stack, close_tag.span)
        }
    }

    //mi handle_open_tag
    /// If the OpenTag has a depth <= the current then close the top of the tag stack
    ///
    /// If the OpenTag has a depth == current+1 then open it up
    ///
    /// If the OpenTag has a depth > current+1 then it has too much depth
    fn handle_open_tag(&mut self, ns_stack: &mut NamespaceStack, open_tag:OpenTag<R::Position>) -> OptEventResult<R> {
        if open_tag.depth <= self.tag_depth {
            let span = Span::new_at(open_tag.span.start());
            self.pending_open_tag = Some(open_tag);
            self.pop_tag_stack(ns_stack, span)
        } else if open_tag.depth == self.tag_depth+1 { // open the new element
            self.tag_stack.push(StackElement::new(self.tag_depth, open_tag));
            ns_stack.push_frame();
            self.start_element_building = true;
            self.tag_depth += 1;
            if open_tag.boxed {
                self.tag_depth = 0;
            }
            Ok(None)
        } else { // tag with too much depth
            Error::unexpected_tag_indent(open_tag.span, self.tag_depth+1)
        }
    }

    //mi handle_token
    fn handle_token(&mut self, ns_stack: &mut NamespaceStack, mut token:Token<R::Position>) -> OptEventResult<R> {
        if self.start_element_building && !token.is_attribute() {
            self.start_element_building = false;
            self.pending_token = Some(token);
            Ok(Some(self.tag_stack.last_mut().unwrap().as_start_element()))
        } else {
            self.token_pos = *token.get_span().end();
            match token.token_type() {
                TokenType::Comment => {
                    let content = token.take_contents();
                    Ok(Some(Event::comment(*token.get_span(), content)))
                },
                TokenType::TagOpen =>  {
                    let span = *token.get_span();
                    let args = token.take_contents();
                    let name = Name::new(ns_stack, &args[0], &args[1]).map_err(|r| Error::of_hml_error(span,r))?;
                    self.pending_open_tag = Some(OpenTag::new(span, name, token.get_depth(), token.get_boxed()));
                    Ok(None)
                },
                TokenType::TagClose  => {
                    let span = *token.get_span();
                    let args = token.take_contents();
                    let name = Name::new(ns_stack, &args[0], &args[1]).map_err(|r| Error::of_hml_error(span,r))?;
                    self.pending_close_tag = Some(CloseTag::new(span, name, token.get_depth()));
                    Ok(None)
                },
                TokenType::Attribute  => {
                    let span = *token.get_span();
                    let args = token.take_contents();
                    let name = Name::new(ns_stack, &args[0], &args[1]).map_err(|r| Error::of_hml_error(span,r))?;
                    let value = &args[2];
                    if self.start_element_building {
                        self.tag_stack.last_mut().unwrap().attributes.add(name, value);
                        Ok(None)
                    } else {
                        Error::unexpected_attribute(span, name)
                    }
                },
                TokenType::Characters  => {
                    let mut data = token.take_contents();
                    let data = data.remove(0);
                    Ok(Some(Event::content(*token.get_span(), data)))
                },
                TokenType::EndOfFile  => {
                    self.pending_eof = true;
                    Ok(None)
                },
            }
        }
    }

    //mp next_event
    /// next_event
    pub fn next_event<T> (&mut self, ns_stack: &mut NamespaceStack, mut get_token:T) -> EventResult<R>
        where T: FnMut () -> Result<R, Token<R::Position>>
    {
        loop {
            if !self.start_emitted {
                self.start_emitted = true;
                let span = Span::new_at(&self.token_pos);
                return Ok(Event::start_document(span, 100));
            } else if self.finished {
                return Error::no_more_events();
            } else if self.end_emitted {
                self.finished = true;
                let span = Span::new_at(&self.token_pos);
                return Ok(Event::end_document(span));
            } else {
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
}

