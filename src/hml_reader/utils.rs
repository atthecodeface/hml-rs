//a Utils
//fp is_quote
/// Returns true if the UTF character is either a single or double quote
#[inline]
pub fn is_newline(ch: char) -> bool {
    ch == '\n'
}
#[inline]
pub fn is_hash(ch: char) -> bool {
    ch == '#'
}
#[inline]
pub fn is_quote(ch: char) -> bool {
    ch == '"' || ch == '\''
}

//fp is_name_start
/// Returns true if the UTF character is a colon, underscore, alphabetic, or UTF name character
pub fn is_name_start(ch: char) -> bool {
    let ch = ch as u32;
    match ch {
        // ?? 58 => {true}, // colon
        95 => true, // underscore
        _ => {
            (65..=90).contains(&ch)       ||    // A-Z
            (97..=122).contains(&ch)      ||    // a-z
            (0xc0..=0xd6).contains(&ch)      ||
            (0xd8..=0xf6).contains(&ch)      ||
            (0xf8..=0x2ff).contains(&ch)      ||
            (0x370..=0x37d).contains(&ch)      ||
            (0x37f..=0x1fff).contains(&ch)      ||
            (0x200c..=0x200d).contains(&ch)      ||
            (0x2070..=0x218f).contains(&ch)      ||
            (0x2c00..=0x2fef).contains(&ch)      ||
            (0x3001..=0xd7ff).contains(&ch)      ||
            (0xf900..=0xfdcf).contains(&ch)      ||
            (0xfdf0..=0xfffd).contains(&ch)      ||
            (0x10000..=0xeffff).contains(&ch)
        }
    }
}

//fp is_name
/// Returns true if the UTF character is a name character or a
/// continuation of a name character that adds -, ., digits, and other
/// UTF characters
pub fn is_name(ch: char) -> bool {
    if is_name_start(ch) {
        true
    } else {
        let ch = ch as u32;
        ((ch==45) || (ch==46) || (ch==0xb7)) || // - .
            (48..=57).contains(&ch) ||
            (0x369..=0x36f).contains(&ch) ||
            (0x203f..=0x2040).contains(&ch)
    }
}
