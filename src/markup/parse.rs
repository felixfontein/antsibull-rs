/*
GNU General Public License v3.0+ (see LICENSES/GPL-3.0-or-later.txt or https://www.gnu.org/licenses/gpl-3.0.txt)
SPDX-FileCopyrightText: 2024, Felix Fontein
SPDX-License-Identifier: GPL-3.0-or-later
*/

use crate::markup::dom;
use crate::util::stringbuilder;
use crate::util::stringbuilder::{Appender, IntoString};

use regex;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::LazyLock;

const IGNORE_MARKER: &'static str = "ignore:";

struct Command<'a> {
    command: &'a str,
    command_match: &'a str,
    parameters: u32,
    escaped_arguments: bool,
    old_markup: bool,
}

impl<'a> Command<'a> {
    const fn new_classic(command: &'a str, command_match: &'a str, parameters: u32) -> Command<'a> {
        return Command {
            command: command,
            command_match: command_match,
            parameters: parameters,
            escaped_arguments: false,
            old_markup: true,
        };
    }

    const fn new_modern(command: &'a str, command_match: &'a str, parameters: u32) -> Command<'a> {
        return Command {
            command: command,
            command_match: command_match,
            parameters: parameters,
            escaped_arguments: true,
            old_markup: false,
        };
    }
}

const ITALICS: Command<'static> = Command::new_classic("I", "I(", 1);
const BOLD: Command<'static> = Command::new_classic("B", "B(", 1);
const MODULE: Command<'static> = Command::new_classic("M", "M(", 1);
const URL: Command<'static> = Command::new_classic("U", "U(", 1);
const LINK: Command<'static> = Command::new_classic("L", "L(", 2);
const RSTREF: Command<'static> = Command::new_classic("R", "R(", 2);
const CODE: Command<'static> = Command::new_classic("C", "C(", 1);
const HORIZONTAL_LINE: Command<'static> =
    Command::new_classic("HORIZONTALLINE", "HORIZONTALLINE", 0);
const PLUGIN: Command<'static> = Command::new_modern("P", "P(", 1);
const ENVVAR: Command<'static> = Command::new_modern("E", "E(", 1);
const OPTION_VALUE: Command<'static> = Command::new_modern("V", "V(", 1);
const OPTION_NAME: Command<'static> = Command::new_modern("O", "O(", 1);
const RETURN_VALUE: Command<'static> = Command::new_modern("RV", "RV(", 1);

const ALL_COMMANDS: [Command<'static>; 13] = [
    ITALICS,
    BOLD,
    MODULE,
    URL,
    LINK,
    RSTREF,
    CODE,
    HORIZONTAL_LINE,
    PLUGIN,
    ENVVAR,
    OPTION_VALUE,
    OPTION_NAME,
    RETURN_VALUE,
];

struct Parser<'a> {
    command_map: HashMap<&'a str, &'a Command<'a>>,
    regex: regex::Regex,
    escape_or_comma: regex::Regex,
    escape_or_closing: regex::Regex,
    fqcn_re: regex::Regex,
    plugin_type_re: regex::Regex,
    array_stub_re: regex::Regex,
    fqcn_type_prefix_re: regex::Regex,
}

fn _map_re_error<T>(result: Result<T, regex::Error>) -> Result<T, String> {
    result.map_err(|error| format!("Compiling regular expression: {}", error))
}

