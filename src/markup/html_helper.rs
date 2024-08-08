/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use std::borrow::Cow;

#[inline(always)]
fn is_url_safe(c: u8) -> bool {
    matches!(
        c,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')' |
        b';' | b'/' | b'?' | b':' | b'@' | b'&' | b'=' | b'+' | b'$' | b',' | b'#'
    )
}

#[inline(always)]
fn is_html_safe(c: u8) -> bool {
    !matches!(c, b'<' | b'>' | b'&')
}

#[inline(always)]
fn hex_digit(value: u8) -> u8 {
    // `encodeURI()` uses upper-case hex digits
    match value {
        0..=9 => b'0' + value,
        10..=15 => b'A' + value - 10,
        _ => b'\0',
    }
}

#[inline(always)]
fn alloc_string(length: usize) -> String {
    String::with_capacity(length | 15)
}

pub struct URLEscaper {}

impl URLEscaper {
    pub fn new() -> URLEscaper {
        URLEscaper {}
    }

    /// Percent encode an URL similar to JavaScript's `encodeURI()` method.
    ///
    /// See [the MDN page for `encodeURI()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/encodeURI).
    pub fn escape<'a>(&self, url: &'a str) -> Cow<'a, str> {
        let length = url.len();
        let mut index = 0;
        let mut result = alloc_string(length);
        loop {
            let mut next_index = index;
            while next_index < length && is_url_safe(url.as_bytes()[next_index]) {
                next_index += 1;
            }
            if index == 0 && next_index == length {
                return Cow::Borrowed(url);
            }
            if index < next_index {
                result.push_str(&url[index..next_index]);
            }
            if next_index == length {
                result.shrink_to_fit();
                return Cow::Owned(result);
            }
            let c = url.as_bytes()[next_index];
            let enc = &[b'%', hex_digit(c >> 4), hex_digit(c & 15)];
            result.push_str(unsafe { std::str::from_utf8_unchecked(enc) });
            index = next_index + 1;
        }
    }

    /// Percent encode an URL similar to JavaScript's `encodeURI()` method, and then HTML escape the result.
    ///
    /// The only difference to escape() is that '&' is escaped to '&amp;'.
    pub fn escape_with_html_escape<'a>(&self, url: &'a str) -> Cow<'a, str> {
        let length = url.len();
        let mut index = 0;
        let mut result = alloc_string(length);
        loop {
            let mut next_index = index;
            while next_index < length
                && is_url_safe(url.as_bytes()[next_index])
                && is_html_safe(url.as_bytes()[next_index])
            {
                next_index += 1;
            }
            if index == 0 && next_index == length {
                return Cow::Borrowed(url);
            }
            if index < next_index {
                result.push_str(&url[index..next_index]);
            }
            if next_index == length {
                result.shrink_to_fit();
                return Cow::Owned(result);
            }
            let c = url.as_bytes()[next_index];
            if c == b'&' {
                result.push_str("&amp;");
            } else {
                let enc = &[b'%', hex_digit(c >> 4), hex_digit(c & 15)];
                result.push_str(unsafe { std::str::from_utf8_unchecked(enc) });
            }
            index = next_index + 1;
        }
    }
}

pub struct HTMLEscaper {}

impl HTMLEscaper {
    pub fn new() -> HTMLEscaper {
        HTMLEscaper {}
    }

    /// Escape HTML.
    pub fn escape<'a>(&self, url: &'a str) -> Cow<'a, str> {
        let length = url.len();
        let mut index = 0;
        let mut result = alloc_string(length);
        loop {
            let mut next_index = index;
            while next_index < length && is_html_safe(url.as_bytes()[next_index]) {
                next_index += 1;
            }
            if index == 0 && next_index == length {
                return Cow::Borrowed(url);
            }
            if index < next_index {
                result.push_str(&url[index..next_index]);
            }
            if next_index == length {
                result.shrink_to_fit();
                return Cow::Owned(result);
            }
            let c = url.as_bytes()[next_index];
            result.push_str(match c {
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'&' => "&amp;",
                _ => "",
            });
            index = next_index + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_escape() {
        let e = URLEscaper::new();
        assert_eq!(e.escape(""), "");
        assert_eq!(
            e.escape("https://ansible.com/test.html"),
            "https://ansible.com/test.html"
        );
        assert_eq!(
            e.escape("https://ansible.com/test.html?f=<a>&g=h"),
            "https://ansible.com/test.html?f=%3Ca%3E&g=h"
        );
        assert_eq!(
            e.escape("https://example.com/test.html?foo=b<a>r&find=\\*#baz.bam%3D(boo"),
            "https://example.com/test.html?foo=b%3Ca%3Er&find=%5C*#baz.bam%253D(boo"
        );

        assert_eq!(e.escape_with_html_escape(""), "");
        assert_eq!(
            e.escape_with_html_escape("https://ansible.com/test.html"),
            "https://ansible.com/test.html"
        );
        assert_eq!(
            e.escape_with_html_escape("https://ansible.com/test.html?f=<a>&g=h"),
            "https://ansible.com/test.html?f=%3Ca%3E&amp;g=h"
        );
        assert_eq!(
            e.escape_with_html_escape(
                "https://example.com/test.html?foo=b<a>r&find=\\*#baz.bam%3D(boo"
            ),
            "https://example.com/test.html?foo=b%3Ca%3Er&amp;find=%5C*#baz.bam%253D(boo"
        );
    }

    #[test]
    fn test_html_escape() {
        let e = HTMLEscaper::new();
        assert_eq!(e.escape(""), "");
        assert_eq!(e.escape("test"), "test");
        assert_eq!(e.escape("<foo>"), "&lt;foo&gt;");
        assert_eq!(e.escape("<f&o>"), "&lt;f&amp;o&gt;");
    }
}
