use crate::{StreamSpan, Tag, Name, NSNameId};

//tp ContentType
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContentType {
    CData,
    Interpretable,
    Whitespace,
}

//tp Event
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EventType {
    StartDocument,
    EndDocument,
    StartElement,
    EndElement,
    Content,
    ProcessingInstruction,
    Comment,
}

//tp Event
#[derive(Debug)]
pub enum Event<F:StreamSpan> {
    /// The start of the document
    StartDocument {
        /// File position of start of the document
        span : F,
        /// Version as an integer - 100 for 1.00, etc
        version: usize,
    },

    EndDocument {
        /// File position of end of the document
        span : F,
    },

    /// Denotes a beginning of an XML element.
    StartElement {
        span       : F,
        tag        : Tag, // includes attributes
    },

    /// Denotes an end of an XML element.
    EndElement {
        span       : F,
        name       : Name,
    },

    /// Denotes one part of the content of an element
    Content {
        span    : F,
        ctype   : ContentType,
        data    : String,
    },

    /// Denotes an XML processing instruction.
    ProcessingInstruction {
        span     : F,
        name     : NSNameId,
        data     : Option<String>,
    },

    /// Denotes a comment.
    Comment {
        span     : F,
        data     : Vec<String>,
    },
}

//ip Event
impl <F:StreamSpan> Event<F> {
    //fp start_document
    pub fn start_document(span:F, version:usize) -> Self {
        Self::StartDocument { span, version }
    }
    //fp end_document
    pub fn end_document(span:F) -> Self {
        Self::EndDocument { span }
    }
    //fp start_element
    pub fn start_element(span:F, tag:Tag) -> Self {
        Self::StartElement { span, tag }
    }
    //fp end_element
    pub fn end_element(span:F, name:Name) -> Self {
        Self::EndElement { span, name }
    }
    //fp comment
    pub fn comment(span:F, data:Vec<String>) -> Self {
        Self::Comment { span, data }
    }
    //fp content
    pub fn content(span:F, data:String) -> Self {
        let ctype = ContentType::CData;
        Self::Content { span, ctype, data }
    }

    //mp get_type
    pub fn get_type(&self) -> EventType {
        match self {
            Self::StartDocument {..} => EventType::StartDocument,
            Self::EndDocument {..} => EventType::EndDocument,
            Self::StartElement {..} => EventType::StartElement,
            Self::EndElement {..} => EventType::EndElement,
            Self::Content {..} => EventType::Content,
            Self::ProcessingInstruction {..} => EventType::ProcessingInstruction,
            Self::Comment {..} => EventType::Comment,
        }
    }

    //mp borrow_span
    pub fn borrow_span(&self) -> &F {
        match self {
            Self::StartDocument {span,..} => span,
            Self::EndDocument {span,..}   => span,
            Self::StartElement {span,..}  => span,
            Self::EndElement {span,..}    => span,
            Self::Content {span,..}       => span,
            Self::ProcessingInstruction {span,..} => span,
            Self::Comment {span,..}       => span,
        }
    }

    //mp as_start_document
    pub fn as_start_document(&self) -> Option<usize> {
        match self { Self::StartDocument{ version, .. } => Some(*version), _ => None }
    }

    //mp as_start_element
    pub fn as_start_element(self) -> Option<Tag> {
        match self { Self::StartElement{ tag, .. } => Some(tag), _ => None }
    }

    //mp as_end_element
    pub fn as_end_element(&self) -> Option<&Name> {
        match self { Self::EndElement{ name, .. } => Some(name), _ => None }
    }

    //mp as_content
    pub fn as_content(&self) -> Option<(ContentType, &str)> {
        match self { Self::Content{ ctype, data, .. } => Some((*ctype, data)), _ => None }
    }

    //mp is_start_document
    pub fn is_start_document(&self) -> bool {
        match self { Self::StartDocument{ .. } => true, _ => false }
    }

    //mp is_end_document
    pub fn is_end_document(&self) -> bool {
        match self { Self::EndDocument{ .. } => true, _ => false }
    }
}

