//! Parser for the user input
//!
//! # References
//!
//! - [Bash Reference Manual](https://www.gnu.org/software/bash/manual/html_node/)
//! - [Quoting](https://www.gnu.org/software/bash/manual/bash.html#Quoting)
//! - Enclosing characters in single quotes preserves the literal value of each character within the quotes.
//!   [Single Quotes](https://www.gnu.org/software/bash/manual/bash.html#Single-Quotes)
//! - Enclosing characters in double quotes preserves the literal value of each character within the quotes except `\`.
//!   The backslash retains its special meaning when followed by `\`, `$`, `"` or newline.
//!   [Double Quotes](https://www.gnu.org/software/bash/manual/bash.html#Double-Quotes)
//! - A non-quoted backslash `\` is treated as an escape character.
//!   It preserves the literal value of the next character.
//!   [Escape Character](https://www.gnu.org/software/bash/manual/bash.html#Escape-Character)
//! - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
//! - [Appending Redirected Output](https://www.gnu.org/software/bash/manual/bash.html#Appending-Redirected-Output)

use crate::constants::DEBUG;
use crate::errors::InvalidInputError;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};

/// Finite state machine that changes state depending on quoting and escaping
#[derive(Debug)]
enum Fsm {
    /// No quotes are active
    Unquoted,
    /// Single quote is active
    Single,
    /// Double quote is active
    Double,
    /// No quotes are active, escape is active
    UnquotedEscape,
    /// Double quote is active, escape is active
    DoubleEscape,
}

impl Display for Fsm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Fsm::Single => "unmatched single quotes",
            Fsm::Double => "unmatched double quotes",
            Fsm::UnquotedEscape | Fsm::DoubleEscape => "unmatched escape character",
            state => panic!("finishing in the '{state:?}' state is not an error"),
        };

        write!(f, "{reason}")
    }
}

/// Type of redirection containing the target, if any
///
/// This is effectively treated as a mini FSM inside the main FSM.
#[derive(Debug, PartialEq)]
pub enum Redirect {
    None,
    Stdout(String),
    Stderr(String),
    AppendStdout(String),
    AppendStderr(String),
}

impl Redirect {
    /// Creates a new [`Redirect`] from an existing one, with the new `target` value
    fn from(self, target: String) -> Self {
        match self {
            Redirect::None => Redirect::None,
            Redirect::Stdout(_) => Redirect::Stdout(target),
            Redirect::Stderr(_) => Redirect::Stderr(target),
            Redirect::AppendStdout(_) => Redirect::AppendStdout(target),
            Redirect::AppendStderr(_) => Redirect::AppendStderr(target),
        }
    }
}

