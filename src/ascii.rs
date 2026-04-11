pub fn escape<B>(bytes: B) -> String
where
    B: ExactSizeIterator<Item = u8>,
{
    let bytes_len = bytes.len();
    let mut buf = Vec::with_capacity(bytes_len * 4);
    for b in bytes {
        match escape_byte(b) {
            Some(mut seq) => buf.append(&mut seq),
            None => buf.push(b),
        }
    }
    String::from_utf8(buf).unwrap() // We know it's all ascii
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum UnescapeError {
    #[error("invalid escape sequence: {0}")]
    InvalidEscapeSequence(String),
}

pub fn unescape<'a, B>(bytes: &'a mut B) -> Result<String, UnescapeError>
where
    B: ExactSizeIterator<Item = &'a u8>,
{
    let bytes_len = bytes.len();
    let mut buf = Vec::with_capacity(bytes_len);
    let mut iter = bytes.peekable();
    while let Some(b) = iter.next() {
        if *b != b'\\' {
            buf.push(*b);
            continue;
        }

        let next_byte = iter.next().ok_or_else(|| {
            UnescapeError::InvalidEscapeSequence("trailing backslash".to_string())
        })?;

        if *next_byte == b'x' {
            let hex_digits: Vec<u8> = iter.by_ref().take(2).cloned().collect();
            if hex_digits.len() != 2 {
                return Err(UnescapeError::InvalidEscapeSequence(format!(
                    "incomplete hex escape: \\x{}",
                    String::from_utf8_lossy(&hex_digits)
                )));
            }
            let hex_str = String::from_utf8(hex_digits).map_err(|_| {
                UnescapeError::InvalidEscapeSequence("invalid UTF-8 in hex digits".to_string())
            })?;
            let byte = u8::from_str_radix(&hex_str, 16).map_err(|_| {
                UnescapeError::InvalidEscapeSequence(format!("invalid hex byte: \\x{}", hex_str))
            })?;
            buf.push(byte);
            continue;
        }

        let escaped_char = match next_byte {
            b't' => b'\t',
            b'r' => b'\r',
            b'n' => b'\n',
            b' ' => b' ',
            b'\'' => b'\'',
            b'"' => b'"',
            b'\\' => b'\\',
            _ => {
                return Err(UnescapeError::InvalidEscapeSequence(format!(
                    "unrecognized escape sequence: \\{}",
                    *next_byte as char
                )));
            }
        };
        buf.push(escaped_char);
    }

    Ok(String::from_utf8(buf).unwrap()) // We know it's all ascii
}

fn escape_byte(b: u8) -> Option<Vec<u8>> {
    if let seq @ Some(_) = match b {
        b'\t' => Some(b"\\t".to_vec()),
        b'\r' => Some(b"\\r".to_vec()),
        b'\n' => Some(b"\\n".to_vec()),
        b' ' => Some(b"\\ ".to_vec()),
        b'\'' => Some(b"\\'".to_vec()),
        b'\"' => Some(b"\\\"".to_vec()),
        b'\\' => Some(b"\\\\".to_vec()),
        _ => None,
    } {
        return seq;
    }

    if (0x20..0x7f).contains(&b) {
        return None;
    }

    Some(format!("\\x{:02x}", b).into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod escape {
        use super::*;

        #[test]
        fn escapes_whitespace_characters() {
            assert_eq!(
                escape(b"space separated".iter().copied()),
                "space\\ separated"
            );
            assert_eq!(
                escape(b"newline\nseparated".iter().copied()),
                "newline\\nseparated"
            );
            assert_eq!(
                escape(&mut b"carriage-return\rseparated".iter().copied()),
                "carriage-return\\rseparated"
            );
            assert_eq!(
                escape(&mut b"tab\tseparated".iter().copied()),
                "tab\\tseparated"
            );
        }

        #[test]
        fn escapes_quotes() {
            assert_eq!(escape(&mut b"'single'".iter().copied()), "\\'single\\'");
            assert_eq!(escape(&mut b"\"double\"".iter().copied()), "\\\"double\\\"");
        }

        #[test]
        fn escapes_backslashes() {
            assert_eq!(escape(&mut b"backslash\\".iter().copied()), "backslash\\\\");
        }

        #[test]
        fn escapes_non_printable_characters() {
            assert_eq!(
                escape(&mut b"\x01\x02\x03".iter().copied()),
                "\\x01\\x02\\x03"
            );
        }
    }

    mod unescape {
        use super::*;

        #[test]
        fn escapes_whitespace_characters() {
            assert_eq!(
                unescape(&mut b"space\\ separated".iter()),
                Ok("space separated".to_string())
            );
            assert_eq!(
                unescape(&mut b"newline\\nseparated".iter()),
                Ok("newline\nseparated".to_string())
            );
            assert_eq!(
                unescape(&mut b"carriage-return\\rseparated".iter()),
                Ok("carriage-return\rseparated".to_string())
            );
            assert_eq!(
                unescape(&mut b"tab\\tseparated".iter()),
                Ok("tab\tseparated".to_string())
            );
        }

        #[test]
        fn escapes_quotes() {
            assert_eq!(
                unescape(&mut b"\\'single\\'".iter()),
                Ok("'single'".to_string())
            );
            assert_eq!(
                unescape(&mut b"\\\"double\\\"".iter()),
                Ok("\"double\"".to_string())
            );
        }

        #[test]
        fn escapes_backslashes() {
            assert_eq!(
                unescape(&mut b"backslash\\\\".iter()),
                Ok("backslash\\".to_string())
            );
        }

        #[test]
        fn escapes_non_printable_characters() {
            assert_eq!(
                unescape(&mut b"\\x01\\x02\\x03".iter()),
                Ok("\x01\x02\x03".to_string())
            );
        }
    }
}
