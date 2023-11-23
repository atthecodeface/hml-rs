/*a Copyright

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

@file    namespace_stack.rs
@brief   A markup namespace stack with various frames
 */

/*a Imports
*/
use crate::names::Namespace;
use crate::names::{NSMap, NSNameId, NSPrefixId, NSUriId};
use std::collections::{HashMap, HashSet};

//a NamespaceStackFrame, NamespaceStack, and StackIter
//ti NamespaceStackFrameIter
pub struct NamespaceStackFrameIter<'frame> {
    frame: &'frame NamespaceStackFrame,
    i: usize,
    n: usize,
}

//ii NamespaceStackIterator
impl<'frame> NamespaceStackFrameIter<'frame> {
    fn new(frame: &'frame NamespaceStackFrame) -> Self {
        let n = frame.order.len();
        let i = 0;
        Self { frame, n, i }
    }
}

//ii Iterator for NamespaceStackIterator
impl<'frame> Iterator for NamespaceStackFrameIter<'frame> {
    type Item = NSMap;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n {
            None
        } else {
            let prefix_id = self.frame.order[self.i];
            self.i += 1;
            if let Some(uri_id) = self.frame.find_mapping(prefix_id) {
                let map = NSMap::new(prefix_id, *uri_id);
                Some(map)
            } else {
                self.next()
            }
        }
    }
}

//ti NamespaceStackFrame
#[derive(Default)]
struct NamespaceStackFrame {
    mappings: HashMap<NSPrefixId, NSUriId>,
    order: Vec<NSPrefixId>,
}

//ii NamespaceStackFrame
impl NamespaceStackFrame {
    //mp add_mapping_by_id
    pub fn add_mapping_by_id(&mut self, map: NSMap) {
        self.mappings.insert(map.prefix_id(), map.uri_id());
    }

    //mp add_mapping_by_id_if_unset
    pub fn add_mapping_by_id_if_unset(&mut self, map: NSMap) -> bool {
        if self.mappings.contains_key(&map.prefix_id()) {
            false
        } else {
            self.add_mapping_by_id(map);
            true
        }
    }

    //mp find_mapping
    /// Find a mapping of a prefix ID
    fn find_mapping(&self, prefix_id: NSPrefixId) -> Option<&NSUriId> {
        self.mappings.get(&prefix_id)
    }

    //mp iter_mappings
    /// Iterate over the mappings defined in this stack frame (prefix to URI)
    fn iter_mappings(&self) -> NamespaceStackFrameIter {
        NamespaceStackFrameIter::new(self)
    }
}

//ti NamespaceStackIterator
pub struct NamespaceStackIterator<'ns, 'b> {
    stack: &'b NamespaceStack<'ns>,
    // frame goes len() .. 1
    frame: usize,
    // index goes 0..frame.len()
    frame_iter: Option<std::collections::hash_map::Iter<'b, NSPrefixId, NSUriId>>,
    // set of NSPrefixId returned so far
    used: HashSet<NSPrefixId>,
}

//ii NamespaceStackIterator
impl<'ns, 'b> NamespaceStackIterator<'ns, 'b> {
    fn new(stack: &'b NamespaceStack<'ns>) -> Self {
        let frame = stack.stack_depth();
        let frame_iter = None;
        let used = HashSet::new();
        Self {
            stack,
            frame,
            frame_iter,
            used,
        }
    }
}

//ii Iterator for NamespaceStackIterator
impl<'ns, 'b> Iterator for NamespaceStackIterator<'ns, 'b> {
    type Item = NSMap;
    fn next(&mut self) -> Option<Self::Item> {
        if self.frame == 0 {
            None
        } else if let Some(iter) = &mut self.frame_iter {
            if let Some((p_id, u_id)) = iter.next() {
                if self.used.contains(p_id) {
                    self.next()
                } else {
                    Some(NSMap::new(*p_id, *u_id))
                }
            } else {
                self.frame -= 1;
                self.frame_iter = None;
                self.next()
            }
        } else {
            self.frame_iter = Some(self.stack.borrow_frame(self.frame - 1).mappings.iter());
            self.next()
        }
    }
}

//tp NamespaceStack
/// A [NamespaceStack] is a use of a [Namespace] structure within a
/// document; in general they form a pair, with the [Namespace]
/// created first and a [NamespaceStack] following.
///
/// The stack consists of frames, which can be pushed and
/// popped. Within a frame there are mappings between prefix strings
/// and URI strings; there will be at most one mapping for each prefix
/// within the frame. Different stack frames may map the same prefix
/// differently, though.
///
/// A mapping for a prefix string is determined by finding that
/// mapping in the topmost stack frame if possible; if there is no
/// mapping in that frame, the next frame down is examined, and so on.
///
/// URI and prefix strings are stored within the [Namespace]
/// structure, and are referred to by 'id's in most of the API - as
/// are mappings from prefix to URI.
///
/// A client of the [NamespaceStack] should add mappings to the [NamespaceStack] with a [NamespaceStack::add] operation, which returns an [NSMap]; it can look up mappings for a prefix string by resolving the prefix string to an ID with [NamespaceStack::find_prefix], and then finding the map using [NamespaceStack::find_mapping].
///
/// A client can iterate through all the mappings using the `iter()` method.
///
/// # Example
///
/// ```
/// use hml_rs::names::{NamespaceStack, Namespace};
///
/// let mut ns  = Namespace::new(true);
/// let mut nst = NamespaceStack::new(&mut ns);
///
/// ```
///
pub struct NamespaceStack<'ns> {
    namespaces: &'ns mut Namespace,
    frames: Vec<NamespaceStackFrame>,
}