/// Parses user input and returns parsed items
///
/// # References
/// - [Field Splitting](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_05)
/// - https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_whitespace
/// - [Quoting](https://www.gnu.org/software/bash/manual/bash.html#Quoting)
/// - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
pub fn parse_input(input: &str) -> Result<(Vec<String>, Redirect), InvalidInputError> {
    // An item can be more than a single word if it was quoted in the input.
    // Conversely, two or more words from the input can be merged into a single word (item)
    // if they were separated only by a matching pair of quotes in the input.
    // Escaping with backslash, `\`, is also supported.

    // Quoted text (single or double quotes) should keep all its whitespace characters,
    // but unquoted text should not, unless escaped.
    // Unquoted text should compress several consecutive whitespace characters into a single space, unless escaped.

    let mut input = input.chars().peekable();

    let mut items: Vec<String> = Vec::new();
    let mut item = String::new();

    // Redirection target
    let mut target: String = String::new();
    let mut redirect = Redirect::None;

    let mut state = Fsm::Unquoted;

    while let Some(ch) = input.next() {
        match state {
            Fsm::Unquoted => match ch {
                ' ' | '\t' | '\n' => {
                    if !item.is_empty() {
                        if redirect.eq(&Redirect::None) {
                            items.push(item.to_string());
                        } else {
                            target.push_str(item.as_str());
                        }
                        item.clear();
                    }
                }
                '\'' => {
                    state = Fsm::Single;
                }
                '"' => {
                    state = Fsm::Double;
                }
                '\\' => {
                    item.push(ch);
                    state = Fsm::UnquotedEscape;
                }
                '>' => match redirect {
                    Redirect::None => {
                        if item.eq("1") {
                            item = String::new();
                            redirect = Redirect::Stdout(target.clone());
                        } else if item.eq("2") {
                            item = String::new();
                            redirect = Redirect::Stderr(target.clone());
                        } else {
                            redirect = Redirect::Stdout(target.clone());
                        }
                    }
                    Redirect::Stdout(trg) => redirect = Redirect::AppendStdout(trg),
                    Redirect::Stderr(trg) => redirect = Redirect::AppendStderr(trg),
                    Redirect::AppendStdout(_) | Redirect::AppendStderr(_) => {
                        return Err("shell: syntax error near unexpected token `>'".into());
                    }
                },
                '&' => {
                    let next = input.peek().unwrap_or(&' ');
                    match redirect {
                        Redirect::None => {
                            // & is used for background operation, and && as logical AND.
                            unimplemented!("not implemented: &");
                        }
                        Redirect::Stdout(ref trg) => {
                            if next.eq(&'2') {
                                redirect = Redirect::Stderr(trg.to_owned());
                                input.next();
                            }
                        }
                        Redirect::Stderr(ref trg) => {
                            if next.eq(&'1') {
                                redirect = Redirect::Stdout(trg.to_owned());
                                input.next();
                            }
                        }
                        Redirect::AppendStdout(ref trg) => {
                            if next.eq(&'2') {
                                redirect = Redirect::AppendStderr(trg.to_owned());
                                input.next();
                            }
                        }
                        Redirect::AppendStderr(ref trg) => {
                            if next.eq(&'1') {
                                redirect = Redirect::AppendStdout(trg.to_owned());
                                input.next();
                            }
                        }
                    }
                }
                _ => {
                    item.push(ch);
                }
            },
            Fsm::Single => match ch {
                '\'' => state = Fsm::Unquoted,
                '"' => {
                    item.push(ch);
                }
                _ => item.push(ch),
            },
            Fsm::Double => match ch {
                '\'' => {
                    item.push(ch);
                }
                '"' => {
                    state = Fsm::Unquoted;
                }
                '\\' => {
                    item.push(ch);
                    state = Fsm::DoubleEscape;
                }
                _ => {
                    item.push(ch);
                }
            },
            Fsm::UnquotedEscape => {
                item.pop();
                item.push(ch);
                state = Fsm::Unquoted;
            }
            Fsm::DoubleEscape => match ch {
                '\'' => {
                    item.push(ch);
                    state = Fsm::Double;
                }
                '"' => {
                    item.pop();
                    item.push(ch);
                    state = Fsm::Double;
                }
                '\\' => {
                    item.pop();
                    item.push(ch);
                    state = Fsm::Double;
                }
                '$' | '`' | '\n' => {
                    item.pop();
                    item.push(ch);
                    state = Fsm::Double;
                }
                _ => {
                    item.push(ch);
                    state = Fsm::Double;
                }
            },
        }
        if DEBUG.get().is_some_and(|&debug| debug) {
            eprintln!("{ch} -> {state:?} {redirect:?}\t{item}");
        }
    }
    if redirect.eq(&Redirect::None) {
        items.push(item.to_string());
    } else {
        target.push_str(item.as_str());
    }

    let redirect = redirect.from(target);

    match state {
        Fsm::Unquoted => Ok((items, redirect)),
        other => Err(InvalidInputError {
            reason: other.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_input;
    use crate::errors::InvalidInputError;

    #[test]
    fn single_quotes_01() {
        let mut input = r#"echo hello   world"#;
        let mut expected = vec![
            "echo".to_string(),
            r#"hello"#.to_string(),
            r#"world"#.to_string(),
        ];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result);

        input = r#"echo 'hello world'"#;
        expected = vec!["echo".to_string(), r#"hello world"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result);

        input = r#"echo 'shell     example' 'test''script' world''hello"#;
        expected = vec![
            "echo".to_string(),
            r#"shell     example"#.to_string(),
            r#"testscript"#.to_string(),
            r#"worldhello"#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn single_quotes_02() {
        let mut input = r#"echo '"'"#;
        let mut expected = vec![r#"""#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo '""'"#;
        expected = vec![r#""""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn double_quotes_01() {
        let mut input = r#"echo "'""#;
        let mut expected = vec![r#"'"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "''""#;
        expected = vec![r#"''"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn double_quotes_02() {
        let mut input = r#"echo "quz  hello"  "bar""#;
        let mut expected = vec![
            "echo".to_string(),
            r#"quz  hello"#.to_string(),
            r#"bar"#.to_string(),
        ];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result);

        input = r#"echo "bar"   "shell's"   "foo""#;
        expected = vec![
            "echo".to_string(),
            r#"bar"#.to_string(),
            r#"shell's"#.to_string(),
            r#"foo"#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result);

        input = r#"echo "shell hello""#;
        expected = vec!["echo".to_string(), r#"shell hello"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result);

        input = r#"echo "hello   script"  "world""shell""#;
        expected = vec![r#"hello   script"#.to_string(), r#"worldshell"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "world"  "example's"  hello""script"#;
        expected = vec![
            r#"world"#.to_string(),
            r#"example's"#.to_string(),
            r#"helloscript"#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo  'hello   world'  'hi''there'  "and""again"  "Hello,   world.""#;
        expected = vec![
            r#"hello   world"#.to_string(),
            r#"hithere"#.to_string(),
            r#"andagain"#.to_string(),
            r#"Hello,   world."#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"   echo  hi   there,   'hello   world'  'hi''"there"'  "and""again"  "Hello   world,   it's   me"   bye   bye."#;
        expected = vec![
            r#"hi"#.to_string(),
            r#"there,"#.to_string(),
            r#"hello   world"#.to_string(),
            r#"hi"there""#.to_string(),
            r#"andagain"#.to_string(),
            r#"Hello   world,   it's   me"#.to_string(),
            r#"bye"#.to_string(),
            r#"bye."#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn escape_01() {
        let mut input = r#"echo \\"#;
        let mut expected = vec!["echo".to_string(), r#"\"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result);

        input = r#"echo '\'"#;
        expected = vec![r#"\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo '\\'"#;
        expected = vec![r#"\\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "\\""#;
        expected = vec![r#"\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn escape_02() {
        let mut input = r#"echo \'"#;
        let mut expected = vec![r#"'"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo \""#;
        expected = vec![r#"""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "\"""#;
        expected = vec![r#"""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo '\"'"#;
        expected = vec![r#"\""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo '"\""'"#;
        expected = vec![r#""\"""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "\'""#;
        expected = vec![r#"\'"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "\\'""#;
        expected = vec![r#"\'"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "\\\"""#;
        expected = vec![r#"\""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo \   test"#;
        expected = vec![r#" "#.to_string(), r#"test"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn escape_03() {
        let mut input = r#"echo "before\   after""#;
        let mut expected = vec![r#"before\   after"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo script\ \ \ \ \ \ shell"#;
        expected = vec![r#"script      shell"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo \'\"shell world\"\'"#;
        expected = vec![r#"'"shell"#.to_string(), r#"world"'"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo \"\'shell world\'\""#;
        expected = vec![r#""'shell"#.to_string(), r#"world'""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn escape_04() {
        let mut input = r#"echo "\\n""#;
        let mut expected = vec![r#"\n"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo example\ntest"#;
        expected = vec![r#"examplentest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo 'example\ntest'"#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "example\ntest""#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo example\\ntest"#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo 'example\\ntest'"#;
        expected = vec![r#"example\\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "example\\ntest""#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn escape_05() {
        let mut input = r#"echo example\"testhello\"shell"#;
        let mut expected = vec![r#"example"testhello"shell"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo 'example\"testhello\"shell'"#;
        expected = vec![r#"example\"testhello\"shell"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo 'shell\\\nscript'"#;
        expected = vec![r#"shell\\\nscript"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo 'test\\nscript'"#;
        expected = vec![r#"test\\nscript"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo 'hello\"worldexample\"test'"#;
        expected = vec![r#"hello\"worldexample\"test"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "hello'script'\\n'world""#;
        expected = vec![r#"hello'script'\n'world"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "hello\"insidequotes"script\""#;
        expected = vec![r#"hello"insidequotesscript""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn escape_06() {
        let mut input = r#"echo "world'hello'\\'example""#;
        let mut expected = vec![r#"world'hello'\'example"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "world\"insidequotes"hello\""#;
        expected = vec![r#"world"insidequoteshello""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo "mixed\"quote'test'\\""#;
        expected = vec![r#"mixed"quote'test'\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);
    }

    #[test]
    fn invalid_input() {
        let mut expected = InvalidInputError {
            reason: "unmatched escape character".to_string(),
        };

        let mut input = r#"echo \"#;
        let mut result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        expected = InvalidInputError {
            reason: "unmatched single quotes".to_string(),
        };

        input = r#"echo '"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        input = r#"echo '''"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        input = r#"echo '\''"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        expected = InvalidInputError {
            reason: "unmatched double quotes".to_string(),
        };

        input = r#"echo ""#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        input = r#"echo """"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);
    }
}
