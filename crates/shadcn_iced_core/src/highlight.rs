//! Syntax highlighting for code blocks.
//!
//! A small lexer for the languages that documentation shows the most (Rust,
//! TOML, shell, JavaScript / TypeScript and JSON), with no dependency: it
//! keeps a copied code block small and adds nothing to the size of a web
//! build, unlike a full grammar engine. It finds keywords, strings,
//! numbers, comments, functions, types and so on, and [`color`] gives them
//! the colors of the GitHub themes that the shadcn/ui documentation
//! highlights its code with (`github-light` and `github-dark-default`),
//! while plain text keeps the foreground of the theme.
//!
//! ```
//! use shadcn_iced_core::highlight::{Language, Token, highlight};
//!
//! let lines = highlight(Language::Rust, "let x = 42;");
//! let tokens: Vec<_> = lines[0].iter().map(|(range, token)| (&"let x = 42;"[range.clone()], *token)).collect();
//!
//! assert_eq!(tokens[0], ("let", Token::Keyword));
//! assert!(tokens.contains(&("42", Token::Number)));
//! ```
use crate::theme::tokens;

use iced::{Color, Theme};

use std::ops::Range;

/// The kind of a piece of code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    /// Anything else, in the color of the text.
    Plain,
    /// `fn`, `let`, `if`, `const`, `export`, ...
    Keyword,
    /// A string or a character literal.
    String,
    /// A number.
    Number,
    /// `true`, `false`, `null`, `None`, `self`, ...
    Constant,
    /// A comment.
    Comment,
    /// A function or a macro being called or defined, or a command.
    Function,
    /// A type, or a section of a TOML file.
    Type,
    /// A shell variable (`$HOME`).
    Variable,
    /// A key of a JSON object or of a TOML table.
    Property,
}

/// A language that [`Highlighter`] knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Rust.
    Rust,
    /// TOML, e.g. `Cargo.toml`.
    Toml,
    /// A shell (`sh`, `bash`, `zsh`), e.g. terminal commands.
    Shell,
    /// JavaScript or TypeScript, with JSX.
    JavaScript,
    /// JSON (with comments).
    Json,
}

impl Language {
    /// Finds a language by one of its usual names (those of Shiki and
    /// Markdown code fences), e.g. `rust`, `rs`, `toml`, `bash`, `sh`,
    /// `shell`, `ts`, `tsx`, `js` or `json`.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name.to_ascii_lowercase().as_str() {
            "rust" | "rs" => Language::Rust,
            "toml" => Language::Toml,
            "sh" | "bash" | "zsh" | "shell" | "shellscript" | "console" | "terminal" => {
                Language::Shell
            }
            "js" | "jsx" | "mjs" | "cjs" | "javascript" | "ts" | "tsx" | "mts" | "typescript" => {
                Language::JavaScript
            }
            "json" | "jsonc" | "json5" => Language::Json,
            _ => return None,
        })
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                "move", "mut", "pub", "ref", "return", "static", "struct", "super", "trait",
                "type", "unsafe", "use", "where", "while", "yield",
            ],
            Language::JavaScript => &[
                "abstract",
                "as",
                "async",
                "await",
                "break",
                "case",
                "catch",
                "class",
                "const",
                "continue",
                "debugger",
                "declare",
                "default",
                "delete",
                "do",
                "else",
                "enum",
                "export",
                "extends",
                "finally",
                "for",
                "from",
                "function",
                "get",
                "if",
                "implements",
                "import",
                "in",
                "instanceof",
                "interface",
                "keyof",
                "let",
                "new",
                "of",
                "private",
                "protected",
                "public",
                "readonly",
                "return",
                "satisfies",
                "set",
                "static",
                "switch",
                "throw",
                "try",
                "type",
                "typeof",
                "var",
                "void",
                "while",
                "with",
                "yield",
            ],
            Language::Shell => &[
                "case", "do", "done", "elif", "else", "esac", "export", "fi", "for", "function",
                "if", "in", "local", "return", "select", "then", "until", "while",
            ],
            Language::Toml | Language::Json => &[],
        }
    }

    fn constants(self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["true", "false", "self", "Self", "None", "Some", "Ok", "Err"],
            Language::JavaScript => &[
                "true",
                "false",
                "null",
                "undefined",
                "this",
                "super",
                "NaN",
                "Infinity",
            ],
            Language::Toml | Language::Json => &["true", "false", "null", "inf", "nan"],
            Language::Shell => &[],
        }
    }

    fn types(self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16",
                "i32", "i64", "i128", "isize", "f32", "f64",
            ],
            Language::JavaScript => &[
                "any", "bigint", "boolean", "never", "number", "object", "string", "symbol",
                "unknown",
            ],
            _ => &[],
        }
    }

    fn line_comment(self) -> &'static [&'static str] {
        match self {
            Language::Rust | Language::JavaScript | Language::Json => &["//"],
            Language::Toml | Language::Shell => &["#"],
        }
    }

    fn block_comments(self) -> bool {
        matches!(self, Language::Rust | Language::JavaScript | Language::Json)
    }

    fn quotes(self) -> &'static [char] {
        match self {
            Language::Rust => &['"', '\''],
            Language::Json => &['"'],
            Language::JavaScript => &['"', '\'', '`'],
            Language::Toml | Language::Shell => &['"', '\''],
        }
    }

    /// Whether strings opened with `quote` may span several lines.
    fn multiline(self, quote: char, triple: bool) -> bool {
        match self {
            Language::Rust => quote == '"',
            Language::JavaScript => quote == '`',
            Language::Toml => triple,
            Language::Shell => true,
            Language::Json => false,
        }
    }

    /// Whether a backslash escapes the next character in strings opened
    /// with `quote`.
    fn escapes(self, quote: char) -> bool {
        !(matches!(self, Language::Shell | Language::Toml) && quote == '\'')
    }
}