impl<'a> Parser<'a> {
    fn new<'b>(commands: &'b [&'a Command<'a>]) -> Result<Parser<'a>, String> {
        let mut regex_buf = String::new();
        let mut command_map: HashMap<&'a str, &'a Command<'a>> = HashMap::new();
        if commands.len() == 0 {
            regex_buf.push_str("x^"); // does not match anything
        } else {
            regex_buf.push_str("(");
            for (index, command) in commands.into_iter().enumerate() {
                match command_map.insert(command.command_match, command) {
                    None => {}
                    Some(previous) => {
                        return Err(format!(
                            "Duplicate command {0:?} (with {1} and {2} arguments, resp.)",
                            command.command_match, previous.parameters, command.parameters,
                        ));
                    }
                }
                if index > 0 {
                    regex_buf.push_str("|");
                }
                regex_buf.push_str("\\b");
                regex_buf.push_str(&regex::escape(command.command_match));
                if command.parameters == 0 {
                    regex_buf.push_str("\\b");
                }
            }
            regex_buf.push_str(")");
        }
        Ok(Parser {
            command_map: command_map,
            regex: regex::Regex::new(&regex_buf)
                .map_err(|error| format!("Compiling regular expression: {}", error))?,
            escape_or_comma: _map_re_error(regex::Regex::new("\\\\.| *, *"))?,
            escape_or_closing: _map_re_error(regex::Regex::new("\\\\.|\\)"))?,
            fqcn_re: _map_re_error(regex::Regex::new(
                "^[a-z0-9_]+\\.[a-z0-9_]+(?:\\.[a-z0-9_]+)+$",
            ))?,
            plugin_type_re: _map_re_error(regex::Regex::new("^[a-z_]+$"))?,
            array_stub_re: _map_re_error(regex::Regex::new("\\[([^\\]]*)\\]"))?,
            fqcn_type_prefix_re: _map_re_error(regex::Regex::new(
                "^([^.]+\\.[^.]+\\.[^#]+)#([^:]+):(.*)$",
            ))?,
        })
    }

    fn is_fqcn(&self, fqcn: &str) -> bool {
        self.fqcn_re.is_match(fqcn)
    }

    fn is_plugin_type(&self, plugin_type: &str) -> bool {
        self.plugin_type_re.is_match(plugin_type)
    }
}

static CLASSIC_MARKUP_PARSER: LazyLock<Parser<'static>> = LazyLock::new(|| {
    let commands: Vec<&Command<'static>> = ALL_COMMANDS.iter().filter(|c| c.old_markup).collect();
    Parser::new(commands.as_slice()).unwrap()
});
static FULL_PARSER: LazyLock<Parser<'static>> = LazyLock::new(|| {
    let commands: Vec<&Command<'static>> = ALL_COMMANDS.iter().collect();
    Parser::new(commands.as_slice()).unwrap()
});

enum Token<'a> {
    End,
    Text {
        text: &'a str,
        start: usize,
        end: usize,
    },
    UnescapedCommand {
        command: &'a Command<'a>,
        parameters: Vec<&'a str>,
        start: usize,
        end: usize,
    },
    EscapedCommand {
        command: &'a Command<'a>,
        parameters: Vec<String>,
        start: usize,
        end: usize,
    },
    Error {
        message: String,
        start: usize,
        end: usize,
    },
}

fn get_source<'a>(input: &'a str, token: &'_ Token<'a>) -> Option<&'a str> {
    match token {
        Token::End => Option::None,
        Token::Text {
            text: _,
            start,
            end,
        } => Option::Some(&input[*start..*end]),
        Token::EscapedCommand {
            command: _,
            parameters: _,
            start,
            end,
        } => Option::Some(&input[*start..*end]),
        Token::UnescapedCommand {
            command: _,
            parameters: _,
            start,
            end,
        } => Option::Some(&input[*start..*end]),
        Token::Error {
            message: _,
            start,
            end,
        } => Option::Some(&input[*start..*end]),
    }
}

struct StringParser<'a, 'b> {
    input: &'a str,
    length: usize,
    position: usize,
    tokens: VecDeque<Token<'a>>,
    parser: &'a Parser<'a>,
    strict: bool,
    helpful_errors: bool,
    r#where: &'b Option<String>,
}

// This should really be str::find_at...
fn find_at<'a>(slice: &'a str, pat: &'a str, at: usize) -> Option<usize> {
    slice[at..].find(pat).map(|i| at + i)
}

