/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

//! String builders.

use std::borrow::Cow;

pub trait Appender<'a> {
    fn push_str(&mut self, value: &'a str);
    fn push_string(&mut self, value: &'a String);
    fn push_borrowed_string(&mut self, value: &String);
    fn push_owned_string(&mut self, value: String);
    fn push_cow_str(&mut self, value: Cow<'a, str>);
}

pub trait AppendTo<'a> {
    fn append_to(self, other: &mut dyn Appender<'a>);
}

pub trait IntoString {
    fn into_string(self) -> String;
    fn len(&self) -> usize;
}

// String

impl<'a> Appender<'a> for String {
    fn push_str(&mut self, value: &'a str) {
        self.push_str(value);
    }

    fn push_string(&mut self, value: &'a String) {
        self.push_str(value.as_str());
    }

    fn push_borrowed_string(&mut self, value: &String) {
        self.push_str(value);
    }

    fn push_owned_string(&mut self, value: String) {
        self.push_str(&value);
    }

    fn push_cow_str(&mut self, value: Cow<'a, str>) {
        self.push_str(&*value);
    }
}

impl<'a> AppendTo<'a> for &'a String {
    fn append_to(self, other: &mut dyn Appender<'a>) {
        other.push_string(self);
    }
}

impl IntoString for String {
    fn into_string(self) -> String {
        self
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// CollectorAppender

pub struct CollectorAppender<'a> {
    length: usize,
    content: Vec<Cow<'a, str>>,
}

impl<'a> CollectorAppender<'a> {
    pub fn new() -> CollectorAppender<'a> {
        CollectorAppender {
            length: 0,
            content: Vec::new(),
        }
    }
}

impl<'a> Appender<'a> for CollectorAppender<'a> {
    fn push_str(&mut self, value: &'a str) {
        self.length += value.len();
        self.content.push(Cow::Borrowed(value));
    }

    fn push_string(&mut self, value: &'a String) {
        self.length += value.len();
        self.content.push(Cow::Borrowed(&value));
    }

    fn push_borrowed_string(&mut self, value: &String) {
        self.length += value.len();
        self.content.push(Cow::Owned(value.clone()));
    }

    fn push_owned_string(&mut self, value: String) {
        self.length += value.len();
        self.content.push(Cow::Owned(value));
    }

    fn push_cow_str(&mut self, value: Cow<'a, str>) {
        self.length += value.len();
        self.content.push(value);
    }
}

impl<'a> AppendTo<'a> for CollectorAppender<'a> {
    fn append_to(self, other: &mut dyn Appender<'a>) {
        for part in self.content {
            other.push_cow_str(part);
        }
    }
}

impl<'a> IntoString for CollectorAppender<'a> {
    fn into_string(self) -> String {
        let mut result = String::with_capacity(self.length);
        for part in &self.content {
            result.push_str(part);
        }
        result
    }

    fn len(&self) -> usize {
        self.length
    }
}

// StringAppender

pub struct StringAppender {
    result: String,
}

impl StringAppender {
    pub fn new() -> StringAppender {
        StringAppender {
            result: String::new(),
        }
    }
}

impl<'a> Appender<'a> for StringAppender {
    fn push_str(&mut self, value: &'a str) {
        self.result.push_str(value);
    }

    fn push_string(&mut self, value: &'a String) {
        self.result.push_str(&value);
    }

    fn push_borrowed_string(&mut self, value: &String) {
        self.result.push_str(value);
    }

    fn push_owned_string(&mut self, value: String) {
        self.result.push_str(&value);
    }

    fn push_cow_str(&mut self, value: Cow<'a, str>) {
        self.result.push_str(&*value);
    }
}

impl<'a> AppendTo<'a> for &'a StringAppender {
    fn append_to(self: &'a StringAppender, other: &mut dyn Appender<'a>) {
        other.push_string(&self.result);
    }
}

impl IntoString for StringAppender {
    fn into_string(self) -> String {
        let mut res = self.result;
        res.shrink_to_fit();
        res
    }

    fn len(&self) -> usize {
        self.result.len()
    }
}