/// What a [`Highlighter`] is in the middle of, at the end of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum State {
    #[default]
    Code,
    BlockComment,
    String {
        quote: char,
        triple: bool,
    },
}

/// Highlights code line by line, keeping track of comments and strings
/// that span several lines.
#[derive(Debug, Clone)]
pub struct Highlighter {
    language: Language,
    state: State,
    /// Whether a shell prompt (`$ `) was seen: the lines without one are
    /// the output of the commands.
    session: bool,
}

impl Highlighter {
    /// Creates a [`Highlighter`] at the start of some code.
    pub fn new(language: Language) -> Self {
        Self {
            language,
            state: State::Code,
            session: false,
        }
    }

    /// Splits the next line into contiguous ranges of bytes with their
    /// [`Token`]. The ranges cover the whole line.
    pub fn line(&mut self, line: &str) -> Vec<(Range<usize>, Token)> {
        if self.language == Language::Shell && self.state == State::Code {
            if line.trim_start().starts_with("$ ") {
                self.session = true;
            } else if self.session {
                let mut spans = Vec::new();
                fill(&mut spans, line.len());
                return spans;
            }
        }

        let mut lexer = Lexer {
            language: self.language,
            line,
            position: 0,
            spans: Vec::new(),
            command: true,
        };

        self.state = lexer.run(self.state);

        let mut spans = lexer.spans;
        fill(&mut spans, line.len());
        spans
    }
}

/// Highlights every line of `code` (split on `\n`).
pub fn highlight(language: Language, code: &str) -> Vec<Vec<(Range<usize>, Token)>> {
    let mut highlighter = Highlighter::new(language);

    code.split('\n')
        .map(|line| highlighter.line(line))
        .collect()
}

