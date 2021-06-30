// Namespaces
//
// The Namespaces type contains sets of namespace URIs and
// identifiers, mappings between them, and stacks of valid mappings
// that change as a document is parsed.
//
mod error;
mod ids;
mod namespace;
mod namespace_stack;
mod name;
mod attribute;
mod tag;
mod event;


pub use error::{MarkupError, MarkupResult};
pub use ids::{NSNameId, NSPrefixId, NSUriId, NSMap};
pub use namespace::Namespace;
pub use namespace_stack::NamespaceStack;
pub use name::Name;
pub use attribute::{Attribute, Attributes};
pub use tag::Tag;

pub use event::Event;
