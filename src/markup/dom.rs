/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use std::fmt;
use std::rc::Rc;

/// Identifies a plugin by FQCN and plugin type.
#[derive(Debug, Clone, PartialEq)]
pub struct PluginIdentifier {
    /// The FQCN of the plugin.
    pub fqcn: String,

    /// The plugin type of the plugin.
    ///
    /// The list of valid plugin types depends on the ansible-core version.
    /// Possible values are (as of ansible-core 2.17):
    /// `become`, `cache`, `callback`, `cliconf`, `connection`, `httpapi`, `inventory`,
    /// `lookup`, `netconf`, `shell`, `vars`, `module`, `strategy`, `test`, `filter`,
    /// and `role`.
    pub r#type: String,
}

impl fmt::Display for PluginIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.fqcn, self.r#type)
    }
}

/// A markup element (part).
///
/// Describes a part of a paragraph. These parts are concatenated without separators
/// to form the paragraph.
#[derive(Debug, PartialEq)]
pub enum Part<'a> {
    /// Some plain text.
    Text { text: &'a str },

    /// Italic text.
    Italic { text: &'a str },

    /// Bold text.
    Bold { text: &'a str },

    /// Code-formatted (teletype) text.
    Code { text: &'a str },

    /// Link to a module by FQCN.
    Module { fqcn: &'a str },

    /// Link to a plugin by FQCN and plugin type.
    Plugin { plugin: PluginIdentifier },

    /// An URL.
    URL { url: &'a str },

    /// A link with title and URL.
    Link { text: &'a str, url: &'a str },

    /// A RST ([ReStructured Text](https://en.wikipedia.org/wiki/ReStructured_Text))
    /// reference with title.
    RSTRef { text: &'a str, r#ref: &'a str },

    /// Reference to an option name, with optional value.
    OptionName {
        /// The plugin this is an option for.
        plugin: Option<Rc<PluginIdentifier>>,

        /// The role entrypoint this is an option for.
        entrypoint: Option<Rc<String>>,

        /// The option name preceeded by its parents.
        ///
        /// For example, for the option `foo.bar.baz`, which is the
        /// suboption `baz` of `bar`, which in turn is a suboption of
        /// the top-level option `foo`, this is the list `["foo", "bar", "baz"]`.
        /// This list does not contain array stubs.
        link: Box<[String]>,

        /// The option name, including array stubs.
        ///
        /// For example `foo[1].bar[].baz`.
        name: String,

        /// The option's value, if present.
        value: Option<String>,
    },

    /// Option value.
    OptionValue { value: String },

    /// Environment variable.
    EnvVariable { name: String },

    /// Reference to a return value, with optional value.
    ReturnValue {
        /// The plugin this is a return value for.
        plugin: Option<Rc<PluginIdentifier>>,

        /// The role entrypoint this is a return value for.
        entrypoint: Option<Rc<String>>,

        /// The return value name preceeded by its parents.
        ///
        /// For example, for the return value `foo.bar.baz`, which is the
        /// sub-return value `baz` of `bar`, which in turn is a sub-return value
        /// of the top-level return value `foo`, this is the list
        /// `["foo", "bar", "baz"]`. This list does not contain array stubs.
        link: Box<[String]>,

        /// The return value name, including array stubs.
        ///
        /// For example `foo[1].bar[].baz`.
        name: String,

        /// The return value's value, if present.
        value: Option<String>,
    },

    /// A horizontal line as a separator.
    HorizontalLine,

    /// An error message.
    ///
    /// Usually reports parsing errors.
    Error { message: String },
}

impl<'a> fmt::Display for Part<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Part::Text { text } => {
                write!(f, "text={:?}", text)
            }
            Part::Italic { text } => {
                write!(f, "italic={:?}", text)
            }
            Part::Bold { text } => {
                write!(f, "bold={:?}", text)
            }
            Part::Code { text } => {
                write!(f, "code={:?}", text)
            }
            Part::Module { fqcn } => {
                write!(f, "module={}", fqcn)
            }
            Part::Plugin { plugin } => {
                write!(f, "plugin={}:{}", plugin.fqcn, plugin.r#type)
            }
            Part::URL { url } => {
                write!(f, "url={:?}", url)
            }
            Part::Link { text, url } => {
                write!(f, "link={:?}->{:?}", text, url)
            }
            Part::RSTRef { text, r#ref } => {
                write!(f, "RSTref={:?}->{:?}", text, r#ref)
            }
            Part::OptionName {
                plugin,
                entrypoint,
                link,
                name,
                value,
            } => {
                write!(
                    f,
                    "option={{plugin={:?}, entrypoint={:?}, link={:?}, name={:?}, value={:?}}}",
                    plugin, entrypoint, link, name, value
                )
            }
            Part::OptionValue { value } => {
                write!(f, "option-value={:?}", value)
            }
            Part::EnvVariable { name } => {
                write!(f, "env-variable={:?}", name)
            }
            Part::ReturnValue {
                plugin,
                entrypoint,
                link,
                name,
                value,
            } => {
                write!(f, "return-value={{plugin={:?}, entrypoint={:?}, link={:?}, name={:?}, value={:?}}}", plugin, entrypoint, link, name, value)
            }
            Part::HorizontalLine => {
                write!(f, "horizontal-line")
            }
            Part::Error { message } => {
                write!(f, "error={:?}", message)
            }
        }
    }
}

/// A markup element (part) together with its source string.
#[derive(Debug, PartialEq)]
pub struct PartWithSource<'a> {
    /// The DOM part.
    pub part: Part<'a>,

    /// The source string that resulted in the DOM part.
    pub source: &'a str,
}

impl<'a> fmt::Display for PartWithSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}; source={:?})", self.part, self.source)
    }
}
