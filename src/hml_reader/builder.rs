//a Imports
use crate::markup::Event;
use crate::names::{Attributes, Name, NamespaceStack, Tag};
use crate::{HmlError, HmlResult, Posn, Span};

//a Internal types
//tp OpenTag
#[derive(Clone, Debug)]
pub struct OpenTag<P, T>
where
    P: Posn,
    T: std::fmt::Debug,
{
    span: Span<P>,
    prefix: String,
    name: String,
    pub extra: T,
}

//ip OpenTag
impl<P, T> OpenTag<P, T>
where
    P: Posn,
    T: std::fmt::Debug,
{
    pub fn new(span: Span<P>, prefix: String, name: String, extra: T) -> Self {
        Self {
            span,
            prefix,
            name,
            extra,
        }
    }
    pub fn span(&self) -> &Span<P> {
        &self.span
    }
}

//tp CloseTag
#[derive(Clone, Debug)]
pub struct CloseTag<P, T>
where
    P: Posn,
    T: std::fmt::Debug,
{
    span: Span<P>,
    #[allow(dead_code)]
    name: Name,
    pub extra: T,
}

//ip CloseTag
impl<P, T> CloseTag<P, T>
where
    P: Posn,
    T: std::fmt::Debug,
{
    pub fn new(
        span: Span<P>,
        ns_stack: &mut NamespaceStack,
        prefix: &str,
        name: &str,
        extra: T,
    ) -> HmlResult<Self, P> {
        let name = HmlError::map_markup_error(Name::new(ns_stack, prefix, name), &span)?;
        Ok(Self { span, name, extra })
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
pub struct StackElement<P, T>
where
    P: Posn,
    T: std::fmt::Debug,
{
    parent_depth: usize,
    open_tag: OpenTag<P, T>,
    tag_name: Name,
    attributes: Attributes,
}

//ii StackElement
impl<P, T> StackElement<P, T>
where
    P: Posn,
    T: std::fmt::Debug,
{
    pub fn new(
        ns_stack: &mut NamespaceStack,
        parent_depth: usize,
        open_tag: OpenTag<P, T>,
    ) -> Self {
        ns_stack.push_frame();

        let attributes = Attributes::default();
        let tag_name = Name::none();
        StackElement {
            parent_depth,
            open_tag,
            tag_name,
            attributes,
        }
    }
    pub fn add_attribute(
        &mut self,
        span: Span<P>,
        ns_stack: &mut NamespaceStack,
        prefix: &str,
        name: &str,
        value: String,
    ) -> HmlResult<(), P> {
        HmlError::map_markup_error(self.attributes.add(ns_stack, prefix, name, value), &span)
    }

    pub fn as_start_element(&mut self, ns_stack: &mut NamespaceStack) -> HmlResult<Event<P>, P> {
        let attributes = std::mem::take(&mut self.attributes);
        let tag = HmlError::map_markup_error(
            Tag::new(
                ns_stack,
                &self.open_tag.prefix,
                &self.open_tag.name,
                attributes,
            ),
            &self.open_tag.span,
        )?;
        self.tag_name = tag.name;
        Ok(Event::start_element(self.open_tag.span, tag))
    }
    pub fn as_end_element(
        &self,
        ns_stack: &mut NamespaceStack,
        span: &Span<P>,
    ) -> (Event<P>, usize) {
        ns_stack.pop_frame();
        (Event::end_element(*span, self.tag_name), self.parent_depth)
    }
}
