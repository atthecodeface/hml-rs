//a Imports
use super::NamespaceStack;
use super::{NSNameId, NSPrefixId, NSUriId};
use crate::{MarkupError, MarkupResult};

//a Name
//tp Name
/// A name within a markup stream consisting of ids for a prefix and
/// the actual name; the prefix can be the none id.
///
/// If the prefix is not none, then the URI id will correspond to the
/// id of a URI that the prefix has been declared to map to (given the
/// namespace stack)
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Name {
    /// A name prefix, e.g. `xsi` in `xsi:string`.
    pub prefix: NSPrefixId,

    /// A namespace URI, e.g. `http://www.w3.org/2000/xmlns/`.
    pub uri: NSUriId,

    /// A name
    pub name: NSNameId,
}

//ip Name
impl Name {
    //fp none
    /// Create a `None` [Name]
    pub fn none() -> Self {
        let prefix = NSPrefixId::none();
        let uri = NSUriId::none();
        let name = NSNameId::none();
        Self { prefix, uri, name }
    }

    //fp new_local
    /// Create a [Name] from a name with an prefix of None
    pub fn new_local(ns: &mut NamespaceStack, name: &str) -> MarkupResult<Self> {
        if name.is_empty() {
            return Err(MarkupError::empty_name());
        }
        let prefix = NSPrefixId::none();
        let uri = NSUriId::none();
        let name = ns.add_name(name);
        Ok(Self { prefix, uri, name })
    }

    //fp new
    /// Create a [Name] fom a prefix string and a name
    ///
    /// If the name is illegal (empty) or the prefix is not mapped (an
    /// empty prefix can be mapped in the namespace by default, so an
    /// empty prefix is not illegal per s) then an error is returned.
    pub fn new(ns: &mut NamespaceStack, prefix: &str, name: &str) -> MarkupResult<Self> {
        if name.is_empty() {
            return Err(MarkupError::empty_name());
        }
        if let Some(p_id) = ns.find_prefix_id(prefix) {
            if let Some(uri) = ns.find_mapping(p_id) {
                let name = ns.add_name(name);
                Ok(Self {
                    prefix: p_id,
                    uri,
                    name,
                })
            } else {
                Err(MarkupError::unmapped_prefix(prefix))
            }
        } else {
            Err(MarkupError::unmapped_prefix(prefix))
        }
    }

    //fp from_str
    /// Create a [Name] fom a string given the current
    /// [NamespaceStack], returning an error if the namespace is
    /// unmapped or the input string is malformed
    ///
    /// A well-formed string is either <ns>:<name> or <name>; two ':'
    /// characters are illegal
    pub fn from_str(ns: &mut NamespaceStack, s: &str) -> MarkupResult<Self> {
        let mut it = s.split(':');
        match (it.next(), it.next(), it.next()) {
            (Some(prefix), Some(name), None) => Self::new(ns, prefix, name),
            (Some(name), None, None) => Self::new_local(ns, name),
            _ => Err(MarkupError::bad_name(s)),
        }
    }

    //fp to_string
    /// Create a new `String` of the [Name]
    pub fn to_string(&self, ns: &NamespaceStack) -> String {
        if self.prefix.is_none() {
            ns.name_str(self.name).to_string()
        } else {
            format!("{}:{}", ns.prefix_str(self.prefix), ns.name_str(self.name))
        }
    }

    //ap has_prefix
    /// Returns true if the name has a prefix
    pub fn has_prefix(&self) -> bool {
        !self.prefix.is_none()
    }

    //ap has_uri
    /// Returns 'true' if the name has a URI associated with it
    pub fn has_uri(&self) -> bool {
        !self.uri.is_none()
    }
}

//a If xml_rs is included
#[cfg(feature = "xml")]
//ip Name
impl Name {
    //mp as_xml_name
    /// Get an [xml::name::OwnedName] from this Name
    pub fn as_xml_name<'a>(&'a self, ns: &'a NamespaceStack) -> xml::name::Name<'a> {
        if self.has_prefix() {
            xml::name::Name::prefixed(ns.name_str(self.name), ns.prefix_str(self.prefix))
        } else {
            xml::name::Name::local(ns.name_str(self.name))
        }
    }
}
