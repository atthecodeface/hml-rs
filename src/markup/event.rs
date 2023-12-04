//a Imports
use crate::names::{NSNameId, Name, NamespaceStack, Tag};
use lexer_rs::{PosnInCharStream, StreamCharSpan};

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
    /// Content can be intepreted (escapes/entities converted to characters/strings)
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
    /// A comment consisting of a String, with new lines *between* comment lines
    ///
    /// There is no trailing newline unless the last line was blank
    /// (in which case that new line separates the last-but-one-line
    /// from an empty last line)
    Comment,
}

//tp Event
/// A markup event occupying a [Span] on a stream
#[derive(Debug)]
pub enum Event<P>
where
    P: PosnInCharStream,
{
    /// The start of the document
    StartDocument {
        /// File position of start of the document
        span: StreamCharSpan<P>,
        /// Version as an integer - 100 for 1.00, etc
        version: usize,
    },

    /// The end of the document
    EndDocument {
        /// File position of end of the document
        span: StreamCharSpan<P>,
    },

    /// Denotes a beginning of an XML element.
    StartElement {
        /// The span of the start element 'tag'
        span: StreamCharSpan<P>,
        /// The actual tag (prefix, URI, name, attributes)
        tag: Tag,
    },

    /// Denotes an end of an XML element.
    EndElement {
        /// The span of the end element 'tag'
        span: StreamCharSpan<P>,
        /// The (prefix, URI, name) of the element (equal to the same
        /// value as the StartElement that this closes)
        name: Name,
    },

    /// Denotes one part of the content of an element
    Content {
        /// The span of the content
        span: StreamCharSpan<P>,
        /// The type of the content: raw, whitespace, needs unescaping
        ctype: ContentType,
        /// The string content
        data: String,
    },

    /// Denotes an XML processing instruction.
    ProcessingInstruction {
        /// The span of the PI
        span: StreamCharSpan<P>,
        /// A NSNameId within the namespace that is the name of the processing instruction
        name: NSNameId,
        /// An optional value for the processing instruction
        data: Option<String>,
    },

    /// Denotes a comment.
    Comment {
        /// The span of the comment
        span: StreamCharSpan<P>,
        /// One string containing *all* the lines of comment (separated by \n)
        ///
        /// The last line does not have \n appended, so single line comments have no newline
        data: String,
        /// Length of each original comment line (not including any additional \n)
        lengths: Vec<usize>,
    },
}

