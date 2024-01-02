//a Documentation
#![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
/*!

# Escape handling

This module is not ready for use

This module provides escape handling for XML and entity replacement

!*/

//a Imports
use std::collections::HashMap;

/// Result of unescaping/unentity-ify a string
pub type Result<T> = std::result::Result<T, std::io::Error>;

// Bit mask of escapes that should be parsed
//cp ESCAPE_QUOTE
/// Bitmask to enable unescaping of &quot;
pub const ESCAPE_QUOTE: usize = 1;
//cp ESCAPE_APOS
/// Bitmask to enable unescaping of &apos;
pub const ESCAPE_APOS: usize = 2;
//cp ESCAPE_GT
/// Bitmask to enable unescaping of &gt;
pub const ESCAPE_GT: usize = 4;
//cp ESCAPE_LF
/// Bitmask to enable unescaping of &#xA;
pub const ESCAPE_LF: usize = 8;
//cp ESCAPE_CR
/// Bitmask to enable unescaping of &#xD;
pub const ESCAPE_CR: usize = 16;

//cp ESCAPE_ATTR
/// Bitmask to enable unescaping of all attributes
pub const ESCAPE_ATTR: usize = ESCAPE_QUOTE | ESCAPE_APOS | ESCAPE_GT | ESCAPE_LF | ESCAPE_CR;

//cp ESCAPE_PCDATA
/// Bitmask used to unescape PCDATA - that is, none
pub const ESCAPE_PCDATA: usize = 0;

#[inline(always)]
fn do_esc(char_set: usize, esc: usize) -> bool {
    (char_set & esc) != 0
}

//fp escape_required
/// Return a Some(string) where string is an unescaped version of the input
pub fn escape_required(bytes: &[u8], char_set: usize, i: usize, n: usize) -> Option<String> {
    let mut r = Vec::with_capacity(n);
    if i > 0 {
        r.extend_from_slice(&bytes[0..i]);
    }
    // for i in i..n {
    for b in bytes.iter().take(n).skip(i) {
        if b & 0x80 != 0 {
            r.push(*b);
        } else {
            match b {
                b'&' => {
                    r.extend_from_slice(b"&amp;");
                }
                b'<' => {
                    r.extend_from_slice(b"&lt;");
                }
                b'\'' if do_esc(char_set, ESCAPE_APOS) => {
                    r.extend_from_slice(b"&apos;");
                }
                b'\"' if do_esc(char_set, ESCAPE_QUOTE) => {
                    r.extend_from_slice(b"&quot;");
                }
                b'>' if do_esc(char_set, ESCAPE_GT) => {
                    r.extend_from_slice(b"&gt;");
                }
                b'\n' if do_esc(char_set, ESCAPE_LF) => {
                    r.extend_from_slice(b"&#xA;");
                }
                b'\r' if do_esc(char_set, ESCAPE_CR) => {
                    r.extend_from_slice(b"&#xD;");
                }
                _ => {
                    r.push(*b);
                }
            }
        }
    }
    let string = unsafe { String::from_utf8_unchecked(r) };
    Some(string)
}

//fp escape
/// Return Some(string) if escaping is needed (given char_set), else None
pub fn escape(s: &str, char_set: usize) -> Option<String> {
    // Note that n == s.len is the length in bytes, not in utf8 characters
    let n = s.len();
    let bytes = s.as_bytes();
    for i in 0..n {
        match bytes[i] {
            b'&' => {
                return escape_required(bytes, char_set, i, n);
            }
            b'<' => {
                return escape_required(bytes, char_set, i, n);
            }
            b'\'' if do_esc(char_set, ESCAPE_APOS) => {
                return escape_required(bytes, char_set, i, n);
            }
            b'\"' if do_esc(char_set, ESCAPE_QUOTE) => {
                return escape_required(bytes, char_set, i, n);
            }
            b'>' if do_esc(char_set, ESCAPE_GT) => {
                return escape_required(bytes, char_set, i, n);
            }
            b'\n' if do_esc(char_set, ESCAPE_LF) => {
                return escape_required(bytes, char_set, i, n);
            }
            b'\r' if do_esc(char_set, ESCAPE_CR) => {
                return escape_required(bytes, char_set, i, n);
            }
            _ => (),
        }
    }
    None
}

//tp Entities
/// A set of entities that should be unmapped and how they should be unmapped
#[derive(Default)]
pub struct Entities<'a> {
    map: HashMap<&'a [u8], &'a str>,
}

