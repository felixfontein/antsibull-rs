/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use regex;
use std::borrow::Cow;

pub struct MDEscaper {
    md_escape_re: regex::Regex,
}

impl MDEscaper {
    pub fn new() -> Result<MDEscaper, regex::Error> {
        Ok(MDEscaper {
            md_escape_re: regex::Regex::new("([!\"#$%&'()*+,:;<=>?@\\[\\\\\\]^_`{|}~.-])")?,
        })
    }

    #[inline]
    pub fn escape<'a>(&self, text: &'a str) -> Cow<'a, str> {
        self.md_escape_re.replace_all(text, "\\$1")
    }
}
