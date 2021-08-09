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

@file    event.rs
@brief   Markup events
 */

//a Imports
use crate::markup::Span;
use crate::names::{NSNameId, Name, Tag};

//a Content
//tp ContentType
/// The type of the content; it may be raw or in need of expansion, or
/// it may be whitespace (if the source is XML)
///
/// Raw data is a Unicode string; in XML this corresponds to a CDATA
/// section, in HML a raw string
///
/// Whitespace is space, tab and newline characters only; it comes
/// from XML source, and if whitespace is deemed important for the
/// application (as XML allows) then it is provided as such here.  It
/// may be used in XML output, in which case it generates the same
/// whitespace; in HML output it is turned in to plain content with
/// escape sequences for tabs and newlines.
///
/// Other content needs interpretation by the event provider, unless
/// it is to be handled unparsed by the application (this is an
/// application choice)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContentType {
    /// Must change to Raw
    Raw,
    /// Content can be intepreted (escapes/entitied converted to characters/strings)
    Interpretable,
    /// The content is whitespace that contains only tab, space or newlines (and hence need not be interpreted)
    Whitespace,
}

//a Event
//tp Event
/// A markup event
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventType {
    /// The start of a document - issued before any other events
    StartDocument,
    /// The end of a document - issued after all other events; all
    /// elements must be closed, and no more events will be returned
    EndDocument,
    /// The start of an element: this is always paired with an [EndElement] event
    StartElement,
    /// The end of an element
    EndElement,
    /// One [String] of content for an element; will always be within an element
    Content,
    /// A processing instruction
    ProcessingInstruction,
    /// A comment consisting a a Vec of Strings, one per line of the comment
    Comment,
}

//tp Event
/// A markup event occupying a [Span] on a stream
#[derive(Debug)]
pub enum Event<F: Span> {
    /// The start of the document
    StartDocument {
        /// File position of start of the document
        span: F,
        /// Version as an integer - 100 for 1.00, etc
        version: usize,
    },

    /// The end of the document
    EndDocument {
        /// File position of end of the document
        span: F,
    },

    /// Denotes a beginning of an XML element.
    StartElement {
        /// The span of the start element 'tag'
        span: F,
        /// The actual tag (prefix, URI, name, attributes)
        tag: Tag,
    },

    /// Denotes an end of an XML element.
    EndElement {
        /// The span of the end element 'tag'
        span: F,
        /// The (prefix, URI, name) of the element (equal to the same
        /// value as the StartElement that this closes)
        name: Name,
    },

    /// Denotes one part of the content of an element
    Content {
        /// The span of the content
        span: F,
        /// The type of the content: raw, whitespace, needs unescaping
        ctype: ContentType,
        /// The string content
        data: String,
    },

    /// Denotes an XML processing instruction.
    ProcessingInstruction {
        /// The span of the PI
        span: F,
        /// A NSNameId within the namespace that is the name of the processing instruction
        name: NSNameId,
        /// An optional value for the processing instruction
        data: Option<String>,
    },

    /// Denotes a comment.
    Comment {
        /// The span of the comment
        span: F,
        /// One string per line of the comment
        data: Vec<String>,
    },
}

//ip Event
impl<F: Span> Event<F> {
    //fp start_document
    /// Create a StartDocument event
    pub fn start_document(span: F, version: usize) -> Self {
        Self::StartDocument { span, version }
    }

    //fp end_document
    /// Create an EndDocument event
    pub fn end_document(span: F) -> Self {
        Self::EndDocument { span }
    }

    //fp start_element
    /// Create a StartElement event with a given [Tag]
    pub fn start_element(span: F, tag: Tag) -> Self {
        Self::StartElement { span, tag }
    }

    //fp end_element
    /// Create an EndElement event with a given [Name]
    pub fn end_element(span: F, name: Name) -> Self {
        Self::EndElement { span, name }
    }

    //fp comment
    /// Create an event of a vec of comment strings
    pub fn comment(span: F, data: Vec<String>) -> Self {
        Self::Comment { span, data }
    }

    //fp content
    /// Create an event of content of the given type
    pub fn content(span: F, ctype: ContentType, data: String) -> Self {
        Self::Content { span, ctype, data }
    }

    //fp content_raw
    /// Create an event of raw content
    pub fn content_raw(span: F, data: String) -> Self {
        Self::content(span, ContentType::Raw, data)
    }

    //fp content_int
    /// Create an event of interpretable content
    pub fn content_int(span: F, data: String) -> Self {
        Self::content(span, ContentType::Interpretable, data)
    }

    //fp content_ws
    /// Create an event of whitespace content
    pub fn content_ws(span: F, data: String) -> Self {
        Self::content(span, ContentType::Whitespace, data)
    }

    //mp get_type
    /// Get the [EventType] corresponding to the [Event]
    pub fn get_type(&self) -> EventType {
        match self {
            Self::StartDocument { .. } => EventType::StartDocument,
            Self::EndDocument { .. } => EventType::EndDocument,
            Self::StartElement { .. } => EventType::StartElement,
            Self::EndElement { .. } => EventType::EndElement,
            Self::Content { .. } => EventType::Content,
            Self::ProcessingInstruction { .. } => EventType::ProcessingInstruction,
            Self::Comment { .. } => EventType::Comment,
        }
    }

    //mp borrow_span
    /// Borrow the span of the event, for logging or errors etc.
    pub fn borrow_span(&self) -> &F {
        match self {
            Self::StartDocument { span, .. } => span,
            Self::EndDocument { span, .. } => span,
            Self::StartElement { span, .. } => span,
            Self::EndElement { span, .. } => span,
            Self::Content { span, .. } => span,
            Self::ProcessingInstruction { span, .. } => span,
            Self::Comment { span, .. } => span,
        }
    }

    //mp as_start_document
    /// Return Some(version number) if the [Event] is a StartDocument
    /// event; else return None
    pub fn as_start_document(&self) -> Option<usize> {
        match self {
            Self::StartDocument { version, .. } => Some(*version),
            _ => None,
        }
    }

    //mp as_start_element
    /// Return Some(Tag) if the [Event] is a StartElement
    /// event; else return None
    pub fn as_start_element(self) -> Option<Tag> {
        match self {
            Self::StartElement { tag, .. } => Some(tag),
            _ => None,
        }
    }

    //mp as_end_element
    /// Return Some(Name) if the [Event] is an EndElement
    /// event; else return None
    pub fn as_end_element(&self) -> Option<&Name> {
        match self {
            Self::EndElement { name, .. } => Some(name),
            _ => None,
        }
    }

    //mp as_content
    /// Return Some(ContentType, string) if the [Event] is a Content
    /// event; else return None
    pub fn as_content(&self) -> Option<(ContentType, &str)> {
        match self {
            Self::Content { ctype, data, .. } => Some((*ctype, data)),
            _ => None,
        }
    }

    //mp is_start_document
    /// Return true if the Event is a StartDocument event
    pub fn is_start_document(&self) -> bool {
        match self {
            Self::StartDocument { .. } => true,
            _ => false,
        }
    }

    //mp is_end_document
    /// Return true if the Event is an EndDocument event
    pub fn is_end_document(&self) -> bool {
        match self {
            Self::EndDocument { .. } => true,
            _ => false,
        }
    }
}