impl<'a, 'b> StringParser<'a, 'b> {
    fn new(
        input: &'a str,
        parser: &'a Parser<'a>,
        strict: bool,
        helpful_errors: bool,
        r#where: &'b Option<String>,
    ) -> StringParser<'a, 'b> {
        StringParser {
            input: input,
            length: input.len(),
            position: 0,
            tokens: VecDeque::new(),
            parser: parser,
            strict: strict,
            helpful_errors: helpful_errors,
            r#where: r#where,
        }
    }

    fn push_text(&mut self, until: usize) {
        self.tokens.push_back(Token::Text {
            text: &self.input[self.position..until],
            start: self.position,
            end: until,
        });
        self.position = until;
    }

    fn strip(
        &self,
        left: usize,
        right: usize,
        strip_left: bool,
        strip_right: bool,
    ) -> (usize, usize) {
        let mut l = left;
        let mut r = right;
        while strip_left && l < r && self.input.as_bytes()[l] == 32 {
            l += 1;
        }
        while strip_right && l < r && self.input.as_bytes()[r] == 32 {
            r -= 1;
        }
        (l, r)
    }

    fn _process_match(
        &mut self,
        m: regex::Match,
        argument: &mut stringbuilder::CollectorAppender<'a>,
    ) -> Result<bool, String> {
        if m.start() > self.position {
            argument.push_str(&self.input[self.position..m.start()]);
        }
        self.position = m.end();
        if self.input.as_bytes()[m.start()] != b'\\' {
            return Ok(true);
        }
        let escaped = &self.input[m.start() + 1..self.position];
        if self.strict && escaped != ")" && escaped != "\\" {
            self.position = m.end();
            return Err(format!("Unnecessarily escaped {:?}", escaped));
        }
        argument.push_str(escaped);
        return Ok(false);
    }

    fn parse_escaped_call(&mut self, count: u32) -> Result<Vec<String>, String> {
        let mut parameters = Vec::new();
        if count == 0 {
            return Ok(parameters);
        }
        let mut commas_left = count - 1;
        while commas_left > 0 {
            let mut argument = stringbuilder::CollectorAppender::new();
            loop {
                let m = match self
                    .parser
                    .escape_or_comma
                    .find_at(self.input, self.position)
                {
                    Some(m) => m,
                    None => {
                        self.position = self.length;
                        return Err(format!(
                            "Cannot find comma separating parameter {} from the next one",
                            count - commas_left
                        ));
                    }
                };
                if self._process_match(m, &mut argument)? {
                    break;
                }
            }
            parameters.push(argument.into_string());
            commas_left -= 1;
        }
        let mut argument = stringbuilder::CollectorAppender::new();
        loop {
            let m = match self
                .parser
                .escape_or_closing
                .find_at(self.input, self.position)
            {
                Some(m) => m,
                None => {
                    self.position = self.length;
                    return Err("Cannot find closing \")\" after last parameter".to_string());
                }
            };
            if self._process_match(m, &mut argument)? {
                break;
            }
        }
        parameters.push(argument.into_string());
        Ok(parameters)
    }

