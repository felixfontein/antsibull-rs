/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

//! Ansible markup parsing and rendering functionality.

mod ansible_doc_text;
mod dom;
mod format;
mod html_antsibull;
mod html_helper;
mod html_plain;
mod md;
mod md_helper;
mod parse;
mod rst_antsibull;
mod rst_helper;
mod rst_plain;

pub use ansible_doc_text::{
    append_ansible_doc_text_paragraph, append_ansible_doc_text_paragraphs, AnsibleDocTextFormatter,
};

pub use dom::{Part, PartWithSource, PluginIdentifier};

pub use parse::{
    parse, parse_paragraphs, parse_paragraphs_without_sources, parse_without_sources, Context,
    ParseOptions,
};

pub use format::{
    append_paragraph, append_paragraphs, Formatter, LinkProvider, NoLinkProvider, OptionLike,
};

pub use html_helper::{HTMLEscaper, URLEscaper};

pub use html_antsibull::{
    append_antsibull_html_paragraph, append_antsibull_html_paragraphs, AntsibullHTMLFormatter,
};

pub use html_plain::{
    append_plain_html_paragraph, append_plain_html_paragraphs, PlainHTMLFormatter,
};

pub use md::{append_md_paragraph, append_md_paragraphs, MDFormatter};

pub use md_helper::MDEscaper;

pub use rst_antsibull::{
    append_antsibull_rst_paragraph, append_antsibull_rst_paragraphs, AntsibullRSTFormatter,
};

pub use rst_helper::RSTEscaper;

pub use rst_plain::{append_plain_rst_paragraph, append_plain_rst_paragraphs, PlainRSTFormatter};

#[cfg(test)]
mod tests {
    use crate::markup::{
        append_ansible_doc_text_paragraphs, append_antsibull_html_paragraphs,
        append_antsibull_rst_paragraphs, append_md_paragraphs, append_plain_html_paragraphs,
        append_plain_rst_paragraphs, dom, parse, parse_paragraphs, LinkProvider, NoLinkProvider,
        OptionLike, ParseOptions, PluginIdentifier,
    };
    use crate::util::{CollectorAppender, IntoString};
    use saphyr::{Hash, Yaml};
    use std::fs::File;
    use std::io::Read;
    use std::rc::Rc;

    struct TemplatedLinkProvider {
        plugin_link: Option<String>,
        plugin_option_like_link: Option<String>,
    }

    impl TemplatedLinkProvider {
        pub fn new(
            plugin_link: &Option<String>,
            plugin_option_like_link: &Option<String>,
        ) -> Result<TemplatedLinkProvider, String> {
            // TODO do some basic checking
            Ok(TemplatedLinkProvider {
                plugin_link: plugin_link.clone(),
                plugin_option_like_link: plugin_option_like_link.clone(),
            })
        }

        pub fn parse(opts: &Hash) -> Result<TemplatedLinkProvider, String> {
            TemplatedLinkProvider::new(
                &opts
                    .get(&Yaml::from_str("pluginLinkTemplate"))
                    .map(|v| v.as_str().unwrap().to_string()),
                &opts
                    .get(&Yaml::from_str("pluginOptionLikeLinkTemplate"))
                    .map(|v| v.as_str().unwrap().to_string()),
            )
        }
    }

