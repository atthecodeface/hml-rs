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

@file    builder.rs
@brief   Markup builder for assisting parsers
 */

//a Imports
use crate::{MarkupResult, Tag, Name, Attributes, Event, NamespaceStack};
use crate::reader::{Reader, Position};
use super::{Span, Error};
type Result<R, T> = super::Result<T, <R as Reader>::Position, <R as Reader>::Error>;

//a Internal types
//tp OpenTag
#[derive(Clone, Debug)]
pub struct OpenTag<P:Position, T:std::fmt::Debug> {
    span : Span<P>,
    prefix : String,
    name   : String,
    pub extra : T,
}

//ip OpenTag
impl <P:Position, T:std::fmt::Debug> OpenTag<P, T> {
    pub fn new(span:Span<P>, prefix:String, name:String, extra:T) -> Self {
        Self { span, prefix, name, extra }
    }
    pub fn span(&self) -> &Span<P> {
        &self.span
    }
}

//tp CloseTag
#[derive(Clone, Debug)]
pub struct CloseTag<P:Position, T:std::fmt::Debug> {
    span : Span<P>,
    name : Name,
    pub extra : T,
}

//ip CloseTag
impl <P:Position, T:std::fmt::Debug> CloseTag<P, T> {
    pub fn new(span:Span<P>, ns_stack:&mut NamespaceStack, prefix:&str, name:&str, extra:T) -> MarkupResult<Self> {
        let name = Name::new(ns_stack, prefix, name)?;
        Ok ( Self { span, name, extra } )
    }
    pub fn span(&self) -> &Span<P> {
        &self.span
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
pub struct StackElement <R:Reader, T:std::fmt::Debug> {
    parent_depth : usize,
    open_tag     : OpenTag<R::Position, T>,
    tag_name     : Name,
    attributes   : Attributes,
}

//ii StackElement
impl <R:Reader, T:std::fmt::Debug> StackElement<R, T> {
    pub fn new(ns_stack: &mut NamespaceStack,
               parent_depth:usize,
               open_tag:OpenTag<R::Position, T>
    ) -> Self {
        ns_stack.push_frame();

        let attributes = Attributes::new();
        let tag_name = Name::none();
        StackElement {
            parent_depth, open_tag, tag_name, attributes
        }
    }
    pub fn add_attribute(&mut self, span:Span<R::Position>, ns_stack:&mut NamespaceStack, prefix:&str, name:&str, value:String) -> Result<R,()> {
        Error::of_markup_result(span, self.attributes.add(ns_stack, prefix, name, value))
    }
    pub fn as_start_element(&mut self, ns_stack:&mut NamespaceStack) -> Result<R, Event<Span<R::Position>>> {
        let attributes = std::mem::replace(&mut self.attributes, Attributes::new());
        let tag = Error::of_markup_result(self.open_tag.span, Tag::new(ns_stack, &self.open_tag.prefix, &self.open_tag.name, attributes))?;
        self.tag_name = tag.name;
        Ok(Event::start_element( self.open_tag.span, tag ))
    }
    pub fn as_end_element(&self, ns_stack:&mut NamespaceStack, span:&Span<R::Position>) -> (Event<Span<R::Position>>, usize) {
        ns_stack.pop_frame();
        ( Event::end_element( *span, self.tag_name ),
          self.parent_depth )
    }
}