    fn parse_unescaped_call(&mut self, count: u32) -> Result<Vec<&'a str>, String> {
        let mut parameters = Vec::new();
        if count == 0 {
            return Ok(parameters);
        }
        let mut commas_left = count - 1;
        let mut first = true;
        while commas_left > 0 {
            let index = match find_at(self.input, ",", self.position) {
                Some(index) => index,
                None => {
                    self.position = self.length;
                    return Err(format!(
                        "Cannot find comma separating parameter {} from the next one",
                        count - commas_left
                    ));
                }
            };
            let (start, end) = self.strip(self.position, index, !first, true);
            parameters.push(&self.input[start..end]);
            self.position = index + 1;
            commas_left -= 1;
            first = false;
        }
        let index = match find_at(self.input, ")", self.position) {
            Some(index) => index,
            None => {
                self.position = self.length;
                return Err("Cannot find closing \")\" after last parameter".to_string());
            }
        };
        let (start, end) = self.strip(self.position, index, !first, false);
        parameters.push(&self.input[start..end]);
        self.position = index + 1;
        Ok(parameters)
    }

    fn _compose_parsing_error(
        &self,
        command: &Command,
        start: usize,
        end: usize,
        error: String,
    ) -> String {
        let error_source = if self.helpful_errors {
            format!("\"{}\"", &self.input[start..end],)
        } else {
            format!(
                "{}{}",
                command.command,
                if command.parameters > 0 { "()" } else { "" },
            )
        };
        format!(
            "While parsing {} at index {}{}: {}",
            error_source,
            start + 1,
            match self.r#where {
                Some(w) => w,
                None => "",
            },
            error,
        )
    }

    fn prepare_tokens(&mut self) {
        let m = match self.parser.regex.find_at(self.input, self.position) {
            Some(m) => m,
            None => {
                self.push_text(self.length);
                return;
            }
        };
        let start = m.start();
        if start > self.position {
            self.push_text(start);
        }
        let command = match self.parser.command_map.get(m.as_str()) {
            Some(command) => command,
            None => {
                self.tokens.push_back(Token::Error {
                    message: format!(
                        "Internal error: cannot find command {:?} at {}",
                        m.as_str(),
                        self.position,
                    ),
                    start: m.start(),
                    end: m.end(),
                });
                return;
            }
        };
        self.position = m.end();
        if command.escaped_arguments {
            match self.parse_escaped_call(command.parameters) {
                Ok(parameters) => {
                    self.tokens.push_back(Token::EscapedCommand {
                        command: command,
                        parameters: parameters,
                        start: m.start(),
                        end: self.position,
                    });
                }
                Err(error) => {
                    self.tokens.push_back(Token::Error {
                        message: self._compose_parsing_error(
                            command,
                            m.start(),
                            self.position,
                            error,
                        ),
                        start: m.start(),
                        end: self.position,
                    });
                }
            };
        } else {
            match self.parse_unescaped_call(command.parameters) {
                Ok(parameters) => {
                    self.tokens.push_back(Token::UnescapedCommand {
                        command: command,
                        parameters: parameters,
                        start: m.start(),
                        end: self.position,
                    });
                }
                Err(error) => {
                    self.tokens.push_back(Token::Error {
                        message: self._compose_parsing_error(
                            command,
                            m.start(),
                            self.position,
                            error,
                        ),
                        start: m.start(),
                        end: self.position,
                    });
                }
            };
        }
    }

    fn next(&mut self) -> Token<'a> {
        loop {
            match self.tokens.pop_front() {
                Some(token) => return token,
                None => {}
            }
            if self.position == self.length {
                return Token::End;
            }
            self.prepare_tokens();
        }
    }
}

/// The parsing context.
pub struct Context {
    /// The current plugin for which this documentation is parsed.
    pub current_plugin: Option<Rc<dom::PluginIdentifier>>,

    /// The current role entrypoint (if applicable) for which this
    /// documentation is parsed.
    pub role_entrypoint: Option<Rc<String>>,
}

fn _parse_option_like<'a>(
    input: String,
    context: &'a Context,
    parser: &'a Parser<'a>,
) -> Result<
    (
        Option<Rc<dom::PluginIdentifier>>,
        Option<Rc<String>>,
        Box<[String]>,
        String,
        Option<String>,
    ),
    String,
