/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use std::borrow::Cow;

#[inline(always)]
fn is_rst_safe(c: u8) -> bool {
    !matches!(c, b'\\' | b'<' | b'>' | b'_' | b'*' | b'`')
}

#[inline(always)]
fn alloc_string(length: usize) -> String {
    String::with_capacity(length | 15)
}

pub struct RSTEscaper {}

impl RSTEscaper {
    pub fn new() -> RSTEscaper {
        RSTEscaper {}
    }

    pub fn escape<'a>(
        &self,
        text: &'a str,
        escape_ending_whitespace: bool,
        must_not_be_empty: bool,
    ) -> Cow<'a, str> {
        let length = text.len();
        if length == 0 {
            if must_not_be_empty {
                return Cow::Owned("\\ ".to_string());
            } else {
                return Cow::Borrowed(text);
            }
        }
        let mut index = 0;
        let mut result = alloc_string(length);
        let mut can_borrow = true;
        if escape_ending_whitespace {
            if text.as_bytes()[0] == b' ' {
                can_borrow = false;
                result.push_str("\\ ");
            } else if text.ends_with(" ") {
                can_borrow = false;
            }
        }
        loop {
            let mut next_index = index;
            while next_index < length && is_rst_safe(text.as_bytes()[next_index]) {
                next_index += 1;
            }
            if index == 0 && can_borrow && next_index == length {
                return Cow::Borrowed(text);
            }
            if index < next_index {
                result.push_str(&text[index..next_index]);
            }
            if next_index == length {
                if escape_ending_whitespace && index < length && text.ends_with(" ") {
                    result.push_str("\\ ");
                }
                result.shrink_to_fit();
                return Cow::Owned(result);
            }
            result.push_str("\\");
            index = next_index + 1;
            result.push_str(&text[next_index..index]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rst_escape() {
        let e = RSTEscaper::new();
        assert_eq!(e.escape("", false, false), "");
        assert_eq!(e.escape("", true, false), "");
        assert_eq!(e.escape("", false, true), "\\ ");
        assert_eq!(e.escape("", true, true), "\\ ");
        assert_eq!(e.escape(" ", false, false), " ");
        assert_eq!(e.escape(" ", true, false), "\\  \\ ");
        assert_eq!(e.escape(" ", false, true), " ");
        assert_eq!(e.escape(" ", true, true), "\\  \\ ");
        assert_eq!(e.escape("  ", false, false), "  ");
        assert_eq!(e.escape("  ", true, false), "\\   \\ ");
        assert_eq!(e.escape("  ", false, true), "  ");
        assert_eq!(e.escape("  ", true, true), "\\   \\ ");
        assert_eq!(
            e.escape(" a\\b<c>d_e*f`g ", false, false),
            " a\\\\b\\<c\\>d\\_e\\*f\\`g "
        );
        assert_eq!(
            e.escape(" a\\b<c>d_e*f`g ", true, false),
            "\\  a\\\\b\\<c\\>d\\_e\\*f\\`g \\ "
        );
        assert_eq!(
            e.escape(" a\\b<c>d_e*f`g ", false, true),
            " a\\\\b\\<c\\>d\\_e\\*f\\`g "
        );
        assert_eq!(
            e.escape(" a\\b<c>d_e*f`g ", true, true),
            "\\  a\\\\b\\<c\\>d\\_e\\*f\\`g \\ "
        );
    }
}
