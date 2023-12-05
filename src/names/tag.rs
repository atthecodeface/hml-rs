//a Imports
use super::{Attributes, Name, NamespaceStack};
use crate::MarkupResult;

//a Tag
//tp Tag
/// A markup tag consists of a name (possibly within a namespace) and
/// a list of attributes (which are name/value pairs)
#[derive(Debug)]
pub struct Tag {
    /// Name with prefix *and URI from namespace stack*
    ///
    /// Note that the Name depends on the Namespace attributes within the tag
    pub name: Name,

    /// Attributes for the tag, including any local namespaces for the tag
    pub attributes: Attributes,
}

//ip Tag
impl Tag {
    //fp new
    /// Create a new [Tag] from a namespace and name strings, and an
    /// attribute list
    pub fn new(
        ns_stack: &mut NamespaceStack,
        ns: &str,
        name: &str,
        attributes: Attributes,
    ) -> MarkupResult<Self> {
        let name = Name::new(ns_stack, ns, name)?;
        Ok(Self { name, attributes })
    }
}