> {
    let mut text = input.as_str();
    let mut value: Option<String> = Option::None;
    match text.split_once("=") {
        Some((r, ov)) => {
            text = r;
            value = Option::Some(ov.to_string());
        }
        None => {}
    }
    let mut plugin: Option<Rc<dom::PluginIdentifier>> = Option::None;
    let mut entrypoint: Option<Rc<String>> = Option::None;
    match parser.fqcn_type_prefix_re.captures(text) {
        Some(capture) => {
            let fqcn = &capture[1];
            if !parser.is_fqcn(fqcn) {
                return Err(format!("Plugin name {:?} is not a FQCN", fqcn));
            }
            let plugin_type = &capture[2];
            if !parser.is_plugin_type(plugin_type) {
                return Err(format!("Plugin type {:?} is not valid", plugin_type));
            }
            text = &text[capture.get(3).unwrap().start()..];
            plugin = Some(Rc::new(dom::PluginIdentifier {
                fqcn: fqcn.to_string(),
                r#type: plugin_type.to_string(),
            }))
        }
        None => {
            if text.starts_with(IGNORE_MARKER) {
                text = &text[IGNORE_MARKER.len()..]
            } else {
                plugin = context.current_plugin.clone();
                entrypoint = context.role_entrypoint.clone();
            }
        }
    }
    if let Some(ref pi) = plugin {
        if pi.r#type == "role" {
            match text.split_once(":") {
                Some((a, b)) => {
                    entrypoint = Some(Rc::new(a.to_string()));
                    text = b;
                }
                None => {}
            }
            if entrypoint == Option::None {
                return Err("Role reference is missing entrypoint".to_string());
            }
        }
    }
    if text.contains(":") || text.contains("#") {
        return Err(format!("Invalid option/return value name {:?}", text));
    }
    let link: Vec<String> = parser
        .array_stub_re
        .replace_all(text, "")
        .to_string()
        .split(".")
        .map(|s| s.to_string())
        .collect();
    Ok((
        plugin,
        entrypoint,
        link.into_boxed_slice(),
        text.to_string(),
        value,
    ))
}

struct ToPartError<'a> {
    command: &'a Command<'a>,
    start: usize,
    end: usize,
    message: String,
}

impl<'a> ToPartError<'a> {
    fn to_part<'b>(self, parser: &StringParser<'a, 'b>) -> Option<dom::Part<'a>> {
        Some(dom::Part::Error {
            message: parser._compose_parsing_error(
                self.command,
                self.start,
                self.end,
                self.message,
            ),
        })
    }
}

