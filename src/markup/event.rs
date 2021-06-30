use crate::{StreamSpan, Tag, Name, NSNameId};

//tp ContentType
#[derive(Clone, Debug, PartialEq)]
pub enum ContentType {
    // CData should perhaps be Vec<u8>
    CData,
    Interpretable,
    Whitespace,
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

    //mp is_end_document
    pub fn is_end_document(&self) -> bool {
        match self { Self::EndDocument{ .. } => true, _ => false }
    }
}

