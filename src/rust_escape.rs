use std::convert::TryFrom;

#[derive(Debug)]
pub enum Error {
    BadEscape(String),
    BadHexDigit(String),
    BadHexEscape(String),
    BadUnicode(String),
    EndOfStringInEscape(String),
}

impl Error {
    fn bad_escape<T>(reason:&str, byte:u8, bytes:&[u8], offset:usize) -> Result<T> {
        Err(Self::BadEscape(format!("{} '{}'", reason, byte)))
    }
    fn bad_hex_digit<T>(reason:&str, byte:u8) -> Result<T> {
        Err(Self::BadHexDigit(format!("{}", reason)))
    }
    fn bad_hex_escape<T>(reason:&str, bytes:&[u8], offset:usize) -> Result<T> {
        Err(Self::BadHexEscape(format!("{}",reason)))
    }
    fn bad_unicode<T>(reason:&str, bytes:&[u8], offset:usize) -> Result<T> {
        Err(Self::BadUnicode(format!("{}",reason)))
    }
    fn end_of_string_in_escape<T>(bytes:&[u8]) -> Result<T> {
        Err(Self::EndOfStringInEscape(std::str::from_utf8(bytes).unwrap().into()))
    }
}

type Result<T> = std::result::Result<T, Error>;

fn hex_of_byte(reason:&str, b:u8) -> Result<u32> {
    let value = {
        if b >= b'0' && b <= b'9' {
            b - b'0'
        } else if b >= b'a' && b <= b'f' {
            10+(b - b'a')
        } else if b >= b'A' && b <= b'F' {
            10+(b - b'A')
        } else {
            return Error::bad_hex_digit(reason, b)
        }
    };
    Ok(value as u32)
}

#[derive(Debug)]
pub struct Escapable<'a> {
    s: &'a str,
    escaped : Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum EscapeState {
    Normal,
    Backslashed,
    Unicode(usize),
}

impl <'a> AsRef<str> for Escapable<'a> {
    fn as_ref(&self) -> &str {
        if let Some(escaped) = &self.escaped  {
            escaped.as_ref()
        } else {
            self.s
        }
    }
}

impl <'a> Escapable<'a> {
    pub fn new(s:&'a str) -> Result<Self> {
        let escaped = Self::opt_unescape(s)?;
        Ok( Self { s, escaped } )
    }

    fn unescape(bytes:&[u8], mut i:usize, n:usize) -> Result<Option<String>> {
        use EscapeState::*;
        let mut r = Vec::with_capacity(n);
        if i > 0 {
            r.extend_from_slice(&bytes[0..i]);
        }
        let mut escape_state = Normal;
        let mut unicode_value = 0;
        while i < n {
            let b = bytes[i];
            match escape_state {
                Normal => {
                    if b == b'\\' {
                        escape_state = Backslashed;
                    } else {
                        r.push(b);
                    }
                },
                Backslashed => {
                    match b {
                        b'0'  => { r.push(0);  escape_state = Normal; },
                        b't'  => { r.push(9);  escape_state = Normal; },
                        b'r'  => { r.push(13); escape_state = Normal; },
                        b'n'  => { r.push(10); escape_state = Normal; },
                        b'\'' => { r.push(39); escape_state = Normal; },
                        b'"'  => { r.push(34); escape_state = Normal; },
                        b'\\' => { r.push(92); escape_state = Normal; },
                        b'x'  => {
                            if i < n-2 {
                                let unicode =
                                    (hex_of_byte( "hex escape requires hex digits", bytes[i+1] )? << 4) +
                                    (hex_of_byte( "hex escape requires hex digits", bytes[i+2] )? << 0 );
                                if unicode > 0x7f {
                                    return Error::bad_hex_escape("hex escape must be in range 0-0x7f", bytes, i)
                                }
                                r.push(unicode as u8);
                                i += 2;
                                escape_state = Normal;
                            } else {
                                return Error::bad_hex_escape("hex escape must be \\xXX", bytes, i)
                            }
                        },
                        b'u'  => {
                            // requires \u{X} minimum
                            if i < n-2 {
                                if bytes[i+1] != b'{' {
                                    return Error::bad_unicode("\\u escape requires { to follow", bytes, i)
                                }
                                unicode_value = hex_of_byte( "unicode escape requires hex digits", bytes[i+2] )?;
                                escape_state = Unicode(1);
                                i += 2;
                            } else {
                                return Error::bad_unicode("malformed unicode escape", bytes, i)
                            }
                        },
                        _ => {
                            return Error::bad_escape("bad escape", b, bytes, i)
                        }
                    }
                },
                Unicode(n) => {
                    if b == b'}' {
                        if let Some(ch) = std::char::from_u32(unicode_value) {
                            let mut buf = [0; 4];
                            let buf = ch.encode_utf8(&mut buf).as_bytes();
                            r.extend_from_slice(buf);
                        } else {
                            return Error::bad_unicode("invalid unicode value", bytes, i)
                        }
                        escape_state = Normal;
                    } else if n==6 {
                        return Error::bad_unicode("at most 6 hex digits", bytes, i)
                    } else {
                        let v = hex_of_byte( "unicode escape requires hex digits", b )?;
                        unicode_value = (unicode_value << 4) + v;
                        escape_state = Unicode(n+1);
                    }
                },
            }
            i += 1;
        }
        if escape_state != Normal {
            Error::end_of_string_in_escape(bytes)
        } else {
            let string = unsafe { String::from_utf8_unchecked(r) };
            Ok(Some(string))
        }
    }
    fn opt_unescape(s:&str) -> Result<Option<String>> {
        let n = s.len();
        let bytes = s.as_bytes();
        for i in 0..n {
            match bytes[i] {
                b'\\' => { return Self::unescape(bytes, i, n); },
                _ => (),
            }
        }
        Ok(None)
    }
}

impl <'a> TryFrom<&'a str> for Escapable<'a> {
    type Error = Error;
    fn try_from(s:&'a str) -> Result<Self> {
        Self::new(s)
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryInto;
    #[test]
    fn test0() {
        let e : Escapable = r"fred".try_into().unwrap();
        assert!(Escapable::new(r"\").is_err());
        assert!(Escapable::new(r"\u").is_err());
        assert!(Escapable::new(r"\ua").is_err());
        assert!(Escapable::new(r"\z").is_err());
        assert!(Escapable::new(r"\x0").is_err());
        assert!(Escapable::new(r"\xG0").is_err());
        assert_eq!(Escapable::new(r"fred").unwrap().as_ref(), r"fred");
        assert_eq!(Escapable::new(r"\x00").unwrap().as_ref(),  "\x00");
        assert_eq!(Escapable::new(r"\n").unwrap().as_ref(),  "\n");
        assert_eq!(Escapable::new(r"\r").unwrap().as_ref(),  "\r");
        assert_eq!(Escapable::new(r"\t").unwrap().as_ref(),  "\t");
        assert_eq!(Escapable::new("\\\"").unwrap().as_ref(),  "\"");
        assert_eq!(Escapable::new(r"\'").unwrap().as_ref(),   "'");
        assert_eq!(Escapable::new(r"\0").unwrap().as_ref(),   "\0");
        assert_eq!(Escapable::new(r"\u{20}").unwrap().as_ref(),   " ");
        assert_eq!(Escapable::new(r"\u{2013}").unwrap().as_ref(),   "\u{2013}");
        assert_eq!(Escapable::new(r"\u{10ffff}").unwrap().as_ref(),   "\u{10ffff}");
        assert!(Escapable::new(r"\u{110000}").is_err());
    }

}
