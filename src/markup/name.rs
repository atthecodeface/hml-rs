//a Imports
use crate::{NSNameId, NSPrefixId, NSUriId};
use crate::NamespaceStack;
use crate::HmlError;

//a Name
//tp Name
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
    //fp new_local
    pub fn new_local(ns:&mut NamespaceStack, name:&str) -> Result<Self, HmlError> {
        if name.is_empty() {
            return HmlError::empty_name()
        }
        let prefix = NSPrefixId::none();
        let uri    = NSUriId::none();
        let name   = ns.add_name(name);
        Ok(Self { prefix, uri, name })
    }

    //fp new
    pub fn new(ns:&mut NamespaceStack, prefix:&str, name:&str) -> Result<Self, HmlError> {
        if name.is_empty() {
            return HmlError::empty_name()
        }
        if let Some(p_id) = ns.find_prefix_id(prefix) {
            if let Some(uri) = ns.find_mapping(p_id) {
                let name   = ns.add_name(name);
                Ok(Self { prefix:p_id, uri, name })
            } else {
                HmlError::unmapped_prefix(prefix)
            }
        } else {
            HmlError::unmapped_prefix(prefix)
        }
    }

    //fp from_str
    pub fn from_str(ns:&mut NamespaceStack, s:&str) -> Result<Self, HmlError> {
        let mut it = s.split(':');
        match (it.next(), it.next(), it.next()) {
            (Some(prefix), Some(name), None) => Self::new(ns, prefix, name),
            (Some(name), None, None)         => Self::new_local(ns, name),
            _ => HmlError::bad_name(s),
        }
    }
}
