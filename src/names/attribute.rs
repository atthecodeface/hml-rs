//a Imports
use super::{Name, NamespaceStack};
use crate::MarkupResult;

//a Attribute
//tp Attribute
/// An [Attribute] has a [Name] and a [String] value.
///
/// They correspond to attributes in markup tags
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Attribute {
    /// Name and optional namespace
    pub name: Name,

    /// Attribute value.
    pub value: String,
}

impl Attribute {
    //fp new
    /// Create a new [Attribute] using the [NamespaceStack] to resolve the name
    pub fn new(
        ns_stack: &mut NamespaceStack,
        prefix: &str,
        name: &str,
        value: String,
    ) -> MarkupResult<Self> {
        if ns_stack.uses_xmlns() {
            if prefix.is_empty() && name == "xmlns" {
                println!("Add ns '' to be {}", value);
                ns_stack.add_ns("", &value);
                let name = Name::new(ns_stack, name, name)?;
                return Ok(Self { name, value });
            } else if prefix == "xmlns" {
                println!("Add ns {} to be value {}", name, value);
                ns_stack.add_ns(name, &value);
            }
        }
        let name = Name::new(ns_stack, prefix, name)?;
        Ok(Self { name, value })
    }

    //zz All done
}

//a Attributes
//tp Attributes
/// A list of attributes in the order in which they appear in the
/// markup stream
#[derive(Debug, Default)]
pub struct Attributes {
    //
    attributes: Vec<Attribute>,
}

//ip Attributes
impl Attributes {
    //mp is_empty
    /// Returns true if the [Attributes] list is empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    //mp add
    /// Add a prefix/name and value to the [Attributes] list, using
    /// the [NamespaceStack] to resolve the prefix into a URI
    pub fn add(
        &mut self,
        ns_stack: &mut NamespaceStack,
        prefix: &str,
        name: &str,
        value: String,
    ) -> MarkupResult<()> {
        self.attributes
            .push(Attribute::new(ns_stack, prefix, name, value)?);
        Ok(())
    }

    //mp steal
    /// Take all the attributes away from another [Attributes] and add them to this
    pub fn steal(&mut self, v: &mut Self) {
        self.attributes.append(&mut v.attributes);
    }

    //mp take
    /// Deconstruct this list of [Attribute] to a `Vec<Attribute>`
    pub fn take(self) -> Vec<Attribute> {
        self.attributes
    }

    //ap attributes
    /// Borrow the [Attribute] vec
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    //zz All done
}