//ip NamespaceStack
impl<'ns> NamespaceStack<'ns> {
    //fp new
    /// Create a new [NamespaceStack], mutably borrowing the
    /// [Namespace] for its lifetime
    pub fn new(namespaces: &'ns mut Namespace) -> Self {
        let mut frames = Vec::new();
        frames.push(NamespaceStackFrame::default());
        let mut s = Self { namespaces, frames };
        if s.uses_xmlns() {
            s.add_default_xmls();
        } else {
            s.add_null_ns();
        }
        s
    }

    //mp uses_xmlns
    /// Return true if the [Namespace] was specified to use the XML
    /// namespace (at creation)
    pub fn uses_xmlns(&self) -> bool {
        self.namespaces.uses_xmlns()
    }

    //mp add_null_ns
    /// Add the null namespace
    ///
    /// This is normally done at the creation of a [NamespaceStack]
    pub fn add_null_ns(&mut self) {
        self.add_ns("", "");
    }

    //mp add_default_xmls
    /// Add the default XML namespaces to the stack frame
    ///
    /// This is normally done at the creation of a [NamespaceStack]
    pub fn add_default_xmls(&mut self) {
        self.add_null_ns();
        self.add_ns("xmlns", "http://www.w3.org/2000/xmlns/");
        self.add_ns("xml", "http://www.w3.org/XML/1998/namespace");
    }

    //mp push_frame
    /// Push a new stack frame on to the [NamespaceStack]
    pub fn push_frame(&mut self) {
        self.frames.push(NamespaceStackFrame::default());
    }

    //mp pop_frame
    /// Pop the topmost stack frame from the [NamespaceStack]
    ///
    /// Panics if the stack is empty
    pub fn pop_frame(&mut self) {
        self.frames.pop().unwrap();
    }

    //mi stack_depth
    /// Returns the depth of the stack
    fn stack_depth(&self) -> usize {
        self.frames.len()
    }

    //mi borrow_frame
    /// Borrow the 'nth' frame of the stack
    fn borrow_frame(&self, n: usize) -> &NamespaceStackFrame {
        &self.frames[n]
    }

    //mp add_mapping_by_id
    /// Add a mapping of NSPrefixId -> NSUriId to the topmost stack frame
    pub fn add_mapping_by_id(&mut self, map: NSMap) {
        self.frames.last_mut().unwrap().add_mapping_by_id(map)
    }

    //mp add_mapping_by_id_if_unset
    /// Add a mapping if it does not exist *in the topmost stack fram*
    pub fn add_mapping_by_id_if_unset(&mut self, map: NSMap) -> bool {
        self.frames
            .last_mut()
            .unwrap()
            .add_mapping_by_id_if_unset(map)
    }

    //mp find_mapping
    /// Find a mapping of an [NSPrefixId] at the highest level of the
    /// stack that it exists
    ///
    /// Returns `None` if there is no mapping on the stack at all
    pub fn find_mapping(&self, prefix_id: NSPrefixId) -> Option<NSUriId> {
        let n = self.frames.len();
        for i in 0..n {
            match self.frames[n - 1 - i].find_mapping(prefix_id) {
                Some(uri_id) => return Some(*uri_id),
                _ => (),
            }
        }
        None
    }

    //mp find_prefix_id
    /// Find the NSPrefixId corresponding to a string within the
    /// underlying [Namespace]
    pub fn find_prefix_id(&mut self, prefix: &str) -> Option<NSPrefixId> {
        self.namespaces.find_prefix(prefix)
    }