//ip Entities
impl<'a> Entities<'a> {
    //fp xml
    /// Create a new Entities set for XML entity parsing
    pub fn xml() -> Self {
        let mut map: HashMap<&[u8], &str> = HashMap::new();
        map.insert(b"amp", "&");
        map.insert(b"AMP", "&");
        map.insert(b"lt", "<");
        map.insert(b"LT", "<");
        map.insert(b"gt", ">");
        map.insert(b"GT", ">");
        map.insert(b"apos", "'");
        map.insert(b"APOS", "'");
        map.insert(b"quot", "\"");
        map.insert(b"QUOT", "\"");
        Self { map }
    }

    //fp find_span
    /// Find the span starting with the given index `i` that is either
    /// from an entity (starting with '&' ending with ';') - which is
    /// then unmapped if possible, or the span until the end of string
    /// or the next entity.
    ///
    /// The return value is the index of the end of the span, and a
    /// possible replacement string or replacement character - if the
    /// span is an entity it can be mapped to either of these (or an
    /// unknown/bad entity is just a simple span).
    ///
    /// Hence a return value of (n, Some(r), None) indicates that from
    /// `i` to `n` (inclusive to exclusive) is an entity that can be
    /// replaced with the string `r`.
    ///
    /// A return value of (n, None, Some(c)) indicates that from
    /// `i` to `n` (inclusive to exclusive) is an entity that can be
    /// replaced with the character `c`.
    ///
    /// The other possible return value is (n, None, None), indicating
    /// that the span from `i` to `n` contains no entity references
    fn find_span(
        &self,
        inc_map: bool,
        bytes: &[u8],
        mut i: usize,
        n: usize,
    ) -> (usize, Option<&str>, Option<char>) {
        if bytes[i] == b'&' {
            i += 1;
            let start = i;
            let mut is_hex = false;
            let mut is_dec = true;
            let mut value = 0;
            while i < n {
                let b = bytes[i];
                if b == b';' {
                    if inc_map {
                        if let Some(c) = self.map.get(&bytes[start..i]) {
                            return (i + 1, Some(c), None);
                        }
                    }
                    if is_hex || is_dec {
                        if let Ok(c) = char::try_from(value) {
                            return (i + 1, None, Some(c));
                        }
                    }
                    i += 1;
                    break;
                }
                if i == start {
                    if b != b'#' {
                        is_dec = false;
                    }
                } else if (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b) {
                    value = (value << 4) | (((b & 0xf) + 9) as u32);
                    is_dec = false;
                } else if b == b'x' {
                    if i == start + 1 && is_dec {
                        is_hex = true;
                    }
                    is_dec = false;
                } else if b.is_ascii_digit() {
                    if is_dec {
                        value = (value * 10).wrapping_add((b - b'0') as u32);
                    } else {
                        value = (value << 4) | ((b & 0xf) as u32);
                    }
                    if value > 0x10ffff {
                        is_dec = false;
                        is_hex = false;
                        value = 0;
                    }
                } else {
                    is_dec = false;
                    is_hex = false;
                }
                i += 1;
            }
            (i, None, None)
        } else {
            i += 1;
            while i < n {
                if bytes[i] == b'&' {
                    break;
                }
                i += 1;
            }
            (i, None, None)
        }
    }

    //fp replace_entities
    /// Replace general entity references and &#..; characters, using the map.
    ///
    /// The buffer `bytes` is the source and it has length `n`.
    ///
    /// The buffer at `bytes` has the span from 0..d as a valid UTF8 string;
    /// at `d` there is an entity that ends at `i` which should be replaced with `c`.
    ///
    /// From `i` there may be more entities that require replacement.
    fn replace_entities_required(
        &self,
        inc_map: bool,
        bytes: &[u8],
        c: &str,
        d: usize,
        mut i: usize,
        n: usize,
    ) -> Option<String> {
        let mut r = Vec::with_capacity(n);
        if d > 0 {
            r.extend_from_slice(&bytes[0..d]);
        }
        r.extend_from_slice(c.as_bytes());
        while i < n {
            let (next_i, opt_a, opt_b) = self.find_span(inc_map, bytes, i, n);
            if let Some(c) = opt_a {
                r.extend_from_slice(c.as_bytes());
            } else if let Some(c) = opt_b {
                let mut buf = [0; 4];
                let buf = c.encode_utf8(&mut buf).as_bytes();
                r.extend_from_slice(buf);
            } else {
                r.extend_from_slice(&bytes[i..next_i]);
            }
            i = next_i;
        }
        let string = unsafe { String::from_utf8_unchecked(r) };
        Some(string)
    }