/// The color of a [`Token`]: the colors of `github-light` or
/// `github-dark-default`, depending on the theme, and the foreground of the
/// theme for plain text.
pub fn color(theme: &Theme, token: Token) -> Color {
    let t = tokens(theme);

    let (light, dark) = match token {
        Token::Plain => return t.palette.foreground,
        Token::Keyword => (0xd73a49, 0xff7b72),
        Token::String => (0x032f62, 0xa5d6ff),
        Token::Number | Token::Constant | Token::Property => (0x005cc5, 0x79c0ff),
        Token::Comment => (0x6a737d, 0x8b949e),
        Token::Function | Token::Type => (0x6f42c1, 0xd2a8ff),
        Token::Variable => (0xe36209, 0xffa657),
    };

    let rgb = if t.is_dark() { dark } else { light };

    Color::from_rgb8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// The last word of `code`, if it ends with one (and spaces).
fn previous_word(code: &str) -> Option<&str> {
    let code = code.trim_end();
    let start = code
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
        .map_or(0, |i| i + 1);

    (start < code.len()).then(|| &code[start..])
}

/// Fills the gaps between `spans` with [`Token::Plain`] and merges the
/// neighbors of the same kind.
fn fill(spans: &mut Vec<(Range<usize>, Token)>, len: usize) {
    let mut filled: Vec<(Range<usize>, Token)> = Vec::with_capacity(spans.len() * 2 + 1);
    let mut push = |range: Range<usize>, token: Token| {
        if range.is_empty() {
            return;
        }

        match filled.last_mut() {
            Some((last, kind)) if *kind == token && last.end == range.start => {
                last.end = range.end;
            }
            _ => filled.push((range, token)),
        }
    };

    let mut position = 0;

    for (range, token) in spans.drain(..) {
        push(position..range.start, Token::Plain);
        position = range.end;
        push(range, token);
    }

    push(position..len, Token::Plain);

    *spans = filled;
}

struct Lexer<'a> {
    language: Language,
    line: &'a str,
    position: usize,
    spans: Vec<(Range<usize>, Token)>,
    /// Whether the next shell word is a command.
    command: bool,
}