    //ap iter_top_mappings
    /// Get a slice of the mappings of the topmost frame
    pub fn iter_top_mappings<'a>(&'a self) -> NamespaceStackFrameIter<'a> {
        assert!(self.frames.is_empty(), "Namespace stack cannot be empty");
        self.frames.last().unwrap().iter_mappings()
    }

    //mp borrow_mapping
    /// Borrow the two strings corresponding to an [NSMap] within the [Namespace]
    pub fn borrow_mapping(&self, map: NSMap) -> (&str, &str) {
        (self.prefix_str(map.prefix_id()), self.uri_str(map.uri_id()))
    }

    //mp name_str
    /// Borrow the name corresponding to an [NSNameId] within the [Namespace]
    pub fn name_str(&self, name: NSNameId) -> &str {
        self.namespaces.name_str(name, "")
    }

    //mp prefix_str
    /// Borrow the prefix corresponding to an [NSPrefixId] within the [Namespace]
    pub fn prefix_str(&self, prefix: NSPrefixId) -> &str {
        self.namespaces.prefix_str(prefix, "")
    }

    //mp uri_str
    /// Borrow the URI corresponding to an [NSUriId] within the [Namespace]
    pub fn uri_str(&self, uri: NSUriId) -> &str {
        self.namespaces.uri_str(uri, "")
    }

    //mp add_name
    /// Add a name for a string to the [Namespace]; if it is already
    /// in the [Namespace] then it is not added but the current
    /// NSNameId is used.
    pub fn add_name(&mut self, name: &str) -> NSNameId {
        self.namespaces.find_or_add_name(name)
    }

    //mp add_ns
    /// Add a prefix -> URI mapping to the [Namespace]
    pub fn add_ns(&mut self, prefix: &str, uri: &str) -> NSMap {
        let ns_map = self.namespaces.add_mapping(prefix, uri);
        self.add_mapping_by_id(ns_map);
        ns_map
    }

    //mp add_ns_if_unset
    /// Add a prefix -> URI mapping to the [Namespace]
    pub fn add_ns_if_unset(&mut self, prefix: &str, uri: &str) -> (NSMap, bool) {
        let ns_map = self.namespaces.add_mapping(prefix, uri);
        (ns_map, self.add_mapping_by_id_if_unset(ns_map))
    }

    //fp fmt_map
    /// Format an [NSMap] within the [Namespace]
    pub fn fmt_map<W: std::fmt::Write>(
        &self,
        w: &mut W,
        map: NSMap,
    ) -> Result<(), std::fmt::Error> {
        write!(
            w,
            "'{}' => '{}'",
            self.prefix_str(map.prefix_id()),
            self.uri_str(map.uri_id())
        )
    }

    //zz All done
}

//ip IntoIterator for NamespaceStack
impl<'ns, 'b> IntoIterator for &'b NamespaceStack<'ns> {
    type Item = NSMap;
    type IntoIter = NamespaceStackIterator<'ns, 'b>;

    fn into_iter(self) -> Self::IntoIter {
        NamespaceStackIterator::new(self)
    }
}

//a Test
#[cfg(test)]
mod test {
    use crate::names::{Namespace, NamespaceStack};
    fn dump_namespace(nst: &NamespaceStack) {
        for i in nst {
            let mut s = String::new();
            nst.fmt_map(&mut s, i).unwrap();
            println!("{}", s);
        }
    }
    #[test]
    fn test_defaults() {
        let mut ns = Namespace::new(false);
        let mut nst = NamespaceStack::new(&mut ns);

        nst.add_default_xmls();

        assert_eq!(nst.into_iter().count(), 3);

        let pid = nst.find_prefix_id("").unwrap();
        assert!(pid.is_none());
        assert_eq!(nst.prefix_str(pid, ""), "");
        assert_eq!(nst.borrow_uri(nst.find_mapping(pid).unwrap()), "");

        let pid = nst.find_prefix_id("xml").unwrap();
        assert_eq!(nst.prefix_str(pid, ""), "xml");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://www.w3.org/XML/1998/namespace"
        );

        let pid = nst.find_prefix_id("xmlns").unwrap();
        assert_eq!(nst.prefix_str(pid, ""), "xmlns");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://www.w3.org/2000/xmlns/"
        );

        let pid = nst.find_prefix_id("fred");
        assert_eq!(pid, None);

        nst.push_frame();

        nst.add_ns("fred", "http://fred.com");
        assert_eq!(nst.into_iter().count(), 4);

        let pid = nst.find_prefix_id("fred").unwrap();
        assert_eq!(nst.prefix_str(pid, ""), "fred");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://fred.com"
        );

        nst.add_ns_if_unset("fred", "http://NOTfred.com");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://fred.com"
        );

        nst.add_ns("fred", "http://fred2.com");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://fred2.com"
        );

        nst.add_ns("xml", "http://xml_override");
        let pid = nst.find_prefix_id("xml").unwrap();
        assert_eq!(nst.prefix_str(pid, ""), "xml");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://xml_override"
        );

        dump_namespace(&nst);

        nst.pop_frame();

        assert_eq!(nst.into_iter().count(), 3);

        let pid = nst.find_prefix_id("").unwrap();
        assert!(pid.is_none());
        assert_eq!(nst.prefix_str(pid, ""), "");
        assert_eq!(nst.borrow_uri(nst.find_mapping(pid).unwrap()), "");

        let pid = nst.find_prefix_id("xml").unwrap();
        assert_eq!(nst.prefix_str(pid, ""), "xml");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://www.w3.org/XML/1998/namespace"
        );

        let pid = nst.find_prefix_id("xmlns").unwrap();
        assert_eq!(nst.prefix_str(pid, ""), "xmlns");
        assert_eq!(
            nst.borrow_uri(nst.find_mapping(pid).unwrap()),
            "http://www.w3.org/2000/xmlns/"
        );

        let pid = nst.find_prefix_id("fred").unwrap(); // Note not None any more
        assert_eq!(nst.find_mapping(pid), None);
    }
}