fn to_part<'a>(
    token: Token<'a>,
    context: &'a Context,
    parser: &'a Parser<'a>,
) -> Result<Option<dom::Part<'a>>, ToPartError<'a>> {
    match token {
        Token::End => panic!("Cannot get part from end token"),
        Token::Text {
            text,
            start: _,
            end: _,
        } => Ok(Some(dom::Part::Text { text: text })),
        Token::UnescapedCommand {
            command,
            parameters,
            start,
            end,
        } => match match command.command {
            "B" => Ok(dom::Part::Bold {
                text: parameters[0],
            }),
            "I" => Ok(dom::Part::Italic {
                text: parameters[0],
            }),
            "M" => {
                if !parser.is_fqcn(parameters[0]) {
                    Err(format!("Module name {:?} is not a FQCN", parameters[0]))
                } else {
                    Ok(dom::Part::Module {
                        fqcn: parameters[0],
                    })
                }
            }
            "U" => Ok(dom::Part::URL { url: parameters[0] }),
            "L" => Ok(dom::Part::Link {
                text: parameters[0],
                url: parameters[1],
            }),
            "R" => Ok(dom::Part::RSTRef {
                text: parameters[0],
                r#ref: parameters[1],
            }),
            "C" => Ok(dom::Part::Code {
                text: parameters[0],
            }),
            "HORIZONTALLINE" => Ok(dom::Part::HorizontalLine),
            _ => Err(format!(
                "Handling unescaped {:?} not yet implemented!",
                command.command
            )),
        } {
            Ok(part) => Ok(Some(part)),
            Err(msg) => Err(ToPartError {
                command: command,
                start: start,
                end: end,
                message: msg,
            }),
        },
        Token::EscapedCommand {
            command,
            mut parameters,
            start,
            end,
        } => match match command.command {
            "P" => {
                let value = parameters.pop().unwrap();
                match value.split_once("#") {
                    Some((fqcn, ptype)) => {
                        if !parser.is_fqcn(fqcn) {
                            Err(format!("Plugin name {:?} is not a FQCN", fqcn))
                        } else if !parser.is_plugin_type(ptype) {
                            Err(format!("Plugin name {:?} is not a FQCN", ptype))
                        } else {
                            Ok(dom::Part::Plugin {
                                plugin: dom::PluginIdentifier {
                                    fqcn: fqcn.to_string(),
                                    r#type: ptype.to_string(),
                                },
                            })
                        }
                    }
                    None => Err(format!(
                        "Parameter {:?} is not of the form FQCN#type",
                        value
                    )),
                }
            }
            "E" => Ok(dom::Part::EnvVariable {
                name: parameters.pop().unwrap(),
            }),
            "V" => Ok(dom::Part::OptionValue {
                value: parameters.pop().unwrap(),
            }),
            "O" => _parse_option_like(parameters.pop().unwrap(), context, parser).map(
                |(plugin, entrypoint, link, name, value)| dom::Part::OptionName {
                    plugin: plugin,
                    entrypoint: entrypoint,
                    link: link,
                    name: name,
                    value: value,
                },
            ),
            "RV" => _parse_option_like(parameters.pop().unwrap(), context, parser).map(
                |(plugin, entrypoint, link, name, value)| dom::Part::ReturnValue {
                    plugin: plugin,
                    entrypoint: entrypoint,
                    link: link,
                    name: name,
                    value: value,
                },
            ),
            _ => Err(format!(
                "Handling escaped {:?} not yet implemented!",
                command.command
            )),
        } {
            Ok(part) => Ok(Some(part)),
            Err(msg) => Err(ToPartError {
                command: command,
                start: start,
                end: end,
                message: msg,
            }),
        },
        Token::Error {
            message,
            start: _,
            end: _,
        } => Ok(Some(dom::Part::Error { message: message })),
    }
}

fn do_parse_with_source<'a, 'b>(
    parser: &mut StringParser<'a, 'b>,
    context: &'a Context,
) -> Vec<dom::PartWithSource<'a>> {
    let mut result = Vec::new();
    loop {
        let token = parser.next();
        if matches!(token, Token::End) {
            break;
        }
        let source = get_source(parser.input, &token);
        match to_part(token, context, parser.parser).unwrap_or_else(|err| err.to_part(parser)) {
            Some(part) => result.push(dom::PartWithSource {
                part: part,
                source: source.unwrap(),
            }),
            None => {}
        }
    }
    result
}

fn do_parse_without_source<'a, 'b>(
    parser: &mut StringParser<'a, 'b>,
    context: &'a Context,
) -> Vec<dom::Part<'a>> {
    let mut result = Vec::new();
    loop {
        let token = parser.next();
        if matches!(token, Token::End) {
            break;
        }
        match to_part(token, context, parser.parser).unwrap_or_else(|err| err.to_part(parser)) {
            Some(part) => result.push(part),
            None => {}
        }
    }
    result
}

/// Parsing options.
pub struct ParseOptions {
    /// Whether to allow all markup, or only classic markup (before introduction of semantic markup).
    only_classic_markup: bool,

    /// Whether to do strict parsing.
    ///
    /// Affects whether quoting is allowed for characters that do not need to be quoted, for example.
    strict: bool,

    /// Whether to include more information (like the whole broken markup) in error messages.
    helpful_errors: bool,

    /// More location information to include in error messages.
    r#where: Option<String>,
}

impl ParseOptions {
    /// Create default parsing information.
    pub fn default() -> ParseOptions {
        ParseOptions {
            only_classic_markup: false,
            strict: false,
            helpful_errors: true,
            r#where: Option::None,
        }
    }

    /// Modify parsing information to restrict to classic markup.
    pub fn only_classic_markup(self) -> ParseOptions {
        ParseOptions {
            only_classic_markup: true,
            strict: self.strict,
            helpful_errors: self.helpful_errors,
            r#where: self.r#where,
        }
    }

