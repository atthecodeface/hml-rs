//a Documentation
/*!

# Names and namespaces etc

This module provides a common mechanism for providing names,
namespaces, a stack of namespaces (used within trees, at different
levels different namespaces are visible), tags and attributes.

[Name]s are kept within the namespace with each [Name] being kept as a
[String] within the [Namespace] precisely once. Indices into the
[Namespace] vector of [Name]s are used to describe them; the same goes
for prefixes and URIs.

This permits simple use of names and other IDs throughout client
modules, without worrying about borrowing strings from a markup reader
and other ownership; it does require that the namespace be passed
around (as a mutable borrowed entity) as a markup document is
parsed. The latter is made simpler with the [NamespaceStack] keeping
hold of the mutably borrowed [Namespace], and the [NamespaceStack] is
passed around mutably - which is required as one parses a document
anyway.

The upshot of this is that this module provides a simple way to handle
tags and attribute names in markup readers by [usize] indices,
reducing the number of string comparisons required in a client.

!*/

//a Imports
mod attribute;
mod ids;
mod name;
mod namespace;
mod namespace_stack;
mod tag;

//a Exports
pub use attribute::{Attribute, Attributes};
pub use ids::{NSMap, NSNameId, NSPrefixId, NSUriId};
pub use name::Name;
pub use namespace::Namespace;
pub use namespace_stack::NamespaceStack;
pub use tag::Tag;
