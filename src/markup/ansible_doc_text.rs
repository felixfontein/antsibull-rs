/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use crate::markup::dom;
use crate::markup::format;
use crate::util::stringbuilder::Appender;
use std::rc::Rc;
use std::sync::LazyLock;

pub struct AnsibleDocTextFormatter {}

impl AnsibleDocTextFormatter {
    fn new() -> AnsibleDocTextFormatter {
        AnsibleDocTextFormatter {}
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
        appender.push_str(text);
        appender.push_str(end);
    }

    #[inline]
    fn append_fqcn<'a>(&self, appender: &mut dyn Appender<'a>, fqcn: &'a str) {
        appender.push_str("[");
        appender.push_str(fqcn);
        appender.push_str("]");
    }

    #[inline]
    fn append_option_like<'a>(
        &self,
        appender: &mut dyn Appender<'a>,
        name: &'a String,
        value: &'a Option<String>,
        plugin: &Option<Rc<dom::PluginIdentifier>>,
        entrypoint: &Option<Rc<String>>,
    ) {
        appender.push_str("`");
        appender.push_string(name);
        if let Some(v) = value {
            appender.push_str("=");
            appender.push_string(v);
        }
        appender.push_str("'");
        if let Some(p) = plugin {
            appender.push_str(" (of ");
            appender.push_borrowed_string(&p.r#type);
            if !matches!(p.r#type.as_str(), "role" | "module" | "playbook") {
                appender.push_str(" plugin");
            }
            appender.push_str(" ");
            appender.push_borrowed_string(&p.fqcn);
            if p.r#type == "role" {
                if let Some(ep) = entrypoint {
                    appender.push_str(", ");
                    appender.push_borrowed_string(ep);
                    appender.push_str(" entrypoint");
                }
            }
            appender.push_str(")");
        }
    }
}

impl<'a> format::Formatter<'a> for AnsibleDocTextFormatter {
    fn append(
        &self,
        appender: &mut dyn Appender<'a>,
        part: &'a dom::Part<'a>,
        _url: Option<String>,
    ) {
        match part {
            dom::Part::Text { text } => appender.push_str(text),
            dom::Part::Bold { text } => self.append_tag(appender, "*", text, "*"),
            dom::Part::Italic { text } => self.append_tag(appender, "`", text, "'"),
            dom::Part::Code { text } => self.append_tag(appender, "`", text, "'"),
            dom::Part::HorizontalLine => appender.push_str("\n-------------\n"),
            dom::Part::OptionValue { value } => self.append_tag(appender, "`", value, "'"),
            dom::Part::EnvVariable { name } => self.append_tag(appender, "`", name, "'"),
            dom::Part::Error { message } => {
                appender.push_str("[[ERROR while parsing: ");
                appender.push_string(message);
                appender.push_str("]]");
            }
            dom::Part::RSTRef { text, r#ref: _ } => appender.push_str(text),
            dom::Part::Link { text, url } => {
                appender.push_str(text);
                appender.push_str(" <");
                appender.push_str(url);
                appender.push_str(">");
            }
            dom::Part::URL { url } => appender.push_str(url),
            dom::Part::Module { fqcn } => self.append_fqcn(appender, &fqcn),
            dom::Part::Plugin { plugin } => self.append_fqcn(appender, &plugin.fqcn),
            dom::Part::OptionName {
                plugin,
                entrypoint,
                link: _,
                name,
                value,
            } => self.append_option_like(appender, name, value, plugin, entrypoint),
            dom::Part::ReturnValue {
                plugin,
                entrypoint,
                link: _,
                name,
                value,
            } => self.append_option_like(appender, name, value, plugin, entrypoint),
        };
    }
}

pub static ANSIBLE_DOC_TEXT_FORMATTER: LazyLock<AnsibleDocTextFormatter> =
    LazyLock::new(|| AnsibleDocTextFormatter::new());

/// Apply the ansible-doc text formatter to all parts of the given paragraph, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the ansible-doc text formatter.
pub fn append_ansible_doc_text_paragraph<'a, I>(
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
        &*ANSIBLE_DOC_TEXT_FORMATTER,
        link_provider,
        "",
        "",
        "",
        current_plugin,
    );
}

/// Apply the ansible-doc text formater to all parts of the given paragraphs, and concatenate the results.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the ansible-doc text formatter.
pub fn append_ansible_doc_text_paragraphs<'a, I, II>(
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
        &*ANSIBLE_DOC_TEXT_FORMATTER,
        link_provider,
        "",
        "",
        "\n\n",
        "",
        current_plugin,
    );
}