    /// Modify parsing information to enable strict parsing.
    pub fn strict(self) -> ParseOptions {
        ParseOptions {
            only_classic_markup: self.only_classic_markup,
            strict: true,
            helpful_errors: self.helpful_errors,
            r#where: self.r#where,
        }
    }

    /// Modify parsing information to disable helpful error messages.
    pub fn unhelpful_errors(self) -> ParseOptions {
        ParseOptions {
            only_classic_markup: self.only_classic_markup,
            strict: self.strict,
            helpful_errors: false,
            r#where: self.r#where,
        }
    }

    /// Modify parsing information to add location information to error messages.
    pub fn r#where(self, r#where: String) -> ParseOptions {
        ParseOptions {
            only_classic_markup: self.only_classic_markup,
            strict: self.strict,
            helpful_errors: self.helpful_errors,
            r#where: Option::Some(r#where),
        }
    }

    /// Modify parsing information to add paragraph index to error messages.
    fn add_paragraph_to_where(&self, index: usize) -> ParseOptions {
        let prefix = format!(" of paragraph {}", index);
        ParseOptions {
            only_classic_markup: self.only_classic_markup,
            strict: self.strict,
            helpful_errors: self.helpful_errors,
            r#where: match self.r#where.as_ref() {
                Some(w) => Some(prefix + &w),
                None => Some(prefix),
            },
        }
    }
}

fn create_parser<'a, 'b>(input: &'a str, opts: &'b ParseOptions) -> StringParser<'a, 'b> {
    StringParser::new(
        input,
        if opts.only_classic_markup {
            &*CLASSIC_MARKUP_PARSER
        } else {
            &*FULL_PARSER
        },
        opts.strict,
        opts.helpful_errors,
        &opts.r#where,
    )
}

/// Parse a paragraph and emit a list of parts.
pub fn parse<'a>(
    input: &'a str,
    context: &'a Context,
    opts: &'_ ParseOptions,
) -> Vec<dom::PartWithSource<'a>> {
    let mut string_parser = create_parser(input, opts);
    do_parse_with_source(&mut string_parser, context)
}

/// Parse a paragraph and emit a list of parts with source information.
pub fn parse_without_sources<'a>(
    input: &'a str,
    context: &'a Context,
    opts: &'_ ParseOptions,
) -> Vec<dom::Part<'a>> {
    let mut string_parser = create_parser(input, opts);
    do_parse_without_source(&mut string_parser, context)
}

/// Parse a paragraph and emit a list of parts.
pub fn parse_paragraphs<'a, I>(
    input: I,
    context: &'a Context,
    opts: &'_ ParseOptions,
) -> Vec<Vec<dom::PartWithSource<'a>>>
where
    I: Iterator<Item = &'a str>,
{
    input
        .enumerate()
        .map(|(index, p)| parse(p, context, &opts.add_paragraph_to_where(index + 1)))
        .collect()
}

/// Parse a paragraph and emit a list of parts with source information.
pub fn parse_paragraphs_without_sources<'a, I>(
    input: I,
    context: &'a Context,
    opts: &'_ ParseOptions,
) -> Vec<Vec<dom::Part<'a>>>
where
    I: Iterator<Item = &'a str>,
{
    input
        .enumerate()
        .map(|(index, p)| {
            parse_without_sources(p, context, &opts.add_paragraph_to_where(index + 1))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markup::dom;

    #[test]
    fn parse_simple() {
        let context = Context {
            current_plugin: None,
            role_entrypoint: None,
        };
        assert_eq!(parse("", &context, &ParseOptions::default()), vec!());
        assert_eq!(
            parse("Foo", &context, &ParseOptions::default()),
            vec!(dom::PartWithSource {
                part: dom::Part::Text { text: "Foo" },
                source: "Foo"
            })
        );
    }
}