//ip Event
impl<P> Event<P>
where
    P: PosnInCharStream,
{
    //fp start_document
    /// Create a StartDocument event
    pub fn start_document(span: StreamCharSpan<P>, version: usize) -> Self {
        Self::StartDocument { span, version }
    }

    //fp end_document
    /// Create an EndDocument event
    pub fn end_document(span: StreamCharSpan<P>) -> Self {
        Self::EndDocument { span }
    }

    //fp start_element
    /// Create a StartElement event with a given [Tag]
    pub fn start_element(span: StreamCharSpan<P>, tag: Tag) -> Self {
        Self::StartElement { span, tag }
    }

    //fp end_element
    /// Create an EndElement event with a given [Name]
    pub fn end_element(span: StreamCharSpan<P>, name: Name) -> Self {
        Self::EndElement { span, name }
    }

    //fp comment
    /// Create an event of a vec of comment strings
    pub fn comment(span: StreamCharSpan<P>, data: String, lengths: Vec<usize>) -> Self {
        Self::Comment {
            span,
            data,
            lengths,
        }
    }

    //fp content
    /// Create an event of content of the given type
    pub fn content(span: StreamCharSpan<P>, ctype: ContentType, data: String) -> Self {
        Self::Content { span, ctype, data }
    }

    //fp content_raw
    /// Create an event of raw content
    pub fn content_raw(span: StreamCharSpan<P>, data: String) -> Self {
        Self::content(span, ContentType::Raw, data)
    }

    //fp content_int
    /// Create an event of interpretable content
    pub fn content_int(span: StreamCharSpan<P>, data: String) -> Self {
        Self::content(span, ContentType::Interpretable, data)
    }

    //fp content_ws
    /// Create an event of whitespace content
    pub fn content_ws(span: StreamCharSpan<P>, data: String) -> Self {
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
    pub fn borrow_span(&self) -> &StreamCharSpan<P> {
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
        matches!(self, Self::StartDocument { .. })
    }

    //mp is_end_document
    /// Return true if the Event is an EndDocument event
    pub fn is_end_document(&self) -> bool {
        matches!(self, Self::EndDocument { .. })
    }
}

//a If xml_rs is included
#[cfg(feature = "xml")]
//ip Event
impl<P> Event<P>
where
    P: PosnInCharStream,
{
    //mp as_xml_writer
    /// Get an [xml::writer::XmlEvent<'a>] from this Name
    pub fn as_xml_writer<'a>(
        &'a self,
        ns: &'a NamespaceStack,
    ) -> Option<xml::writer::XmlEvent<'a>> {
        use Event::*;
        match self {
            StartDocument { version, .. } => {
                let version = if *version == 100 {
                    xml::common::XmlVersion::Version10
                } else {
                    xml::common::XmlVersion::Version11
                };
                Some(xml::writer::XmlEvent::StartDocument {
                    version,
                    encoding: None,
                    standalone: None,
                })
            }
            StartElement { tag, .. } => {
                let name = tag.name.as_xml_name(ns);
                let mut x = xml::writer::XmlEvent::start_element(name);
                for a in tag.attributes.attributes() {
                    let attr_name = a.name.as_xml_name(ns);
                    x = x.attr(attr_name, &a.value);
                }
                Some(x.into())
            }
            EndElement { .. } => Some(xml::writer::XmlEvent::end_element().into()),
            EndDocument { .. } => None,
            Content { data, .. } => Some(xml::writer::XmlEvent::characters(data)),
            ProcessingInstruction { name, data, .. } => {
                let name = ns.name_str(*name);
                Some(xml::writer::XmlEvent::processing_instruction(
                    name,
                    data.as_ref().map(|x| x.as_str()),
                ))
            }
            Comment { data, .. } => Some(xml::writer::XmlEvent::comment(data)),
        }
    }

    //mp as_xml_reader
    /// Get an [xml::reader::XmlEvent<'a>] from this Name
    pub fn as_xml_reader(
        &self,
        ns: &NamespaceStack,
        _fill_namespaces: bool,
    ) -> Option<xml::reader::XmlEvent> {
        use Event::*;
        match self {
            StartDocument { version, .. } => {
                let version = if *version == 100 {
                    xml::common::XmlVersion::Version10
                } else {
                    xml::common::XmlVersion::Version11
                };
                Some(xml::reader::XmlEvent::StartDocument {
                    version,
                    encoding: "UTF8".to_string(),
                    standalone: None,
                })
            }
            StartElement { tag, .. } => {
                let name = tag.name.as_xml_name(ns).to_owned();
                let namespace = xml::namespace::Namespace::empty();
                let mut attributes = Vec::new();
                for a in tag.attributes.attributes() {
                    let attr_name = a.name.as_xml_name(ns).to_owned();
                    attributes.push(xml::attribute::OwnedAttribute::new(attr_name, &a.value));
                }
                Some(xml::reader::XmlEvent::StartElement {
                    name,
                    namespace,
                    attributes,
                })
            }
            EndElement { name, .. } => Some(xml::reader::XmlEvent::EndElement {
                name: name.as_xml_name(ns).to_owned(),
            }),
            EndDocument { .. } => Some(xml::reader::XmlEvent::EndDocument),
            Content { data, .. } => Some(xml::reader::XmlEvent::Characters(data.clone())),
            ProcessingInstruction { name, data, .. } => {
                let name = ns.name_str(*name);
                Some(xml::reader::XmlEvent::ProcessingInstruction {
                    name: name.to_string(),
                    data: data.clone(),
                })
            }
            Comment { data, .. } => Some(xml::reader::XmlEvent::Comment(data.to_string())),
        }
    }
}
