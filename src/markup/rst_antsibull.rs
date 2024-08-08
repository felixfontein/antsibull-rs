/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use crate::markup::dom;
use crate::markup::format;
use crate::markup::html_helper;
use crate::markup::rst_helper;
use crate::util::stringbuilder;
use crate::util::stringbuilder::{Appender, IntoString};
use std::rc::Rc;
use std::sync::LazyLock;

pub struct AntsibullRSTFormatter {
    rst_escaper: rst_helper::RSTEscaper,
    url_escaper: html_helper::URLEscaper,
}

impl AntsibullRSTFormatter {
    fn new() -> AntsibullRSTFormatter {
        AntsibullRSTFormatter {
            rst_escaper: rst_helper::RSTEscaper::new(),
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
        appender.push_cow_str(self.rst_escaper.escape(text, true, true));
        appender.push_str(end);
    }

    #[inline]
    fn append_link<'a>(&self, appender: &mut dyn Appender<'a>, text: &'a str, url: &'a str) {
        if text.len() == 0 {
            return;
        }
        if url.len() == 0 {
            appender.push_cow_str(self.rst_escaper.escape(text, false, false));
            return;
        }
        appender.push_str("\\ `");
        appender.push_cow_str(self.rst_escaper.escape(text, true, false));
        appender.push_str(" <");
        appender.push_cow_str(self.url_escaper.escape(url));
        appender.push_str(">`__\\ ");
    }

    #[inline]
    fn append_fqcn<'a>(&self, appender: &mut dyn Appender<'a>, fqcn: &'a str, r#type: &'a str) {
        appender.push_str("\\ :ref:`");
        appender.push_cow_str(self.rst_escaper.escape(fqcn, false, false));
        appender.push_str(" <ansible_collections.");
        appender.push_str(fqcn);
        appender.push_str("_");
        appender.push_str(r#type);
        appender.push_str(">`\\ ");
    }

    #[inline]
    fn append_option_like<'a>(
        &self,
        appender: &mut dyn Appender<'a>,
        plugin: &'a Option<Rc<dom::PluginIdentifier>>,
        entrypoint: &'a Option<Rc<String>>,
        name: &'a String,
        value: &'a Option<String>,
        what: format::OptionLike,
    ) {
        appender.push_str("\\ :");
        appender.push_str(match what {
            format::OptionLike::Option => "ansopt",
            format::OptionLike::RetVal => "ansretval",
        });
        appender.push_str(":`");
        let mut builder = stringbuilder::StringAppender::new();
        if let Some(p) = plugin {
            builder.push_str(&p.fqcn);
            builder.push_str("#");
            builder.push_str(&p.r#type);
            builder.push_str(":");
        }
        if let Some(ep) = entrypoint {
            builder.push_str(&ep);
            builder.push_str(":");
        }
        builder.push_str(name);
        if let Some(v) = value {
            builder.push_str("=");
            builder.push_str(&v);
        }
        appender.push_owned_string(
            self.rst_escaper
                .escape(&builder.into_string(), true, true)
                .into_owned(),
        );
        appender.push_str("`\\ ");
    }
}

impl<'a> format::Formatter<'a> for AntsibullRSTFormatter {
    fn append(
        &self,
        appender: &mut dyn Appender<'a>,
        part: &'a dom::Part<'a>,
        _url: Option<String>,
    ) {
        match part {
            dom::Part::Text { text } => {
                appender.push_cow_str(self.rst_escaper.escape(text, false, false))
            }
            dom::Part::Bold { text } => self.append_tag(appender, "\\ :strong:`", text, "`\\ "),
            dom::Part::Italic { text } => self.append_tag(appender, "\\ :emphasis:`", text, "`\\ "),
            dom::Part::Code { text } => self.append_tag(appender, "\\ :literal:`", text, "`\\ "),
            dom::Part::HorizontalLine => appender.push_str("\n\n.. raw:: html\n\n  <hr>\n\n"),
            dom::Part::OptionValue { value } => {
                self.append_tag(appender, "\\ :ansval:`", value, "`\\ ")
            }
            dom::Part::EnvVariable { name } => {
                self.append_tag(appender, "\\ :envvar:`", name, "`\\ ")
            }
            dom::Part::Error { message } => {
                appender.push_str("\\ :strong:`ERROR while parsing`\\ : ");
                appender.push_cow_str(self.rst_escaper.escape(message, true, true));
                appender.push_str("\\ ");
            }
            dom::Part::RSTRef { text, r#ref } => {
                appender.push_str("\\ :ref:`");
                appender.push_cow_str(self.rst_escaper.escape(text, true, true));
                appender.push_str(" <");
                appender.push_str(r#ref);
                appender.push_str(">`\\ ");
            }
            dom::Part::Link { text, url } => self.append_link(appender, text, url),
            dom::Part::URL { url } => self.append_link(appender, url, url),
            dom::Part::Module { fqcn } => self.append_fqcn(appender, &fqcn, "module"),
            dom::Part::Plugin { plugin } => {
                self.append_fqcn(appender, &plugin.fqcn, &plugin.r#type)
            }
            dom::Part::OptionName {
                plugin,
                entrypoint,
                link: _,
                name,
                value,
            } => self.append_option_like(
                appender,
                plugin,
                entrypoint,
                name,
                value,
                format::OptionLike::Option,
            ),
            dom::Part::ReturnValue {
                plugin,
                entrypoint,
                link: _,
                name,
                value,
            } => self.append_option_like(
                appender,
                plugin,
                entrypoint,
                name,
                value,
                format::OptionLike::RetVal,
            ),
        };
    }
}

pub static ANTSIBULL_RST_FORMATTER: LazyLock<AntsibullRSTFormatter> =
    LazyLock::new(|| AntsibullRSTFormatter::new());

/// Apply the Antsibull RST formatter to all parts of the given paragraph, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the Antsibull RST formatter.
pub fn append_antsibull_rst_paragraph<'a, I>(
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
        &*ANTSIBULL_RST_FORMATTER,
        link_provider,
        "",
        "",
        "\\ ",
        current_plugin,
    );
}

/// Apply the Antsibull RST formatter to all parts of the given paragraphs, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the Antsibull RST formatter.
pub fn append_antsibull_rst_paragraphs<'a, I, II>(
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
        &*ANTSIBULL_RST_FORMATTER,
        link_provider,
        "",
        "",
        "\n\n",
        "\\ ",
        current_plugin,
    );
}
