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

pub struct PlainRSTFormatter {
    rst_escaper: rst_helper::RSTEscaper,
    url_escaper: html_helper::URLEscaper,
}

impl PlainRSTFormatter {
    fn new() -> PlainRSTFormatter {
        PlainRSTFormatter {
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
    ) {
        appender.push_str("\\ :literal:`");

        let mut builder = stringbuilder::StringAppender::new();
        builder.push_str(&name);
        if let Some(v) = value {
            builder.push_str("=");
            builder.push_str(&v);
        }
        appender.push_owned_string(
            self.rst_escaper
                .escape(&builder.into_string(), true, true)
                .into_owned(),
        );
        appender.push_str("`");

        let escaped_ep = entrypoint
            .as_ref()
            .map(|ep| self.rst_escaper.escape(&*ep, true, true).into_owned())
            .unwrap_or("".to_string());
        let mut plugin_result: Vec<&'a str> = Vec::with_capacity(11);
        if let Some(p) = plugin {
            plugin_result.push(&p.r#type);
            if !matches!(p.r#type.as_str(), "module" | "role" | "playbook") {
                plugin_result.push(" plugin");
            }
            plugin_result.push(" :ref:`");
            plugin_result.push(&p.fqcn);
            plugin_result.push(" <ansible_collections.");
            plugin_result.push(&p.fqcn);
            plugin_result.push("_");
            plugin_result.push(&p.r#type);
            plugin_result.push(">`");
        }
        if let Some(_) = entrypoint {
            if plugin_result.len() > 0 {
                plugin_result.push(", ");
            }
            plugin_result.push("entrypoint ");
            // escaped_ep will be added below
        }
        if plugin_result.len() > 0 {
            appender.push_str(" (of ");
            for v in plugin_result {
                appender.push_str(v);
            }
            appender.push_owned_string(escaped_ep);
            appender.push_str(")");
        }

        appender.push_str("\\ ");
    }
}

impl<'a> format::Formatter<'a> for PlainRSTFormatter {
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
            dom::Part::HorizontalLine => appender.push_str("\n\n------------\n\n"),
            dom::Part::OptionValue { value } => {
                self.append_tag(appender, "\\ :literal:`", value, "`\\ ")
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
            } => self.append_option_like(appender, plugin, entrypoint, name, value),
            dom::Part::ReturnValue {
                plugin,
                entrypoint,
                link: _,
                name,
                value,
            } => self.append_option_like(appender, plugin, entrypoint, name, value),
        };
    }
}

pub static PLAIN_RST_FORMATTER: LazyLock<PlainRSTFormatter> =
    LazyLock::new(|| PlainRSTFormatter::new());

/// Apply the plain RST formatter to all parts of the given paragraph, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the plain RST formatter.
pub fn append_plain_rst_paragraph<'a, I>(
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
        &*PLAIN_RST_FORMATTER,
        link_provider,
        "",
        "",
        "\\ ",
        current_plugin,
    );
}

/// Apply the plain RST formatter to all parts of the given paragraphs, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the plain RST formatter.
pub fn append_plain_rst_paragraphs<'a, I, II>(
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
        &*PLAIN_RST_FORMATTER,
        link_provider,
        "",
        "",
        "\n\n",
        "\\ ",
        current_plugin,
    );
}
