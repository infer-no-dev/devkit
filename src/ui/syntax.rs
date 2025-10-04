//! Syntax highlighting support for code display

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use std::collections::HashMap;

/// Syntax highlighter for various programming languages
#[derive(Debug, Clone)]
pub struct SyntaxHighlighter {
    language_patterns: HashMap<String, LanguageRules>,
}

/// Language-specific highlighting rules
#[derive(Debug, Clone)]
struct LanguageRules {
    keywords: Vec<String>,
    types: Vec<String>,
    strings: StringRules,
    comments: CommentRules,
    numbers: bool,
    operators: Vec<String>,
}

/// String highlighting rules
#[derive(Debug, Clone)]
struct StringRules {
    single_quote: bool,
    double_quote: bool,
    triple_quote: bool,
}

/// Comment highlighting rules
#[derive(Debug, Clone)]
struct CommentRules {
    line_comment: Option<String>,
    block_comment: Option<(String, String)>,
}

/// Syntax highlighting theme
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub keyword: Style,
    pub string: Style,
    pub comment: Style,
    pub number: Style,
    pub operator: Style,
    pub type_name: Style,
    pub function: Style,
    pub default: Style,
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            keyword: Style::default().fg(Color::Blue),
            string: Style::default().fg(Color::Green),
            comment: Style::default().fg(Color::Gray),
            number: Style::default().fg(Color::Cyan),
            operator: Style::default().fg(Color::Yellow),
            type_name: Style::default().fg(Color::Magenta),
            function: Style::default().fg(Color::LightBlue),
            default: Style::default().fg(Color::White),
        }
    }
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with built-in language support
    pub fn new() -> Self {
        let mut highlighter = Self {
            language_patterns: HashMap::new(),
        };

        // Add language definitions
        highlighter.add_rust_support();
        highlighter.add_python_support();
        highlighter.add_javascript_support();
        highlighter.add_json_support();
        highlighter.add_markdown_support();

        highlighter
    }

    /// Highlight code and return styled spans
    pub fn highlight(&self, code: &str, language: &str) -> Vec<Line<'static>> {
        let theme = SyntaxTheme::default();
        
        if let Some(rules) = self.language_patterns.get(language) {
            self.highlight_with_rules(code, rules, &theme)
        } else {
            // No specific rules, return unstyled text
            code.lines()
                .map(|line| Line::from(Span::styled(line.to_string(), theme.default)))
                .collect()
        }
    }

    /// Highlight code with specific language rules
    fn highlight_with_rules(&self, code: &str, rules: &LanguageRules, theme: &SyntaxTheme) -> Vec<Line<'static>> {
        let mut result = Vec::new();

        for line in code.lines() {
            let highlighted_line = self.highlight_line(line, rules, theme);
            result.push(highlighted_line);
        }

        result
    }

    /// Highlight a single line of code
    fn highlight_line(&self, line: &str, rules: &LanguageRules, theme: &SyntaxTheme) -> Line<'static> {
        let mut spans = Vec::new();
        let mut current_pos = 0;
        let chars: Vec<char> = line.chars().collect();

        while current_pos < chars.len() {
            // Skip whitespace
            if chars[current_pos].is_whitespace() {
                let start = current_pos;
                while current_pos < chars.len() && chars[current_pos].is_whitespace() {
                    current_pos += 1;
                }
                spans.push(Span::styled(
                    chars[start..current_pos].iter().collect::<String>(),
                    theme.default,
                ));
                continue;
            }

            // Check for line comments
            if let Some(comment_prefix) = &rules.comments.line_comment {
                if line[current_pos..].starts_with(comment_prefix) {
                    spans.push(Span::styled(
                        line[current_pos..].to_string(),
                        theme.comment,
                    ));
                    break;
                }
            }

            // Check for strings
            if rules.strings.double_quote && chars[current_pos] == '"' {
                let (string_span, new_pos) = self.extract_string(line, current_pos, '"');
                spans.push(Span::styled(string_span, theme.string));
                current_pos = new_pos;
                continue;
            }

            if rules.strings.single_quote && chars[current_pos] == '\'' {
                let (string_span, new_pos) = self.extract_string(line, current_pos, '\'');
                spans.push(Span::styled(string_span, theme.string));
                current_pos = new_pos;
                continue;
            }

            // Check for numbers
            if rules.numbers && chars[current_pos].is_ascii_digit() {
                let start = current_pos;
                while current_pos < chars.len() && 
                      (chars[current_pos].is_ascii_alphanumeric() || chars[current_pos] == '.' || chars[current_pos] == '_') {
                    current_pos += 1;
                }
                spans.push(Span::styled(
                    chars[start..current_pos].iter().collect::<String>(),
                    theme.number,
                ));
                continue;
            }

            // Check for operators
            let mut found_operator = false;
            for operator in &rules.operators {
                if line[current_pos..].starts_with(operator) {
                    spans.push(Span::styled(operator.clone(), theme.operator));
                    current_pos += operator.len();
                    found_operator = true;
                    break;
                }
            }
            if found_operator {
                continue;
            }

            // Check for identifiers (keywords, types, functions)
            if chars[current_pos].is_alphabetic() || chars[current_pos] == '_' {
                let start = current_pos;
                while current_pos < chars.len() && 
                      (chars[current_pos].is_alphanumeric() || chars[current_pos] == '_') {
                    current_pos += 1;
                }
                
                let identifier = chars[start..current_pos].iter().collect::<String>();
                
                let style = if rules.keywords.contains(&identifier) {
                    theme.keyword
                } else if rules.types.contains(&identifier) {
                    theme.type_name
                } else {
                    theme.default
                };
                
                spans.push(Span::styled(identifier, style));
                continue;
            }

            // Default: single character
            spans.push(Span::styled(
                chars[current_pos].to_string(),
                theme.default,
            ));
            current_pos += 1;
        }

        Line::from(spans)
    }

    /// Extract a string literal from the line
    fn extract_string(&self, line: &str, start: usize, quote_char: char) -> (String, usize) {
        let mut current = start + 1; // Skip opening quote
        let chars: Vec<char> = line.chars().collect();
        
        while current < chars.len() {
            if chars[current] == quote_char {
                current += 1; // Include closing quote
                break;
            }
            if chars[current] == '\\' && current + 1 < chars.len() {
                current += 2; // Skip escaped character
            } else {
                current += 1;
            }
        }
        
        (chars[start..current].iter().collect(), current)
    }

    /// Add Rust language support
    fn add_rust_support(&mut self) {
        let rust_rules = LanguageRules {
            keywords: vec![
                "as", "break", "const", "continue", "crate", "else", "enum", "extern",
                "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
                "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while",
                "async", "await", "dyn"
            ].iter().map(|s| s.to_string()).collect(),
            
            types: vec![
                "bool", "char", "i8", "i16", "i32", "i64", "i128", "isize",
                "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64",
                "String", "str", "Vec", "Option", "Result", "Box", "Arc", "Rc"
            ].iter().map(|s| s.to_string()).collect(),
            
            strings: StringRules {
                single_quote: true,
                double_quote: true,
                triple_quote: false,
            },
            
            comments: CommentRules {
                line_comment: Some("//".to_string()),
                block_comment: Some(("/*".to_string(), "*/".to_string())),
            },
            
            numbers: true,
            operators: vec![
                "==", "!=", "<=", ">=", "&&", "||", "->", "=>", "::", "+=", "-=", "*=", "/="
            ].iter().map(|s| s.to_string()).collect(),
        };
        
        self.language_patterns.insert("rust".to_string(), rust_rules.clone());
        self.language_patterns.insert("rs".to_string(), rust_rules);
    }

    /// Add Python language support
    fn add_python_support(&mut self) {
        let python_rules = LanguageRules {
            keywords: vec![
                "and", "as", "assert", "break", "class", "continue", "def", "del", "elif",
                "else", "except", "False", "finally", "for", "from", "global", "if", "import",
                "in", "is", "lambda", "None", "nonlocal", "not", "or", "pass", "raise",
                "return", "True", "try", "while", "with", "yield", "async", "await"
            ].iter().map(|s| s.to_string()).collect(),
            
            types: vec![
                "int", "float", "str", "bool", "list", "dict", "tuple", "set", "object"
            ].iter().map(|s| s.to_string()).collect(),
            
            strings: StringRules {
                single_quote: true,
                double_quote: true,
                triple_quote: true,
            },
            
            comments: CommentRules {
                line_comment: Some("#".to_string()),
                block_comment: None,
            },
            
            numbers: true,
            operators: vec![
                "==", "!=", "<=", ">=", "and", "or", "not", "+=", "-=", "*=", "/=", "//="
            ].iter().map(|s| s.to_string()).collect(),
        };
        
        self.language_patterns.insert("python".to_string(), python_rules.clone());
        self.language_patterns.insert("py".to_string(), python_rules);
    }

    /// Add JavaScript/TypeScript language support
    fn add_javascript_support(&mut self) {
        let js_rules = LanguageRules {
            keywords: vec![
                "break", "case", "catch", "class", "const", "continue", "debugger", "default",
                "delete", "do", "else", "export", "extends", "false", "finally", "for",
                "function", "if", "import", "in", "instanceof", "let", "new", "null", "return",
                "super", "switch", "this", "throw", "true", "try", "typeof", "var", "void",
                "while", "with", "yield", "async", "await", "interface", "type"
            ].iter().map(|s| s.to_string()).collect(),
            
            types: vec![
                "boolean", "number", "string", "object", "undefined", "symbol", "any", "void"
            ].iter().map(|s| s.to_string()).collect(),
            
            strings: StringRules {
                single_quote: true,
                double_quote: true,
                triple_quote: false,
            },
            
            comments: CommentRules {
                line_comment: Some("//".to_string()),
                block_comment: Some(("/*".to_string(), "*/".to_string())),
            },
            
            numbers: true,
            operators: vec![
                "===", "!==", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "=>"
            ].iter().map(|s| s.to_string()).collect(),
        };
        
        self.language_patterns.insert("javascript".to_string(), js_rules.clone());
        self.language_patterns.insert("js".to_string(), js_rules.clone());
        self.language_patterns.insert("typescript".to_string(), js_rules.clone());
        self.language_patterns.insert("ts".to_string(), js_rules);
    }

    /// Add JSON language support
    fn add_json_support(&mut self) {
        let json_rules = LanguageRules {
            keywords: vec!["true", "false", "null"].iter().map(|s| s.to_string()).collect(),
            types: vec![],
            strings: StringRules {
                single_quote: false,
                double_quote: true,
                triple_quote: false,
            },
            comments: CommentRules {
                line_comment: None,
                block_comment: None,
            },
            numbers: true,
            operators: vec![":".to_string()],
        };
        
        self.language_patterns.insert("json".to_string(), json_rules);
    }

    /// Add Markdown language support (basic)
    fn add_markdown_support(&mut self) {
        let md_rules = LanguageRules {
            keywords: vec![],
            types: vec![],
            strings: StringRules {
                single_quote: false,
                double_quote: false,
                triple_quote: false,
            },
            comments: CommentRules {
                line_comment: None,
                block_comment: None,
            },
            numbers: false,
            operators: vec!["#".to_string(), "*".to_string(), "_".to_string()],
        };
        
        self.language_patterns.insert("markdown".to_string(), md_rules.clone());
        self.language_patterns.insert("md".to_string(), md_rules);
    }

    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        self.language_patterns.keys().cloned().collect()
    }

    /// Check if a language is supported
    pub fn supports_language(&self, language: &str) -> bool {
        self.language_patterns.contains_key(language)
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}