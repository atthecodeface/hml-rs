//a Imports
use crate::Name;

//a Attribute
//tp Attribute
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Attribute {
    /// Name and optional namespace
    pub name: Name,

    /// Attribute value.
    pub value: String
}

impl Attribute {
    pub fn new(name:Name, value:&str) -> Self {
        let value = value.into();
        Self { name, value }
    }
}

//tp Attributes
#[derive(Debug)]
pub struct Attributes {
    //
    attributes: Vec<Attribute>
}

//ip Attributes
impl Attributes {
    //fp new
    pub fn new() -> Self {
        Self { attributes : Vec::new() }
    }
    //mp is_empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
    pub fn add(&mut self, name:Name, value:&str) {
        self.attributes.push(Attribute::new(name,value));
    }
    pub fn steal(&mut self, v:&mut Self) {
        self.attributes.append(&mut v.attributes);
    }
    pub fn take(self) -> Vec<Attribute>  {
        self.attributes
    }
    /*
    pub fn to_attributes(&self) -> Vec<(String,String)> {
        self.attributes
            .iter()
            .map( |(n,v)| (n.name.clone(),v.clone()) )
            .collect()
    }
*/
    /*
    pub fn as_xml_attributes<'a> (&'a self) -> Cow<'a, [XmlAttribute<'a>]> {
        self.attributes
            .iter()
            .map( |(n,v)| XmlAttribute::new(n.as_xml_name(), v) )
            .collect()
    }
     */
    /*
    pub fn iter_attributes(&self) -> impl Iterator<Item = (&str, &str)> {
        self.attributes
            .iter()
            .map( |(n,v)| (n.name.as_str(), v.as_str()) )
    }
*/
}