impl Lexer<'_> {
    fn rest(&self) -> &str {
        &self.line[self.position..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn push(&mut self, start: usize, token: Token) {
        if self.position > start {
            self.spans.push((start..self.position, token));
        }
    }

    fn run(&mut self, state: State) -> State {
        match state {
            State::Code => {}
            State::BlockComment => {
                if !self.block_comment(0) {
                    return State::BlockComment;
                }
            }
            State::String { quote, triple } => {
                if !self.string_body(0, quote, triple) {
                    return state;
                }
            }
        }

        while let Some(c) = self.peek() {
            let start = self.position;

            if c.is_whitespace() {
                self.position += c.len_utf8();
                continue;
            }

            if self
                .language
                .line_comment()
                .iter()
                .any(|comment| self.rest().starts_with(comment))
                && self.comment_allowed()
            {
                self.position = self.line.len();
                self.push(start, Token::Comment);
                break;
            }

            if self.language.block_comments() && self.rest().starts_with("/*") {
                self.position += 2;
                if !self.block_comment(start) {
                    return State::BlockComment;
                }
                continue;
            }

            if self.language.quotes().contains(&c) && !self.is_lifetime(c) {
                let triple = self.language == Language::Toml
                    && self.rest().starts_with(&c.to_string().repeat(3));
                self.position += if triple { 3 } else { 1 };

                if !self.string_body(start, c, triple) {
                    return State::String { quote: c, triple };
                }

                if self.language == Language::Json
                    && self.followed_by(':')
                    && let Some(last) = self.spans.last_mut()
                {
                    last.1 = Token::Property;
                }
                self.command = false;
                continue;
            }

            match self.language {
                Language::Shell => self.shell(c, start),
                Language::Toml => self.toml(c, start),
                _ => self.code(c, start),
            }
        }

        State::Code
    }

    /// Whether a comment may start here: in a shell, `#` only starts a
    /// comment at the start of a word.
    fn comment_allowed(&self) -> bool {
        match self.language {
            Language::Shell => self.line[..self.position]
                .chars()
                .last()
                .is_none_or(char::is_whitespace),
            _ => true,
        }
    }

    /// Whether a `'` starts a Rust lifetime (`'a`) rather than a character.
    fn is_lifetime(&self, c: char) -> bool {
        if self.language != Language::Rust || c != '\'' {
            return false;
        }

        let mut chars = self.rest().chars().skip(1);

        !matches!(
            (chars.next(), chars.next()),
            (Some('\\'), _) | (Some(_), Some('\''))
        )
    }

    /// Consumes a block comment up to its end, returning whether it ends on
    /// this line.
    fn block_comment(&mut self, start: usize) -> bool {
        match self.rest().find("*/") {
            Some(end) => {
                self.position += end + 2;
                self.push(start, Token::Comment);
                true
            }
            None => {
                self.position = self.line.len();
                self.push(start, Token::Comment);
                false
            }
        }
    }

    /// Consumes a string up to its closing quote, returning whether it
    /// closes on this line.
    fn string_body(&mut self, start: usize, quote: char, triple: bool) -> bool {
        let escapes = self.language.escapes(quote);
        let closing = if triple {
            quote.to_string().repeat(3)
        } else {
            quote.to_string()
        };

        let mut chars = self.rest().char_indices();

        while let Some((offset, c)) = chars.next() {
            if escapes && c == '\\' {
                let _ = chars.next();
                continue;
            }

            if self.rest()[offset..].starts_with(&closing) {
                self.position += offset + closing.len();
                self.push(start, Token::String);
                return true;
            }
        }

        self.position = self.line.len();
        self.push(start, Token::String);

        // A string that cannot span lines ends with the line.
        !self.language.multiline(quote, triple)
    }

    /// Whether the next character that is not a space is `c`.
    fn followed_by(&self, c: char) -> bool {
        self.rest().trim_start().starts_with(c)
    }

    fn word(&mut self, extra: &[char]) -> &str {
        let start = self.position;

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || extra.contains(&c) {
                self.position += c.len_utf8();
            } else {
                break;
            }
        }

        &self.line[start..self.position]
    }

    fn number(&mut self, start: usize) {
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '.' && self.after_dot_is_digit() {
                self.position += c.len_utf8();
            } else {
                break;
            }
        }

        self.push(start, Token::Number);
    }

    fn after_dot_is_digit(&self) -> bool {
        self.rest()[1..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    }

    /// Rust, JavaScript and JSON.
    fn code(&mut self, c: char, start: usize) {
        if c.is_ascii_digit() {
            self.number(start);
            return;
        }

        if c.is_alphabetic() || c == '_' || c == '$' && self.language == Language::JavaScript {
            let extra: &[char] = if self.language == Language::JavaScript {
                &['$']
            } else {
                &[]
            };
            let word = self.word(extra).to_owned();
            let language = self.language;

            let token = if language.keywords().contains(&word.as_str()) {
                Token::Keyword
            } else if language.constants().contains(&word.as_str()) {
                Token::Constant
            } else if language.types().contains(&word.as_str()) {
                Token::Type
            } else if language == Language::Rust && self.rest().starts_with('!') {
                // A macro.
                self.position += 1;
                Token::Function
            } else if self.rest().starts_with('(')
                || language == Language::Rust && self.rest().starts_with("::<")
                || previous_word(&self.line[..start]) == Some("fn")
            {
                Token::Function
            } else if word.starts_with(|c: char| c.is_uppercase()) {
                Token::Type
            } else if language == Language::JavaScript && self.followed_by(':') && self.in_object()
            {
                Token::Property
            } else {
                Token::Plain
            };

            self.push(start, token);
            return;
        }

        self.position += c.len_utf8();
    }

    /// Whether the code before the current word looks like the inside of an
    /// object literal or a type (`{ key: ...` or `, key: ...`).
    fn in_object(&self) -> bool {
        let before = self.line[..self.position]
            .trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '$')
            .trim_end();

        before.is_empty() || before.ends_with('{') || before.ends_with(',')
    }

    fn toml(&mut self, c: char, start: usize) {
        let first = self.line[..start].trim().is_empty();

        if c == '[' && first {
            // A table header, `[package]` or `[[bin]]`.
            self.position = self
                .rest()
                .rfind(']')
                .map_or(self.line.len(), |end| self.position + end + 1);
            self.push(start, Token::Type);
            return;
        }

        if c.is_ascii_digit() || (c == '+' || c == '-') && self.after_dot_is_digit() {
            self.position += c.len_utf8();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || matches!(c, '_' | '.' | '-' | ':' | '+') {
                    self.position += c.len_utf8();
                } else {
                    break;
                }
            }
            self.push(start, Token::Number);
            return;
        }

        if c.is_alphabetic() || c == '_' {
            let word = self.word(&['-', '.']).to_owned();

            let token = if self.followed_by('=') && !self.after_equals(start) {
                Token::Property
            } else if Language::Toml.constants().contains(&word.as_str()) {
                Token::Constant
            } else {
                Token::Plain
            };

            self.push(start, token);
            return;
        }

        self.position += c.len_utf8();
    }

    /// Whether an `=` comes before `position` on the line (outside of
    /// strings, roughly), i.e. whether a word is part of a value.
    fn after_equals(&self, position: usize) -> bool {
        let before = &self.line[..position];
        let in_braces = before.rfind('{').is_some_and(|open| {
            before[open..].rfind([',', '{']).is_some_and(|last| {
                before[open + last..]
                    .trim_start_matches([',', '{'])
                    .trim()
                    .is_empty()
            })
        });

        before.contains('=') && !in_braces
    }

    fn shell(&mut self, c: char, start: usize) {
        // The prompt of a terminal session, `$ pnpm build`.
        if c == '$'
            && self.line[..start].trim().is_empty()
            && self.rest()[1..].starts_with(char::is_whitespace)
        {
            self.position += 1;
            self.command = true;
            return;
        }

        if c == '$' {
            self.position += 1;

            if self.rest().starts_with('{') {
                self.position = self
                    .rest()
                    .find('}')
                    .map_or(self.line.len(), |end| self.position + end + 1);
            } else if self.rest().starts_with('(') {
                // A command substitution: what follows is a command.
                self.position += 1;
                self.command = true;
                self.push(start, Token::Plain);
                return;
            } else {
                let _ = self.word(&[]);
            }

            self.push(start, Token::Variable);
            self.command = false;
            return;
        }

        if matches!(c, '|' | '&' | ';' | '(' | '`') {
            self.position += c.len_utf8();
            self.command = true;
            return;
        }

        if matches!(c, ')' | '<' | '>' | '\\' | '{' | '}') {
            self.position += c.len_utf8();
            return;
        }

        // A word: everything up to a space or an operator.
        while let Some(c) = self.peek() {
            if c.is_whitespace()
                || matches!(
                    c,
                    '|' | '&' | ';' | '(' | ')' | '<' | '>' | '"' | '\'' | '`' | '$'
                )
            {
                break;
            }
            self.position += c.len_utf8();
        }

        let word = &self.line[start..self.position];

        let token = if !self.command {
            Token::Plain
        } else if Language::Shell.keywords().contains(&word) {
            // `then`, `do`... are followed by a command.
            self.command = !matches!(
                word,
                "export" | "local" | "for" | "case" | "in" | "function"
            );
            self.push(start, Token::Keyword);
            return;
        } else if let Some(equals) = word.find('=') {
            // An assignment, `NAME=value`, before an optional command.
            self.spans.push((start..start + equals, Token::Variable));
            return;
        } else if word == "sudo" || word == "env" {
            self.push(start, Token::Function);
            return;
        } else {
            self.command = false;
            Token::Function
        };

        self.push(start, token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(language: Language, code: &str) -> Vec<Vec<(String, Token)>> {
        highlight(language, code)
            .into_iter()
            .zip(code.split('\n'))
            .map(|(spans, line)| {
                spans
                    .into_iter()
                    .filter(|(_, token)| *token != Token::Plain)
                    .map(|(range, token)| (line[range].to_owned(), token))
                    .collect()
            })
            .collect()
    }

    fn of(pairs: &[(&str, Token)]) -> Vec<(String, Token)> {
        pairs.iter().map(|(s, t)| ((*s).to_owned(), *t)).collect()
    }

    #[test]
    fn spans_cover_every_line() {
        let code = "fn main() {\n    println!(\"héllo\"); // done\n}";

        for (spans, line) in highlight(Language::Rust, code).iter().zip(code.split('\n')) {
            let mut end = 0;
            for (range, _) in spans {
                assert_eq!(range.start, end);
                end = range.end;
            }
            assert_eq!(end, line.len());
        }
    }

    #[test]
    fn rust() {
        use Token::*;

        let lines = tokens(
            Language::Rust,
            "use iced::Element;\n/* a\n b */ fn view<'a>(x: u32) -> Option<&'a str> {\n    let c = 'x';\n    println!(\"{x}\") // print\n}",
        );

        assert_eq!(lines[0], of(&[("use", Keyword), ("Element", Type)]));
        assert_eq!(lines[1], of(&[("/* a", Comment)]));
        assert_eq!(
            lines[2],
            of(&[
                (" b */", Comment),
                ("fn", Keyword),
                ("view", Function),
                ("u32", Type),
                ("Option", Type),
                ("str", Type),
            ])
        );
        assert_eq!(lines[3], of(&[("let", Keyword), ("'x'", String)]));
        assert_eq!(
            lines[4],
            of(&[
                ("println!", Function),
                ("\"{x}\"", String),
                ("// print", Comment)
            ])
        );
    }

    #[test]
    fn toml() {
        use Token::*;

        let lines = tokens(
            Language::Toml,
            "[dependencies]\niced = { version = \"0.14\", features = [\"svg\"] }\nedition = 2024 # comment\nflag = true",
        );

        assert_eq!(lines[0], of(&[("[dependencies]", Type)]));
        assert_eq!(
            lines[1],
            of(&[
                ("iced", Property),
                ("version", Property),
                ("\"0.14\"", String),
                ("features", Property),
                ("\"svg\"", String),
            ])
        );
        assert_eq!(
            lines[2],
            of(&[
                ("edition", Property),
                ("2024", Number),
                ("# comment", Comment)
            ])
        );
        assert_eq!(lines[3], of(&[("flag", Property), ("true", Constant)]));
    }

    #[test]
    fn shell() {
        use Token::*;

        let lines = tokens(
            Language::Shell,
            "pnpm add ai # install\nFOO=1 cargo run -p gallery | grep \"$HOME\"\nif true; then echo $PATH; fi",
        );

        assert_eq!(lines[0], of(&[("pnpm", Function), ("# install", Comment)]));
        assert_eq!(
            tokens(
                Language::Shell,
                "$ pnpm build
> next build
Done in 2s"
            ),
            vec![of(&[("pnpm", Function)]), vec![], vec![]]
        );
        assert_eq!(
            lines[1],
            of(&[
                ("FOO", Variable),
                ("cargo", Function),
                ("grep", Function),
                ("\"$HOME\"", String),
            ])
        );
        assert_eq!(
            lines[2],
            of(&[
                ("if", Keyword),
                ("true", Function),
                ("then", Keyword),
                ("echo", Function),
                ("$PATH", Variable),
                ("fi", Keyword),
            ])
        );
    }

    #[test]
    fn javascript() {
        use Token::*;

        let lines = tokens(
            Language::JavaScript,
            "import { Button } from \"@/ui\"\nconst config = { name: `a\nb`, size: 2 }\nexport function App(): string {}",
        );

        assert_eq!(
            lines[0],
            of(&[
                ("import", Keyword),
                ("Button", Type),
                ("from", Keyword),
                ("\"@/ui\"", String)
            ])
        );
        assert_eq!(
            lines[1],
            of(&[("const", Keyword), ("name", Property), ("`a", String)])
        );
        assert_eq!(
            lines[2],
            of(&[("b`", String), ("size", Property), ("2", Number)])
        );
        assert_eq!(
            lines[3],
            of(&[
                ("export", Keyword),
                ("function", Keyword),
                ("App", Function),
                ("string", Type)
            ])
        );
    }

    #[test]
    fn json() {
        use Token::*;

        let lines = tokens(
            Language::Json,
            "{ \"name\": \"ui\", \"private\": true, \"n\": 1.5 }",
        );

        assert_eq!(
            lines[0],
            of(&[
                ("\"name\"", Property),
                ("\"ui\"", String),
                ("\"private\"", Property),
                ("true", Constant),
                ("\"n\"", Property),
                ("1.5", Number),
            ])
        );
    }

    #[test]
    fn languages_by_name() {
        assert_eq!(Language::from_name("RS"), Some(Language::Rust));
        assert_eq!(Language::from_name("tsx"), Some(Language::JavaScript));
        assert_eq!(Language::from_name("bash"), Some(Language::Shell));
        assert_eq!(Language::from_name("cobol"), None);
    }
}
