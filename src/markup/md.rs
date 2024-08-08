/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use crate::markup::dom;
use crate::markup::format;
use crate::markup::html_helper;
use crate::markup::md_helper;
use crate::util::stringbuilder::Appender;
use regex;
use std::rc::Rc;
use std::sync::LazyLock;

pub struct MDFormatter {
    md_escaper: md_helper::MDEscaper,
    url_escaper: html_helper::URLEscaper,
}

impl MDFormatter {
    fn new() -> Result<MDFormatter, regex::Error> {
        Ok(MDFormatter {
            md_escaper: md_helper::MDEscaper::new()?,
            url_escaper: html_helper::URLEscaper::new(),
        })
    }

    #[inline]
    fn append_tag<'a>(
        &self,
        appender: &mut dyn Appender<'a>,
        start: &'a str,
        text: &'a str,
        end: &'a str,
    ) {
        appender.push_str(start);
        appender.push_cow_str(self.md_escaper.escape(text));
        appender.push_str(end);
    }

    #[inline]
    fn append_link<'a>(&self, appender: &mut dyn Appender<'a>, text: &'a str, url: &'a str) {
        appender.push_str("[");
        appender.push_cow_str(self.md_escaper.escape(text));
        appender.push_str("](");
        appender.push_owned_string(
            self.md_escaper
                .escape(&*self.url_escaper.escape(url))
                .into_owned(),
        );
        appender.push_str(")");
    }

    #[inline]
    fn append_fqcn<'a>(
        &self,
        appender: &mut dyn Appender<'a>,
        fqcn: &'a str,
        url: &Option<String>,
    ) {
        match url {
            Some(u) => {
                appender.push_str("[");
                appender.push_cow_str(self.md_escaper.escape(fqcn));
                appender.push_str("](");
                appender.push_owned_string(
                    self.md_escaper
                        .escape(&*self.url_escaper.escape(u))
                        .into_owned(),
                );
                appender.push_str(")");
            }
            None => appender.push_cow_str(self.md_escaper.escape(fqcn)),
        }
    }

    #[inline]
    fn append_option_like<'a>(
        &self,
        appender: &mut dyn Appender<'a>,
        name: &'a String,
        value: &'a Option<String>,
        what: format::OptionLike,
        url: &Option<String>,
    ) {
        appender.push_str("<code>");
        let strong = matches!(what, format::OptionLike::Option) && matches!(value, None);
        if strong {
            appender.push_str("<strong>");
        }
        if let Some(u) = url {
            appender.push_str("<a href=\"");
            appender.push_owned_string(self.url_escaper.escape_with_html_escape(u).into_owned());
            appender.push_str("\">");
        }
        appender.push_cow_str(self.md_escaper.escape(name));
        if let Some(v) = value {
            appender.push_str("\\=");
            appender.push_cow_str(self.md_escaper.escape(v));
        }
        if let Some(_) = url {
            appender.push_str("</a>");
        }
        if strong {
            appender.push_str("</strong>");
        }
        appender.push_str("</code>");
    }
}

impl<'a> format::Formatter<'a> for MDFormatter {
    fn append(
        &self,
        appender: &mut dyn Appender<'a>,
        part: &'a dom::Part<'a>,
        url: Option<String>,
    ) {
        match part {
            dom::Part::Text { text } => appender.push_cow_str(self.md_escaper.escape(text)),
            dom::Part::Bold { text } => self.append_tag(appender, "<b>", text, "</b>"),
            dom::Part::Italic { text } => self.append_tag(appender, "<em>", text, "</em>"),
            dom::Part::Code { text } => self.append_tag(appender, "<code>", text, "</code>"),
            dom::Part::HorizontalLine => appender.push_str("<hr>"),
            dom::Part::OptionValue { value } => {
                self.append_tag(appender, "<code>", value, "</code>")
            }
            dom::Part::EnvVariable { name } => self.append_tag(appender, "<code>", name, "</code>"),
            dom::Part::Error { message } => {
                appender.push_str("<b>ERROR while parsing</b>: ");
                appender.push_cow_str(self.md_escaper.escape(message));
            }
            dom::Part::RSTRef { text, r#ref: _ } => {
                appender.push_cow_str(self.md_escaper.escape(text))
            }
            dom::Part::Link { text, url } => self.append_link(appender, text, url),
            dom::Part::URL { url } => self.append_link(appender, url, url),
            dom::Part::Module { fqcn } => self.append_fqcn(appender, &fqcn, &url),
            dom::Part::Plugin { plugin } => self.append_fqcn(appender, &plugin.fqcn, &url),
            dom::Part::OptionName {
                plugin: _,
                entrypoint: _,
                link: _,
                name,
                value,
            } => self.append_option_like(appender, name, value, format::OptionLike::Option, &url),
            dom::Part::ReturnValue {
                plugin: _,
                entrypoint: _,
                link: _,
                name,
                value,
            } => self.append_option_like(appender, name, value, format::OptionLike::RetVal, &url),
        };
    }
}

pub static MARKDOWN_FORMATTER: LazyLock<MDFormatter> =
    LazyLock::new(|| MDFormatter::new().unwrap());

/// Apply the MarkDown formatter to all parts of the given paragraph, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the MarkDown formatter.
pub fn append_md_paragraph<'a, I>(
    appender: &mut dyn Appender<'a>,
    paragraph: I,
    link_provider: &dyn format::LinkProvider,
    current_plugin: &Option<Rc<dom::PluginIdentifier>>,
) where
    I: Iterator<Item = &'a dom::Part<'a>>,
{
    format::append_paragraph(
        appender,
        paragraph,
        &*MARKDOWN_FORMATTER,
        link_provider,
        "",
        "",
        "\n\n",
        current_plugin,
    );
}

/// Apply the MarkDown formatter to all parts of the given paragraphs, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the MarkDown formatter.
pub fn append_md_paragraphs<'a, I, II>(
    appender: &mut dyn Appender<'a>,
    paragraphs: I,
    link_provider: &dyn format::LinkProvider,
    current_plugin: &Option<Rc<dom::PluginIdentifier>>,
) where
    I: IntoIterator<Item = II>,
    II: Iterator<Item = &'a dom::Part<'a>>,
{
    format::append_paragraphs(
        appender,
        paragraphs,
        &*MARKDOWN_FORMATTER,
        link_provider,
        "",
        "",
        "\n\n",
        " ",
        current_plugin,
    );
}