    //fp replace_entities
    /// Replace general entity references and &#..; characters, using the map.
    ///
    /// Return None if the string has no replacements required; else Some(new string).
    ///
    /// The replacements that are used should *also* be replaced if this is expanding a general entity use.
    ///
    /// We don't handle parameter entities here yet ('%thing;')
    ///
    /// However, the map should not be used for entity declaration
    /// contents in XML hence inc_map is provided. However, character
    /// entities &#..; are expanded in entity declarations.
    ///
    /// Character entities are *ALSO* expanded when entities are used.
    ///
    /// Another option would be to use two different [Entities] to
    /// handle the two different cases.
    ///
    /// <!ENTITY example "<p>An ampersand (&#38;#38;) may be escaped
    /// numerically (&#38;#38;#38;) or with a general entity
    /// (&amp;amp;).</p>" >
    ///
    /// makes 'example' be
    ///
    /// <p>An ampersand (&#38;) may be escaped
    /// numerically (&#38;#38;) or with a general entity
    /// (&amp;amp;).</p>
    ///
    /// and a reference in a doc to &example; is then replaced with a 'p' element with content
    ///
    /// An ampersand (&) may be escaped
    /// numerically (&#38;) or with a general entity
    /// (&amp;).
    ///
    pub fn replace_entities(&self, inc_map: bool, s: &str) -> Option<String> {
        // Note that s.len is the length in bytes, not in utf8 characters
        let n = s.len();
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < n {
            // Find next span
            //
            let (next_i, opt_a, opt_b) = self.find_span(inc_map, bytes, i, n);
            if let Some(c) = opt_a {
                // The return from find_span was(n, Some(c:&str), None): the span up to `n` is
                // an entity reference to be replaced with `c`
                return self.replace_entities_required(inc_map, bytes, c, i, next_i, n);
            } else if let Some(c) = opt_b {
                // The return from find_span was(n, None, Some(c:char)): the span up to `n` is
                // an entity reference to be replaced with `c`
                let mut buf = [0; 4];
                let buf = c.encode_utf8(&mut buf);
                return self.replace_entities_required(inc_map, bytes, buf, i, next_i, n);
            }
            // The return from find_span was(n, None, None): the span up to `n` has
            // no entity references
            i = next_i;
        }
        None
    }
}

//a Test
#[cfg(test)]
mod test {
    use super::*;
    // fn check_ok( r:Result<Option<String>>, e:Option<&str> ) {
    fn check_ok(r: Option<String>, e: Option<&str>) {
        // assert!(r.is_ok());
        // let r = r.unwrap();
        assert_eq!(r, e.map(|s| s.into()));
    }
    #[test]
    fn test0() {
        check_ok(escape("fred", ESCAPE_ATTR), None);
        check_ok(escape("banana", ESCAPE_ATTR), None);
        check_ok(
            escape("My < and more", ESCAPE_ATTR),
            Some("My &lt; and more"),
        );
        check_ok(
            escape("My > and less", ESCAPE_ATTR),
            Some("My &gt; and less"),
        );
        check_ok(
            escape("My '\"& etc", ESCAPE_ATTR),
            Some("My &apos;&quot;&amp; etc"),
        );
        check_ok(escape("\u{1f600}", ESCAPE_ATTR), None);
        check_ok(escape("\u{1f600} <", ESCAPE_ATTR), Some("\u{1f600} &lt;"));
        check_ok(
            escape("\u{1f600} < \u{1f600} ", ESCAPE_ATTR),
            Some("\u{1f600} &lt; \u{1f600} "),
        );
    }
    #[test]
    fn test_entities() {
        let e = Entities::xml();
        check_ok(e.replace_entities(true, "fred"), None);
        check_ok(e.replace_entities(true, "&amp;&AMP;"), Some("&&"));
        check_ok(e.replace_entities(true, "&lt;&LT;&GT;&gt;"), Some("<<>>"));
        check_ok(e.replace_entities(true, "&blob;&QUOT;"), Some("&blob;\""));
        check_ok(e.replace_entities(true, "&#xfffffff;"), None);
        check_ok(e.replace_entities(true, "&#x32;"), Some("2"));
        check_ok(e.replace_entities(true, "&#32;"), Some(" "));
        check_ok(e.replace_entities(true, "&#9999999999999;"), None);
        check_ok(e.replace_entities(true, "&#x32;&#32;"), Some("2 "));
        check_ok(e.replace_entities(true, "&#32;&#x32;"), Some(" 2"));
    }
}
