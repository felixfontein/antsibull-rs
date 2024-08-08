/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use crate::markup::dom;
use crate::markup::format;
use crate::markup::html_helper;
use crate::util::stringbuilder::Appender;
use std::rc::Rc;
use std::sync::LazyLock;

pub struct PlainHTMLFormatter {
    html_escaper: html_helper::HTMLEscaper,
    url_escaper: html_helper::URLEscaper,
}

impl PlainHTMLFormatter {
    fn new() -> PlainHTMLFormatter {
        PlainHTMLFormatter {
            html_escaper: html_helper::HTMLEscaper::new(),
            url_escaper: html_helper::URLEscaper::new(),
        }
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
        appender.push_cow_str(self.html_escaper.escape(text));
        appender.push_str(end);
    }

    #[inline]
    fn append_link<'a>(&self, appender: &mut dyn Appender<'a>, text: &'a str, url: &'a str) {
        appender.push_str("<a href='");
        appender.push_cow_str(self.url_escaper.escape_with_html_escape(url));
        appender.push_str("'>");
        appender.push_cow_str(self.html_escaper.escape(text));
        appender.push_str("</a>");
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
                appender.push_str("<a href='");
                appender
                    .push_owned_string(self.url_escaper.escape_with_html_escape(u).into_owned());
                appender.push_str("'>");
                appender.push_cow_str(self.html_escaper.escape(fqcn));
                appender.push_str("</a>");
            }
            None => {
                appender.push_str("<span>");
                appender.push_cow_str(self.html_escaper.escape(fqcn));
                appender.push_str("</span>");
            }
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
        appender.push_cow_str(self.html_escaper.escape(name));
        if let Some(v) = value {
            appender.push_str("=");
            appender.push_cow_str(self.html_escaper.escape(v));
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

impl<'a> format::Formatter<'a> for PlainHTMLFormatter {
    fn append(
        &self,
        appender: &mut dyn Appender<'a>,
        part: &'a dom::Part<'a>,
        url: Option<String>,
    ) {
        match part {
            dom::Part::Text { text } => appender.push_cow_str(self.html_escaper.escape(text)),
            dom::Part::Bold { text } => self.append_tag(appender, "<b>", text, "</b>"),
            dom::Part::Italic { text } => self.append_tag(appender, "<em>", text, "</em>"),
            dom::Part::Code { text } => self.append_tag(appender, "<code>", text, "</code>"),
            dom::Part::HorizontalLine => appender.push_str("<hr>"),
            dom::Part::OptionValue { value } => {
                self.append_tag(appender, "<code>", value, "</code>")
            }
            dom::Part::EnvVariable { name } => self.append_tag(appender, "<code>", name, "</code>"),
            dom::Part::Error { message } => {
                appender.push_str("<span class=\"error\">ERROR while parsing: ");
                appender.push_cow_str(self.html_escaper.escape(message));
                appender.push_str("</span>");
            }
            dom::Part::RSTRef { text, r#ref: _ } => {
                self.append_tag(appender, "<span>", text, "</span>")
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

pub static PLAIN_HTML_FORMATTER: LazyLock<PlainHTMLFormatter> =
    LazyLock::new(|| PlainHTMLFormatter::new());

/// Apply the plain HTML formatter to all parts of the given paragraph, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the plain HTML formatter.
pub fn append_plain_html_paragraph<'a, I>(
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
        &*PLAIN_HTML_FORMATTER,
        link_provider,
        "<p>",
        "</p>",
        "",
        current_plugin,
    );
}

/// Apply the plain HTML formatter to all parts of the given paragraphs, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the plain HTML formatter.
pub fn append_plain_html_paragraphs<'a, I, II>(
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
        &*PLAIN_HTML_FORMATTER,
        link_provider,
        "<p>",
        "</p>",
        "",
        "",
        current_plugin,
    );
}