    impl LinkProvider for TemplatedLinkProvider {
        fn plugin_link(&self, plugin: &dom::PluginIdentifier) -> Option<String> {
            match &self.plugin_link {
                Some(template) => Some(
                    template
                        .replace("{plugin_fqcn}", &plugin.fqcn)
                        .replace("{plugin_fqcn_slashes}", &plugin.fqcn.replace(".", "/"))
                        .replace("{plugin_type}", &plugin.r#type),
                ),
                None => None,
            }
        }

        fn plugin_option_like_link(
            &self,
            plugin: &dom::PluginIdentifier,
            entrypoint: Option<&String>,
            what: OptionLike,
            name: &[String],
            _current_plugin: bool,
        ) -> Option<String> {
            match &self.plugin_option_like_link {
                Some(template) => Some(
                    template
                        .replace("{plugin_fqcn}", &plugin.fqcn)
                        .replace("{plugin_fqcn_slashes}", &plugin.fqcn.replace(".", "/"))
                        .replace("{plugin_type}", &plugin.r#type)
                        .replace(
                            "{what}",
                            match what {
                                OptionLike::Option => "option",
                                OptionLike::RetVal => "retval",
                            },
                        )
                        .replace(
                            "{entrypoint}",
                            &entrypoint.map(|v| v.as_str()).unwrap_or(""),
                        )
                        .replace(
                            "{entrypoint_with_leading_dash}",
                            &entrypoint
                                .map(|ep| format!("-{}", ep))
                                .unwrap_or_else(|| "".to_string()),
                        )
                        .replace("{name_dots}", &name.join("."))
                        .replace("{name_slashes}", &name.join("/")),
                ),
                None => None,
            }
        }
    }

    fn parse_current_plugin(opts: &Hash) -> Result<Option<Rc<dom::PluginIdentifier>>, String> {
        match opts.get(&Yaml::from_str("currentPlugin")) {
            Some(cp) => {
                let current_plugin = cp.as_hash().unwrap();
                Ok(Some(Rc::new(dom::PluginIdentifier {
                    fqcn: current_plugin
                        .get(&Yaml::from_str("fqcn"))
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    r#type: current_plugin
                        .get(&Yaml::from_str("type"))
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                })))
            }
            None => Ok(None),
        }
    }

    fn get_render_options(
        params: &Hash,
        name: &str,
    ) -> (Option<Rc<dom::PluginIdentifier>>, Box<dyn LinkProvider>) {
        let mut current_plugin: Option<Rc<dom::PluginIdentifier>> = None;
        let mut link_provider: Box<dyn LinkProvider> = Box::new(NoLinkProvider::new());
        if let Some(o) = &params.get(&Yaml::from_str(name)) {
            let opts = o.as_hash().unwrap();
            link_provider = Box::new(TemplatedLinkProvider::parse(opts).unwrap());
            current_plugin = parse_current_plugin(opts).unwrap();
        }
        (current_plugin, link_provider)
    }

    fn get_context_options(params: &Hash) -> (parse::Context, ParseOptions) {
        let mut context = parse::Context {
            current_plugin: None,
            role_entrypoint: None,
        };
        let mut options = ParseOptions::default();
        if let Some(parse_opts_t) = &params.get(&Yaml::from_str("parse_opts")) {
            let parse_opts = parse_opts_t.as_hash().unwrap();
            if let Some(f) = &parse_opts.get(&Yaml::from_str("currentPlugin")) {
                let current_plugin = &f.as_hash().unwrap();
                context.current_plugin = Some(Rc::new(PluginIdentifier {
                    fqcn: current_plugin[&Yaml::from_str("fqcn")]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    r#type: current_plugin[&Yaml::from_str("type")]
                        .as_str()
                        .unwrap()
                        .to_string(),
                }));
            }
            if let Some(f) = &parse_opts.get(&Yaml::from_str("roleEntrypoint")) {
                context.role_entrypoint = Some(Rc::new(f.as_str().unwrap().to_string()));
            }
            if let Some(f) = &parse_opts.get(&Yaml::from_str("onlyClassicMarkup")) {
                if f.as_bool().unwrap() {
                    options = options.only_classic_markup();
                }
            }
            if let Some(f) = &parse_opts.get(&Yaml::from_str("helpfulErrors")) {
                if !f.as_bool().unwrap() {
                    options = options.unhelpful_errors();
                }
            }
        };
        (context, options)
    }

    #[test]
    fn test_vectors() {
        let mut contents = String::new();
        File::open("test-vectors.yaml")
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();
        let yaml_data = Yaml::load_from_str(&contents).unwrap();
        let data = yaml_data[0].as_hash().unwrap()[&Yaml::from_str("test_vectors")]
            .as_hash()
            .unwrap();
        for (k, v) in data {
            let name = k.as_str().unwrap();
            println!("Processing test {:?}...", name);
            let params = v.as_hash().unwrap();
            let (context, options) = get_context_options(params);
            let parsed_source = match &params[&Yaml::from_str("source")] {
                Yaml::String(s) => vec![parse(&s, &context, &options)],
                Yaml::Array(a) => {
                    parse_paragraphs(a.iter().map(|s| s.as_str().unwrap()), &context, &options)
                }
                v => panic!("Unknown type: {:?}", v),
            };
            if let Some(f) = &params.get(&Yaml::from_str("html")) {
                let expected = f.as_str().unwrap();
                let (current_plugin, link_provider) = get_render_options(params, "html_opts");
                let mut appender = CollectorAppender::new();
                append_antsibull_html_paragraphs(
                    &mut appender,
                    parsed_source
                        .iter()
                        .map(|paragraph| paragraph.iter().map(|ps| &ps.part)),
                    &*link_provider,
                    &current_plugin,
                );
                assert_eq!(appender.into_string(), expected);
            }
            if let Some(f) = &params.get(&Yaml::from_str("html_plain")) {
                let expected = f.as_str().unwrap();
                let (current_plugin, link_provider) = get_render_options(params, "html_opts");
                let mut appender = CollectorAppender::new();
                append_plain_html_paragraphs(
                    &mut appender,
                    parsed_source
                        .iter()
                        .map(|paragraph| paragraph.iter().map(|ps| &ps.part)),
                    &*link_provider,
                    &current_plugin,
                );
                assert_eq!(appender.into_string(), expected);
            }
            if let Some(f) = &params.get(&Yaml::from_str("md")) {
                let expected = f.as_str().unwrap();
                let (current_plugin, link_provider) = get_render_options(params, "md_opts");
                let mut appender = CollectorAppender::new();
                append_md_paragraphs(
                    &mut appender,
                    parsed_source
                        .iter()
                        .map(|paragraph| paragraph.iter().map(|ps| &ps.part)),
                    &*link_provider,
                    &current_plugin,
                );
                assert_eq!(appender.into_string(), expected);
            }
            if let Some(f) = &params.get(&Yaml::from_str("rst")) {
                let expected = f.as_str().unwrap();
                let (current_plugin, link_provider) = get_render_options(params, "rst_opts");
                let mut appender = CollectorAppender::new();
                append_antsibull_rst_paragraphs(
                    &mut appender,
                    parsed_source
                        .iter()
                        .map(|paragraph| paragraph.iter().map(|ps| &ps.part)),
                    &*link_provider,
                    &current_plugin,
                );
                assert_eq!(appender.into_string(), expected);
            }
            if let Some(f) = &params.get(&Yaml::from_str("rst_plain")) {
                let expected = f.as_str().unwrap();
                let (current_plugin, link_provider) = get_render_options(params, "rst_opts");
                let mut appender = CollectorAppender::new();
                append_plain_rst_paragraphs(
                    &mut appender,
                    parsed_source
                        .iter()
                        .map(|paragraph| paragraph.iter().map(|ps| &ps.part)),
                    &*link_provider,
                    &current_plugin,
                );
                assert_eq!(appender.into_string(), expected);
            }
            if let Some(f) = &params.get(&Yaml::from_str("ansible_doc_text")) {
                let expected = f.as_str().unwrap();
                let (current_plugin, link_provider) =
                    get_render_options(params, "ansible_doc_text_opts");
                let mut appender = CollectorAppender::new();
                append_ansible_doc_text_paragraphs(
                    &mut appender,
                    parsed_source
                        .iter()
                        .map(|paragraph| paragraph.iter().map(|ps| &ps.part)),
                    &*link_provider,
                    &current_plugin,
                );
                assert_eq!(appender.into_string(), expected);
            }
        }
    }
}
