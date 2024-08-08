/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use crate::markup::dom;
use crate::util::stringbuilder::Appender;
use std::rc::Rc;

pub trait Formatter<'a> {
    fn append(&self, appender: &mut dyn Appender<'a>, part: &'a dom::Part<'a>, url: Option<String>);
}

pub enum OptionLike {
    Option,
    RetVal,
}

pub trait LinkProvider {
    fn plugin_link(&self, plugin: &dom::PluginIdentifier) -> Option<String>;
    fn plugin_option_like_link(
        &self,
        plugin: &dom::PluginIdentifier,
        entrypoint: Option<&String>,
        what: OptionLike,
        name: &[String],
        current_plugin: bool,
    ) -> Option<String>;
}

pub struct NoLinkProvider {}

impl NoLinkProvider {
    pub fn new() -> NoLinkProvider {
        NoLinkProvider {}
    }
}

impl LinkProvider for NoLinkProvider {
    fn plugin_link(&self, _plugin: &dom::PluginIdentifier) -> Option<String> {
        None
    }

    fn plugin_option_like_link(
        &self,
        _plugin: &dom::PluginIdentifier,
        _entrypoint: Option<&String>,
        _what: OptionLike,
        _name: &[String],
        _current_plugin: bool,
    ) -> Option<String> {
        None
    }
}

/// Apply the formatter to all parts of the given paragraph, concatenate the results, and insert start and end sequences for the paragraph.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the formatter.
pub fn append_paragraph<'a, I>(
    appender: &mut dyn Appender<'a>,
    paragraph: I,
    formatter: &dyn Formatter<'a>,
    link_provider: &dyn LinkProvider,
    par_start: &'a str,
    par_end: &'a str,
    par_empty: &'a str,
    current_plugin: &Option<Rc<dom::PluginIdentifier>>,
) where
    I: Iterator<Item = &'a dom::Part<'a>>,
{
    appender.push_str(par_start);
    let mut first = true;
    for part in paragraph {
        first = false;
        let url: Option<String> = match part {
            dom::Part::Module { fqcn } => link_provider.plugin_link(&dom::PluginIdentifier {
                fqcn: fqcn.to_string(),
                r#type: "module".to_string(),
            }),
            dom::Part::Plugin { plugin } => link_provider.plugin_link(&plugin),
            dom::Part::OptionName {
                plugin,
                entrypoint,
                link,
                name: _,
                value: _,
            } => match plugin.as_ref() {
                Some(rcp) => link_provider.plugin_option_like_link(
                    &*rcp,
                    entrypoint.as_ref().map(|s| &**s),
                    OptionLike::Option,
                    &*link,
                    match current_plugin.as_ref() {
                        Some(cp) => *rcp == *cp,
                        None => false,
                    },
                ),
                None => None,
            },
            dom::Part::ReturnValue {
                plugin,
                entrypoint,
                link,
                name: _,
                value: _,
            } => match plugin.as_ref() {
                Some(rcp) => link_provider.plugin_option_like_link(
                    &*rcp,
                    entrypoint.as_ref().map(|s| &**s),
                    OptionLike::RetVal,
                    &*link,
                    match current_plugin.as_ref() {
                        Some(cp) => *rcp == *cp,
                        None => false,
                    },
                ),
                None => None,
            },
            _ => None,
        };
        formatter.append(appender, part, url);
    }
    if first {
        appender.push_str(par_empty);
    }
    appender.push_str(par_end);
}

/// Apply the formatter to all parts of the given paragraphs, concatenate the results, and insert start and end sequences for paragraphs and sequences between paragraphs.
///
/// `link_provider` and `current_plugin` will be used to compute optional URLs that will be passed to the formatter.
pub fn append_paragraphs<'a, I, II>(
    appender: &mut dyn Appender<'a>,
    paragraphs: I,
    formatter: &dyn Formatter<'a>,
    link_provider: &dyn LinkProvider,
    par_start: &'a str,
    par_end: &'a str,
    par_sep: &'a str,
    par_empty: &'a str,
    current_plugin: &Option<Rc<dom::PluginIdentifier>>,
) where
    I: IntoIterator<Item = II>,
    II: Iterator<Item = &'a dom::Part<'a>>,
{
    let mut first = true;
    for paragraph in paragraphs {
        if first {
            first = false;
        } else {
            appender.push_str(&par_sep);
        }
        append_paragraph(
            appender,
            paragraph,
            formatter,
            link_provider,
            par_start,
            par_end,
            par_empty,
            current_plugin,
        );
    }
}
