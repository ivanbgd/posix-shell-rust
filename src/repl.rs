//! REPL (Read-Eval-Print Loop)
//!
//! The main shell loop.
//!
//! Takes user input, parses it and calls the appropriate command or program handlers.
//!
//! # References
//!
//! - [REPL @ Wikipedia](https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop)
//! - [Bash Reference Manual](https://www.gnu.org/software/bash/manual/html_node/)
//! - Enclosing characters in single quotes preserves the literal value of each character within the quotes.
//!   [Single Quotes](https://www.gnu.org/software/bash/manual/bash.html#Single-Quotes)
//! - Enclosing characters in double quotes preserves the literal value of each character within the quotes except `\`.
//!   The backslash retains its special meaning when followed by `\`, `$`, `"` or newline.
//!   [Double Quotes](https://www.gnu.org/software/bash/manual/bash.html#Double-Quotes)
//! - A non-quoted backslash `\` is treated as an escape character.
//!   It preserves the literal value of the next character.
//!   [Escape Character](https://www.gnu.org/software/bash/manual/bash.html#Escape-Character)

use crate::cmd::run_program;
use crate::constants::{Handler, COMMANDS, DEBUG, HANDLERS, PROMPT};
use crate::errors::InvalidInputError;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{self, Write};
use std::iter::zip;

/// The main shell loop.
pub fn repl() {
    loop {
        // Print prompt
        print!("{PROMPT}");
        io::stdout().flush().expect("Flush failed");

        // Wait for user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Read line failed");

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        parse_input_and_handle_cmd(input);
    }
}

/// Parses user input and calls the appropriate command or program handler
fn parse_input_and_handle_cmd(input: &str) {
    let handlers = get_handlers();

    let items = match parse_input(input) {
        Ok(items) => items,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let items = items
        .iter()
        .map(|item| item.as_str())
        .collect::<Vec<&str>>();

    let cmd = items[0].trim();
    let args = if items.len() > 1 { &items[1..] } else { &[] };

    if DEBUG {
        eprintln!("cmd: {cmd:?}");
        eprintln!("args: {args:?}");
    }

    match handlers.get(cmd) {
        Some(&handler) => handler(args),
        None => run_program(cmd, args),
    }
}

/// Builds a table of command handlers and returns it
fn get_handlers<'a>() -> HashMap<&'a str, Handler> {
    let pairs: [(&str, Handler); COMMANDS.len()] = zip(COMMANDS, HANDLERS)
        .collect::<Vec<_>>()
        .try_into()
        .expect("Failed to convert vector to array");
    HashMap::from(pairs)
}

/// Finite state machine that changes state depending on quoting
#[derive(Debug)]
enum Fsm {
    /// No quotes are active
    Unquoted,
    /// Single quote is active
    Single,
    /// Double quote is active
    Double,
    /// Double quote nested in single quote is active
    SingleDouble,
    /// Single quote nested in double quote is active
    DoubleSingle,
}

impl Display for Fsm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            Fsm::Single => "unmatched single quotes",
            Fsm::Double => "unmatched double quotes",
            state => panic!("finishing in the '{state:?}' state is not an error"),
        };

        write!(f, "{reason}")
    }
}

/// Parses user input and returns parsed items
///
/// An item can be more than a single word if it was quoted in the input.
///
/// Conversely, two or more words from the input can be merged into a single word (item)
/// if they were separated only by a matching pair of quotes in the input.
///
/// Escaping with backslash, `\`, is also supported.
///
/// # References
/// - [Field Splitting](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_05)
/// - https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_whitespace
///
/// # Examples
///
/// ```shell
/// $   echo  hi   there,   'hello   world'  'hi''"there"'  "and""again"  "Hello   world,   it's   me"   bye   bye.
/// hi there, hello   world hi"there" andagain Hello   world,   it's   me bye bye.
/// ```
fn parse_input(input: &str) -> Result<Vec<String>, InvalidInputError> {
    // Quoted text (single or double quotes) should keep all its whitespace characters,
    // but unquoted text should not, unless escaped.
    // Unquoted text should reduce several consecutive whitespace characters to a single space, unless escaped.

    let mut items: Vec<String> = Vec::new();
    let mut item = String::new();

    let mut state = Fsm::Unquoted;
    let mut escape = false;

    for ch in input.chars() {
        match state {
            Fsm::Unquoted => match ch {
                ' ' | '\t' | '\n' => {
                    if escape {
                        item.pop();
                        item.push(ch);
                    } else if !item.is_empty() {
                        items.push(item.to_string());
                        item.clear();
                    }
                    escape = false;
                }
                '\'' => {
                    if escape {
                        item.pop();
                        item.push(ch);
                    } else {
                        state = Fsm::Single;
                    }
                    escape = false;
                }
                '"' => {
                    if escape {
                        item.pop();
                        item.push(ch);
                    } else {
                        state = Fsm::Double;
                    }
                    escape = false;
                }
                '\\' => {
                    if escape {
                        item.pop();
                    }
                    item.push(ch);
                    escape = !escape;
                }
                _ => {
                    if escape {
                        item.pop();
                    }
                    item.push(ch);
                    escape = false;
                }
            },
            Fsm::Single => match ch {
                '\'' => state = Fsm::Unquoted,
                '"' => {
                    state = Fsm::SingleDouble;
                    item.push(ch);
                }
                _ => item.push(ch),
            },
            Fsm::Double => match ch {
                '\'' => {
                    state = Fsm::DoubleSingle;
                    escape = false;
                    item.push(ch);
                }
                '"' => {
                    if escape {
                        item.pop();
                        item.push(ch);
                    } else {
                        state = Fsm::Unquoted;
                    }
                    escape = false;
                }
                '\\' => {
                    if escape {
                        item.pop();
                    }
                    item.push(ch);
                    escape = !escape;
                }
                '$' | '`' | '\n' => {
                    if escape {
                        item.pop();
                    }
                    item.push(ch);
                    escape = false;
                }
                _ => {
                    item.push(ch);
                    escape = false;
                }
            },
            Fsm::SingleDouble => match ch {
                '\'' => state = Fsm::Unquoted,
                '"' => {
                    state = Fsm::Single;
                    item.push(ch);
                }
                _ => item.push(ch),
            },
            Fsm::DoubleSingle => match ch {
                '\'' => {
                    state = Fsm::Double;
                    item.push(ch);
                }
                '"' => state = Fsm::Unquoted,
                _ => item.push(ch),
            },
        }
        if DEBUG {
            eprintln!("{ch} -> {state:?} e: {escape}\t{item}");
        }
    }
    items.push(item.to_string());

    if escape {
        return Err(InvalidInputError {
            reason: "unmatched escape character".to_string(),
        });
    }

    match state {
        Fsm::Unquoted | Fsm::SingleDouble | Fsm::DoubleSingle => Ok(items),
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
        // expected = vec![r#"  test"#.to_string()];
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
        // expected = vec![r#"'"shell world"'"#.to_string()];
        expected = vec![r#"'"shell"#.to_string(), r#"world"'"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result[1..]);

        input = r#"echo \"\'shell world\'\""#;
        // expected = vec![r#""'shell world'""#.to_string()];
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
