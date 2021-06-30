//a Imports
use crate::{Name, Attributes};

//a Tag
//tp Tag
#[derive(Debug)]
pub struct Tag {
    /// Name with prefix *and URI from namespace stack*
    pub name: Name,

    /// Attributes
    pub attributes: Attributes,
}

//ip Tag
impl Tag {
    pub fn new(name:Name, attributes:Attributes) -> Self {
        Self { name, attributes }
    }
    /*
    pub fn add_attribute(&mut self, a:Attribute) {
        self.attributes.add(a)
    }
    pub fn iter_attributes(&self) -> std::slice::Iter<Attribute> {
        self.attributes.iter()
    }
*/
}

