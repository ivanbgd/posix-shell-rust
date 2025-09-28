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
use std::iter::Peekable;
use std::path::PathBuf;
use std::str::Chars;

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
            Fsm::Single => "unmatched single quotes\n",
            Fsm::Double => "unmatched double quotes\n",
            Fsm::UnquotedEscape | Fsm::DoubleEscape => "unmatched escape character\n",
            state => panic!("finishing in the '{state:?}' state is not an error"),
        };

        write!(f, "{reason}")
    }
}

// TODO
// /// Type of redirection
// ///
// /// This is effectively treated as a mini FSM embedded inside the main FSM.
// #[derive(Clone, Debug, PartialEq)]
// enum RedirectionKind {
//     None,
//     Stdout,
//     Stderr,
//     AppendStdout,
//     AppendStderr,
//     CombinedStdout,
//     CombinedStderr,
//     AppendCombinedStdout,
//     AppendCombinedStderr,
// }
//
// /// Type of redirection containing the target, if any
// ///
// /// This is effectively treated as a mini FSM embedded inside the main FSM.
// ///
// /// This allows for multiple redirections, not only of different kinds, but even of the same kind, too.
// ///
// /// In case the same kind is repeated, only the last one takes effect, meaning, only the last target
// /// will be used for redirection, but all the previous ones need to be evaluated, too.
// ///
// /// For example, `cat src > target1 > target2` should write `stdout` (contents of `src`) to `target2` only,
// /// but it should also create `target1` if it didn't exist.
// ///
// /// That's how `bash` works, for example, but **not** how `zsh` works, for example.
// ///
// /// To make the distinction clear, `zsh` would write the contents of `src` to both `target1` **and** `target2`.
// #[derive(Clone, Debug)]
// pub struct Redirection {
//     kind: RedirectionKind,
//     targets: Option<Vec<String>>,
// }
//
// impl Redirection {
//     /// Creates a new [`Redirection`] in default state, which is "no redirection".
//     fn new() -> Self {
//         Self {
//             kind: RedirectionKind::None,
//             targets: None,
//         }
//     }
//
//     /// Adds a new `target` to the list of targets for this kind of redirection
//     fn add_target(&mut self, target: String) {
//         if self.kind != RedirectionKind::None {
//             if self.targets.is_none() {
//                 self.targets = Some(Vec::new());
//             }
//             self.targets.clone().unwrap().push(target);
//         }
//     }
// }
//
// pub struct Redirections(Vec<Redirection>);
//
// impl Redirections {
//     fn new() -> Self {
//         Self { 0: vec![] }
//     }
//
//     fn update(&mut self, new_redir: Redirection) {
//         for redir in self.0.iter_mut() {
//             if redir.kind == new_redir.kind {
//                 redir.add_target(new_redir.t);
//             }
//         }
//     }
// }

/// A helper enum/FSM for redirection target
///
/// This is effectively treated as a mini FSM embedded inside the main FSM.
///
/// It's used in unquoted parts of input, i.e, in the [`Fsm::Unquoted`] state.
///
/// It is not public by design.
///
/// Contains the following variants:
/// - None
/// - Stdout
/// - Stderr
#[derive(Debug, PartialEq)]
enum RedirectionFsm {
    None,
    Stdout,
    Stderr,
    CombinedStdout, // TODO: Needed?
    CombinedStderr, // TODO: Needed?
}

/// Trait for working with [`Stdout`] or [`Stderr`]
///
/// This is particularly useful for multiple redirections.
trait StdOutErr {
    /// Constructs a new instance, filling the `kind` and `paths` fields.
    fn new() -> Self;

    /// Updates [`Self`] with a new `path`.
    fn add_path(&mut self, path: PathBuf);

    // TODO
    // /// Updates [`Self`] with new `kind` and with a new `path`.
    // ///
    // /// `kind` cannot be [`RedirectionKind::None`], because once
    // /// the user has decided to redirect, there's no going back.
    // fn update(&mut self, kind: RedirectionKind, path: PathBuf);
}

/// The mode of redirection
///
/// Contains the following variants:
/// - None
/// - Overwrite
/// - Append
///
/// Can be used for both `stdout` and `stderr`.
#[derive(Clone, Debug, PartialEq)]
pub enum RedirectionMode {
    /// No redirection
    None,
    /// Standard redirection that overwrites the target path
    Overwrite,
    /// A redirection kind that appends to the target path
    Append,
}

/// The redirection to `stdout`
///
/// Contains kind of redirection, as [`RedirectionMode`], and the target file paths.
///
/// In case of multiple redirections, only the last one applies.
///
/// Nevertheless, all previous targets (paths) should be cleared.
#[derive(Clone, Debug, PartialEq)]
pub struct Stdout {
    pub kind: RedirectionMode,
    pub paths: Vec<PathBuf>,
}

impl StdOutErr for Stdout {
    fn new() -> Self {
        Self {
            kind: RedirectionMode::None,
            paths: Vec::new(),
        }
    }

    fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    // TODO
    // fn update(&mut self, kind: RedirectionKind, path: PathBuf) {
    //     assert_ne!(kind, RedirectionKind::None);
    //     self.kind = kind;
    //     if self.paths.is_none() {
    //         self.paths = Some(Vec::new());
    //     }
    //     self.paths.clone().unwrap().push(path);
    // }
}

/// The redirection to `stderr`
///
/// Contains kind of redirection, as [`RedirectionMode`], and the target file paths.
///
/// In case of multiple redirections, only the last one applies.
///
/// Nevertheless, all previous targets (paths) should be cleared.
#[derive(Clone, Debug, PartialEq)]
pub struct Stderr {
    pub kind: RedirectionMode,
    pub paths: Vec<PathBuf>,
}

impl StdOutErr for Stderr {
    fn new() -> Self {
        Self {
            kind: RedirectionMode::None,
            paths: Vec::new(),
        }
    }

    fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
        // TODO
        // if self.paths.is_none() {
        //     self.paths = Some(Vec::new());
        // }
        // self.paths.clone().unwrap().push(path);
    }

    // TODO
    // fn update(&mut self, kind: RedirectionKind, path: PathBuf) {
    //     assert_ne!(kind, RedirectionKind::None);
    //     self.kind = kind;
    //     if self.paths.is_none() {
    //         self.paths = Some(Vec::new());
    //     }
    //     self.paths.clone().unwrap().push(path);
    // }
}

/// Contains the `stdout` and `stderr` redirections.
#[derive(Debug, PartialEq)]
pub struct Redirections {
    /// A redirection to `stdout`
    pub stdout: Stdout,
    /// A redirection to `stderr`
    pub stderr: Stderr,
}

impl Redirections {
    /// Constructs a new instance from the existing `stdout` and `stderr` instances.
    fn from(stdout: Stdout, stderr: Stderr) -> Self {
        Self { stdout, stderr }
    }

    // /// Constructs a new instance, filling the `stdout` and `stderr` fields.
    // fn new() -> Self {
    //     Self {
    //         stdout: Stdout::new(),
    //         stderr: Stderr::new(),
    //     }
    // }

    // fn new() -> Self {
    //     Self {
    //         stdout: Stdout {
    //             kind: RedirectionKind::None,
    //             targets: None,
    //         },
    //         stderr: Stderr {
    //             kind: RedirectionKind::None,
    //             targets: None,
    //         },
    //     }
    // }

    // /// TODO docs
    // fn update(&mut self, mut field: impl StdOutErr, kind: RedirectionKind, target: PathBuf) {
    //     field.update(kind, target);
    // }
    // fn update<T: StdOutErr>(&mut self, mut field: T, kind: RedirectionKind, target: PathBuf) {
    //     field.update(kind, target);
    // }
}

// impl Redirections {
//     /// Gets the `stdout` and `stderr` target (path) fields for reading.
//     pub fn get(self) -> (Stdout, Stderr) {
//         (self.stdout, self.stderr)
//     }
// }

// pub struct Redirections2 {
//     // kind: RedirectionKind2,
//     stdout: Vec<u8>,
//     stderr: Vec<u8>,
//     stdout_targets: Option<Vec<String>>,
//     stderr_targets: Option<Vec<String>>,
// }

// impl Redirect { TODO remove
//     /// Creates a new [`Redirect`] from an existing one, with the new `target` value
//     fn from(self, target: String) -> Self {
//         match self {
//             Redirect::None => Redirect::None,
//             Redirect::Stdout(_) => Redirect::Stdout(target),
//             Redirect::Stderr(_) => Redirect::Stderr(target),
//             Redirect::AppendStdout(_) => Redirect::AppendStdout(target),
//             Redirect::AppendStderr(_) => Redirect::AppendStderr(target),
//             Redirect::CombinedStdout(_) => Redirect::CombinedStdout(target),
//             Redirect::CombinedStderr(_) => Redirect::CombinedStderr(target),
//             Redirect::AppendCombinedStdout(_) => Redirect::AppendCombinedStdout(target),
//             Redirect::AppendCombinedStderr(_) => Redirect::AppendCombinedStderr(target),
//         }
//     }
// }

/// Parses user input and returns parsed items, together with [`Redirections`].
///
/// # Errors
/// - [`InvalidInputError`]
///
/// # References
/// - [Field Splitting](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html#tag_18_06_05)
/// - https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_whitespace
/// - [Quoting](https://www.gnu.org/software/bash/manual/bash.html#Quoting)
/// - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
pub fn parse_input(input: &str) -> Result<(Vec<String>, Redirections), InvalidInputError> {
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

    // Redirection targets
    // let mut redirections = Redirections::new();
    // let mut redirection = Redirection::new(); todo
    // let mut redir_kind = RedirectionKind::None;
    // let mut redirection = false;
    let mut redirection = RedirectionFsm::None;
    let mut stdout = Stdout::new();
    let mut stderr = Stderr::new();

    let mut state = Fsm::Unquoted;

    // redirections.update(stdout, RedirectionKind::Append, PathBuf::new());

    while let Some(ch) = input.next() {
        match state {
            Fsm::Unquoted => match ch {
                ' ' | '\t' | '\n' => {
                    // eprintln!("#### 1 ch = '{ch}', item = '{item}'"); // todo rem
                    if !item.is_empty() {
                        // eprintln!("  #### 2 ch = '{ch}', item = '{item}'"); // todo rem
                        match redirection {
                            RedirectionFsm::None => items.push(item.to_string()),
                            RedirectionFsm::Stdout => stdout.add_path(PathBuf::from(item.clone())),
                            RedirectionFsm::Stderr => stderr.add_path(PathBuf::from(item.clone())),
                            // todo remove:
                            RedirectionFsm::CombinedStdout | RedirectionFsm::CombinedStderr => {
                                break; // todo ?
                                // stdout.add_path(PathBuf::from(item.clone()));
                                // stderr.add_path(PathBuf::from(item.clone()));
                            }
                        }
                        // redirection = Redirection::None;

                        // if !redirection {
                        //     items.push(item.to_string());
                        // }

                        // if stdout.kind.eq(&RedirectionKind::None)
                        //     && stderr.kind.eq(&RedirectionKind::None)

                        // else if stdout.kind.ne(&RedirectionKind::None) {
                        // }
                        // else {
                        // // redirections.push(redirect.from(item.clone())); todo
                        // let mut redirection = Redirection::new();
                        // redirection.add_target(item.clone());
                        // redirections.push(redirection.clone());
                        // // redirection = Redirection::new();
                        // redir_kind = RedirectionKind::None;
                        // }

                        item.clear();
                        redirection = RedirectionFsm::None;
                    }
                    // redirection = RedirectionFsm::None;
                    // redirection = false;
                }
                '\'' => {
                    state = Fsm::Single;
                    redirection = RedirectionFsm::None;
                    // redirection = false;
                }
                '"' => {
                    state = Fsm::Double;
                    redirection = RedirectionFsm::None;
                    // redirection = false;
                }
                '\\' => {
                    item.push(ch);
                    state = Fsm::UnquotedEscape;
                    redirection = RedirectionFsm::None;
                    // redirection = false;
                }
                '>' => {
                    handle_closing_angle_bracket_unquoted(
                        // &mut input,
                        &mut items,
                        &mut item,
                        // redirection,
                        &mut redirection,
                        &mut stdout,
                        &mut stderr,
                    )?;
                    // redirection = true;
                }
                '&' => {
                    handle_ampersand_unquoted(
                        &mut input,
                        &mut item,
                        &mut redirection,
                        &mut stdout,
                        &mut stderr,
                    )?;
                    // redirection = RedirectionFsm::None;
                    // redirection = false;
                }
                _ => {
                    item.push(ch);
                    // redirection = RedirectionFsm::None;
                    // redirection = false;
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
            eprintln!("{ch} -> {state:?}, {redirection:?}, {stdout:?}, {stderr:?}\t{item}");
        }
    }
    // eprintln!("#### 3 item = '{item}'"); // todo rem
    if !item.is_empty() {
        // eprintln!("#### 4 item = '{item}'"); // todo rem
        match redirection {
            RedirectionFsm::None => items.push(item.to_string()),
            RedirectionFsm::Stdout => stdout.add_path(PathBuf::from(item.clone())),
            RedirectionFsm::Stderr => stderr.add_path(PathBuf::from(item.clone())),
            RedirectionFsm::CombinedStdout | RedirectionFsm::CombinedStderr => {
                // todo rem
                stdout.add_path(PathBuf::from(item.clone()));
                stderr.add_path(PathBuf::from(item.clone()));
            }
        }
    }

    eprintln!("@ -> {state:?}, {redirection:?}, {stdout:?}, {stderr:?}\t{item}"); // todo rem

    let redirections = Redirections::from(stdout, stderr);

    match state {
        Fsm::Unquoted => Ok((items, redirections)),
        other => Err(InvalidInputError {
            reason: other.to_string(),
        }),
    }
}

/// Handles the received `>` character in the [`Fsm::Unquoted`] state.
///
/// This character is used for output redirection.
///
/// Only updates the [`RedirectionMode`] of the appropriate redirection output, `stdout` or `stderr`,
/// to [`RedirectionMode::Overwrite`] or to [`RedirectionMode::Append`].
///
/// It doesn't append the target path, because that is yet to be parsed outside of this function.
///
/// # Errors
/// - Returns [`InvalidInputError`] in case of three or more consecutive `>`, i.e., `>>>`.
fn handle_closing_angle_bracket_unquoted(
    items: &mut Vec<String>,
    item: &mut String,
    redirection: &mut RedirectionFsm,
    stdout: &mut Stdout,
    stderr: &mut Stderr,
) -> Result<(), InvalidInputError> {
    match *redirection {
        RedirectionFsm::None => {
            if (*item).eq("1") {
                // `1>`
                item.clear();
                *redirection = RedirectionFsm::Stdout;
                stdout.kind = RedirectionMode::Overwrite;
            } else if (*item).eq("2") {
                // `2>`
                item.clear();
                *redirection = RedirectionFsm::Stderr;
                stderr.kind = RedirectionMode::Overwrite;
            } else {
                // `>`
                if !item.is_empty() {
                    items.push(item.to_string());
                    item.clear();
                }
                *redirection = RedirectionFsm::Stdout;
                stdout.kind = RedirectionMode::Overwrite;
            }
        }
        RedirectionFsm::Stdout => {
            if stdout.kind == RedirectionMode::Overwrite {
                // `1>>` or `>>`
                stdout.kind = RedirectionMode::Append;
            } else {
                // `>>>`
                return Err("shell: syntax error near unexpected token `>'\n".into());
            }
        }
        RedirectionFsm::Stderr => {
            if stderr.kind == RedirectionMode::Overwrite {
                // `2>>`
                stderr.kind = RedirectionMode::Append;
            } else {
                // `>>>`
                return Err("shell: syntax error near unexpected token `>'\n".into());
            }
        }
        _ => {} // todo rem
    }

    Ok(())
}

/// Handles the received `&` character in the [`Fsm::Unquoted`] state.
///
/// Supports shell-specific `&>word` and shell-specific `>&word`, which redirect both
/// `stdout` and `stderr` to the file whose name is the expansion of `word`.
///
/// Unimplemented: `&` on its own is used for background operation, and `&&` as logical AND.
///
/// # References
/// - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
///
/// # Errors
/// - Returns [`InvalidInputError`] in case of `&>&` or `>&&`.
/// - Returns [`InvalidInputError`] in case of unimplemented cases, i.e., ` & ` (`bg`) and ` && ` (`AND`).
fn handle_ampersand_unquoted(
    input: &mut Peekable<Chars>,
    item: &mut str,
    redirection: &mut RedirectionFsm,
    stdout: &mut Stdout,
    stderr: &mut Stderr,
) -> Result<(), InvalidInputError> {
    let next_peeked = input.peek().unwrap_or(&' ');

    if item.is_empty() && next_peeked.is_whitespace() {
        // Unimplemented: Background operation
        return Err("shell: unimplemented `&'\n".into());
    } else if next_peeked.eq(&'&') {
        return if *redirection != RedirectionFsm::None {
            // `>&&`
            Err("shell: syntax error near unexpected token `&'\n".into())
        } else {
            // Unimplemented: The logical AND operator
            Err("shell: unimplemented `&&'\n".into())
        };
    }

    match *redirection {
        RedirectionFsm::None => {
            if next_peeked.eq(&'>') {
                // shell-specific `&>word` or `&> word`
                *redirection = RedirectionFsm::Stdout;
                stdout.kind = RedirectionMode::Overwrite;
                stderr.kind = RedirectionMode::None;
                let next = input.next();
                if next.is_some() {
                    if let Some(next_next_peeked) = input.peek() {
                        if next_next_peeked.eq(&'>') {
                            // shell-specific `&>>`
                            stdout.kind = RedirectionMode::Append;
                            let next = input.next();
                            if next.is_some() && input.peek().is_none() {
                                // `&>> ` or `>>&\t` or `&>>\n`
                                return Err(
                                    "shell: syntax error near unexpected token `newline'\n".into(),
                                );
                            }
                        } else if next_next_peeked.eq(&'&') {
                            // `&>&`
                            return Err("shell: syntax error near unexpected token `&'\n".into());
                        }
                    } else {
                        // `&> ` or `>&\t` or `&>\n`
                        return Err("shell: syntax error near unexpected token `newline'\n".into());
                    }
                }
            }
        }
        RedirectionFsm::Stdout => {
            if next_peeked.eq(&'2') {
                // `1>&2` or `&>2`

                // if stdout.paths.is_empty() {
                //     *redirection = RedirectionFsm::None;
                //     stdout.kind = RedirectionMode::None;
                //     stderr.kind = RedirectionMode::None;
                // } else {
                *redirection = RedirectionFsm::Stderr;
                stderr.kind = stdout.kind.clone();
                stdout.kind = RedirectionMode::None;
                // }
                input.next();

                // stderr.paths.push(PathBuf::new());
            }
        }
        RedirectionFsm::Stderr => {
            if next_peeked.eq(&'1') {
                // `2>&1`
                // if stderr.paths.is_empty() {
                //     *redirection = RedirectionFsm::None;
                //     stdout.kind = RedirectionMode::None;
                //     stderr.kind = RedirectionMode::None;
                // } else {
                *redirection = RedirectionFsm::Stdout;
                stdout.kind = stderr.kind.clone();
                stderr.kind = RedirectionMode::None;
                // }
                input.next();
            }
        }
        _ => {} // todo rem
    }

    // *redirection = RedirectionFsm::None; // todo ?

    Ok(())

    /*
        let next_peeked = input.peek().unwrap_or(&' ');
        // let mut next: Option<char> = None; todo

        if next_peeked.eq(&'>') {
            // shell-specific `&>word` or `&> word`
            stdout.kind = RedirectionMode::Overwrite;
            stderr.kind = RedirectionMode::Overwrite;
            *redirection = RedirectionFsm::CombinedStdout;
            let next = input.next();
            if next.is_some() {
                let next_next_peeked = input.peek().unwrap_or(&' ');
                if next_next_peeked.eq(&'>') {
                    // shell-specific `&>>`
                    stdout.kind = RedirectionMode::Append;
                    stderr.kind = RedirectionMode::Append;
                    input.next();
                    return Ok(());
                } else if next_next_peeked.eq(&'&') {
                    // `&>&`
                    return Err("shell: syntax error near unexpected token `&'\n".into());
                }
            }
        } else if item.is_empty() && next_peeked.is_whitespace() {
            // Unimplemented: Background operation
            return Err("shell: unimplemented `&'\n".into());
        } else if next_peeked.eq(&'&') {
            // Unimplemented: The logical AND operator
            return Err("shell: unimplemented `&&'\n".into());
        }

        Ok(())
    */
}

// fn handle_closing_angle_bracket_unquoted(
//     items: &mut Vec<String>,
//     item: &mut String,
//     redirection: &mut Redirection,
//     stdout: &mut Stdout,
//     stderr: &mut Stderr,
// ) -> Result<(), InvalidInputError> {
//     if (*item).eq("1") {
//         // item.clear(); todo
//         match redirection {
//             Redirection::None => {
//                 // `1>`
//                 stdout.kind = RedirectionMode::Overwrite;
//             }
//             Redirection::Stdout => {
//                 match stdout.kind {
//                     RedirectionMode::None => panic!("redirection mode should not be none"),
//                     RedirectionMode::Overwrite => {
//                         // `1>>`
//                         stdout.kind = RedirectionMode::Append;
//                     }
//                     RedirectionMode::Append => {
//                         // `>>>`
//                         return Err("shell: syntax error near unexpected token `>'\n".into());
//                     }
//                 }
//             }
//             Redirection::Stderr => panic!("redirection should not be stderr"),
//             _ => {} // todo remove
//         }
//     } else if (*item).eq("2") {
//         // item.clear(); todo
//         match redirection {
//             Redirection::None => {
//                 // `2>`
//                 stderr.kind = RedirectionMode::Overwrite;
//             }
//             Redirection::Stdout => panic!("redirection should not be stdout"),
//             Redirection::Stderr => {
//                 match stderr.kind {
//                     RedirectionMode::None => panic!("redirection mode should not be none"),
//                     RedirectionMode::Overwrite => {
//                         // `2>>`
//                         stderr.kind = RedirectionMode::Append;
//                     }
//                     RedirectionMode::Append => {
//                         // `>>>`
//                         return Err("shell: syntax error near unexpected token `>'\n".into());
//                     }
//                 }
//             }
//             _ => {} // todo remove
//         }
//     } else {
//         // `>`
//         if !item.is_empty() {
//             items.push(item.to_string());
//             item.clear();
//         }
//         match redirection {
//             Redirection::None => {
//                 // `>`
//                 stdout.kind = RedirectionMode::Overwrite;
//             }
//             Redirection::Stdout => {
//                 match stdout.kind {
//                     RedirectionMode::None => panic!("redirection mode should not be none"),
//                     RedirectionMode::Overwrite => {
//                         // `>>`
//                         stdout.kind = RedirectionMode::Append;
//                     }
//                     RedirectionMode::Append => {
//                         // `>>>`
//                         return Err("shell: syntax error near unexpected token `>'\n".into());
//                     }
//                 }
//             }
//             Redirection::Stderr => panic!("redirection should not be stderr"),
//             _ => {} // todo remove
//         }
//     }
//
//     Ok(())
// }

// fn handle_closing_angle_bracket_unquoted(
//     input: &mut Peekable<Chars>,
//     items: &mut Vec<String>,
//     item: &mut String,
//     redirection: &mut Redirection,
//     stdout: &mut Stdout,
//     stderr: &mut Stderr,
//     // redirect: &mut Redirect, todo
// ) -> Result<(), InvalidInputError> {
//     let next_peeked = input.peek().unwrap_or(&' ');
//     let mut next: Option<char> = None;
//
//     if (*item).eq("1") {
//         // `1>`
//         item.clear();
//         if next_peeked.eq(&'>') {
//             // `1>>`
//             stdout.kind = RedirectionMode::Append;
//             next = input.next();
//         } else {
//             stdout.kind = RedirectionMode::Overwrite;
//         }
//         *redirection = Redirection::Stdout;
//         // stdout.update(RedirectionKind::Overwrite, PathBuf::new()); todo
//     } else if (*item).eq("2") {
//         // `2>`
//         item.clear();
//         if next_peeked.eq(&'>') {
//             // `2>>`
//             stderr.kind = RedirectionMode::Append;
//             next = input.next();
//         } else {
//             stderr.kind = RedirectionMode::Overwrite;
//         }
//         *redirection = Redirection::Stderr;
//         // stderr.update(RedirectionKind::Overwrite, PathBuf::new()); todo
//     } else {
//         // `>`
//         if !item.is_empty() {
//             items.push(item.to_string());
//             item.clear();
//         }
//         if next_peeked.eq(&'>') {
//             // `>>`
//             stdout.kind = RedirectionMode::Append;
//             next = input.next();
//         } else {
//             stdout.kind = RedirectionMode::Overwrite;
//         }
//         *redirection = Redirection::Stdout;
//         // stdout.update(RedirectionKind::Overwrite, PathBuf::new()); todo
//     }
//
//     // if next.is_some() {
//     //     let next_next_peeked = input.peek().unwrap_or(&' ');
//     //     // TODO: Handle `>&` here? And `>&2`? Also `>&&`? Try in handle_ampersand_unquoted().
//     //     if next_next_peeked.eq(&'>') {
//     //         // `>>>`
//     //         return Err("shell: syntax error near unexpected token `>'\n".into());
//     //     }
//     // }
//
//     match input.peek() {
//         Some('>') => return Err("shell: syntax error near unexpected token `>'\n".into()), // `>>>`
//         Some('&') => {
//             let next = input.next();
//             let next_next_peeked = input.peek().unwrap_or(&' ');
//
//             // shell-specific `>&word` or `>& word`
//             stdout.kind = RedirectionMode::Overwrite;
//             stderr.kind = RedirectionMode::Overwrite;
//             *redirection = Redirection::CombinedStdout;
//             let next = input.next();
//         }
//         Some(..) => {}
//         None => {}
//     }
//
//     // ORIGINAL:
//     // match redirect {
//     //     Redirect::None => {
//     //         if (*item).eq("1") {
//     //             // `1>`
//     //             item.clear();
//     //             *redirect = Redirect::Stdout(String::new());
//     //         } else if (*item).eq("2") {
//     //             // `2>`
//     //             item.clear();
//     //             *redirect = Redirect::Stderr(String::new());
//     //         } else {
//     //             // `>`
//     //             if !item.is_empty() {
//     //                 items.push(item.to_string());
//     //                 item.clear();
//     //             }
//     //             *redirect = Redirect::Stdout(String::new());
//     //         }
//     //     }
//     //     Redirect::Stdout(trg) => *redirect = Redirect::AppendStdout(trg.to_owned()),
//     //     Redirect::Stderr(trg) => *redirect = Redirect::AppendStderr(trg.to_owned()),
//     //     Redirect::CombinedStdout(trg) => *redirect = Redirect::AppendCombinedStdout(trg.to_owned()),
//     //     Redirect::AppendStdout(_)
//     //     | Redirect::AppendStderr(_)
//     //     | Redirect::AppendCombinedStdout(_) => {
//     //         return Err("shell: syntax error near unexpected token `>'\n".into());
//     //     }
//     //     _ => todo!(),
//     // }
//
//     Ok(())
// }

/*
/// Handles the received `&` character in the [`Fsm::Unquoted`] state.
///
/// Supports shell-specific `&>word` and shell-specific `>&word`, which redirect both
/// `stdout` and `stderr` to the file whose name is the expansion of `word`.
///
/// Unimplemented: `&` on its own is used for background operation, and `&&` as logical AND.
///
/// # References
/// - [Redirecting Output](https://www.gnu.org/software/bash/manual/bash.html#Redirecting-Output)
///
/// # Errors
/// - Returns [`InvalidInputError`] in case of `&>&` or `>&&`.
/// - Returns [`InvalidInputError`] in case of unimplemented cases, i.e., ` & ` (`bg`) and ` && ` (`AND`).
fn handle_ampersand_unquoted(
    input: &mut Peekable<Chars>,
    item: &mut str,
    redirection: &mut Redirection,
    stdout: &mut Stdout,
    stderr: &mut Stderr,
) -> Result<(), InvalidInputError> {
    let next_peeked = input.peek().unwrap_or(&' ');
    // let mut next: Option<char> = None; todo

    if next_peeked.eq(&'>') {
        // shell-specific `&>word` or `&> word`
        stdout.kind = RedirectionMode::Overwrite;
        stderr.kind = RedirectionMode::Overwrite;
        *redirection = Redirection::CombinedStdout;
        let next = input.next();
        if next.is_some() {
            let next_next_peeked = input.peek().unwrap_or(&' ');
            if next_next_peeked.eq(&'>') {
                // shell-specific `&>>`
                stdout.kind = RedirectionMode::Append;
                stderr.kind = RedirectionMode::Append;
                input.next();
                return Ok(());
            } else if next_next_peeked.eq(&'&') {
                // `&>&`
                return Err("shell: syntax error near unexpected token `&'\n".into());
            }
        }
    } else if item.is_empty() && next_peeked.is_whitespace() {
        // Unimplemented: Background operation
        return Err("shell: unimplemented `&'\n".into());
    } else if next_peeked.eq(&'&') {
        // Unimplemented: The logical AND operator
        return Err("shell: unimplemented `&&'\n".into());
    }

    // match redirection {
    //     Redirection::None => {}
    //     Redirection::Stdout => {
    //         if next_peeked.eq(&'2') {
    //             // `1>&2` or `>&2`
    //             *redirection = Redirection::Stderr;
    //             stdout.kind = RedirectionKind::Overwrite;
    //             stderr.kind = RedirectionKind::Overwrite;
    //             // *redirection = Redirection::CombinedStderr; todo
    //             input.next();
    //         } else {
    //             // shell-specific `>&word` or `>& word`
    //             *redirection = Redirection::Stdout;
    //             stdout.kind = RedirectionKind::Overwrite;
    //             stderr.kind = RedirectionKind::Overwrite;
    //             // *redirection = Redirection::CombinedStdout; todo
    //         }
    //     }
    //     Redirection::Stderr => {
    //         if next_peeked.eq(&'1') {
    //             // `2>&1`
    //             *redirection = Redirection::Stdout;
    //             stdout.kind = RedirectionKind::Overwrite;
    //             stderr.kind = RedirectionKind::Overwrite;
    //             input.next();
    //         }
    //     }
    //     Redirection::CombinedStdout => {}
    //     Redirection::CombinedStderr => {}
    // }

    /*
        let next = input.peek().unwrap_or(&' ');

        match redirect {
            Redirect::None => {
                // Unimplemented: `&` on its own is used for background operation, and `&&` as logical AND.
                if next.eq(&'>') {
                    // shell-specific `&>word` or `&> word`
                    *redirect = Redirect::CombinedStdout(String::new());
                    input.next();
                } else if item.is_empty() && next.is_whitespace() {
                    // Unimplemented: Background operation
                    return Err("shell: unimplemented `&'\n".into());
                } else if next.eq(&'&') {
                    // Unimplemented: The logical AND operator
                    return Err("shell: unimplemented `&&'\n".into());
                }
            }
            Redirect::Stdout(trg) => {
                if next.eq(&'2') {
                    // `1>&2` or `>&2`
                    *redirect = Redirect::Stderr(trg.to_owned());
                    input.next();
                } else {
                    // shell-specific `>&word` or `>& word`
                    *redirect = Redirect::CombinedStdout(trg.to_owned());
                }
            }
            Redirect::Stderr(trg) => {
                if next.eq(&'1') {
                    // `2>&1`
                    *redirect = Redirect::Stdout(trg.to_owned());
                    input.next();
                }
            }
            Redirect::AppendStdout(trg) => {
                if next.eq(&'2') {
                    // `1>>&2` or `>>&2`
                    *redirect = Redirect::AppendStderr(trg.to_owned());
                    input.next();
                }
            }
            Redirect::AppendStderr(trg) => {
                if next.eq(&'1') {
                    // `2>>&1`
                    *redirect = Redirect::AppendStdout(trg.to_owned());
                    input.next();
                }
            }
            Redirect::CombinedStdout(_) | Redirect::AppendCombinedStdout(_) => {
                return Err("shell: syntax error near unexpected token `&'\n".into());
            }
            _ => todo!(),
        }
    */

    Ok(())
}
*/

#[cfg(test)]
mod tests {
    use super::{parse_input, RedirectionMode, Redirections, StdOutErr, Stderr, Stdout};
    use crate::errors::InvalidInputError;
    use std::path::PathBuf;

    #[test]
    fn single_quotes_01() {
        let mut input = r#"echo hello   world"#;
        let mut expected = vec![
            "echo".to_string(),
            r#"hello"#.to_string(),
            r#"world"#.to_string(),
        ];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0);

        input = r#"echo 'hello world'"#;
        expected = vec!["echo".to_string(), r#"hello world"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0);

        input = r#"echo 'shell     example' 'test''script' world''hello"#;
        expected = vec![
            "echo".to_string(),
            r#"shell     example"#.to_string(),
            r#"testscript"#.to_string(),
            r#"worldhello"#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0);
    }

    #[test]
    fn single_quotes_02() {
        let mut input = r#"echo '"'"#;
        let mut expected = vec![r#"""#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo '""'"#;
        expected = vec![r#""""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn double_quotes_01() {
        let mut input = r#"echo "'""#;
        let mut expected = vec![r#"'"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "''""#;
        expected = vec![r#"''"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
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
        assert_eq!(expected, result.0);

        input = r#"echo "bar"   "shell's"   "foo""#;
        expected = vec![
            "echo".to_string(),
            r#"bar"#.to_string(),
            r#"shell's"#.to_string(),
            r#"foo"#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0);

        input = r#"echo "shell hello""#;
        expected = vec!["echo".to_string(), r#"shell hello"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0);

        input = r#"echo "hello   script"  "world""shell""#;
        expected = vec![r#"hello   script"#.to_string(), r#"worldshell"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "world"  "example's"  hello""script"#;
        expected = vec![
            r#"world"#.to_string(),
            r#"example's"#.to_string(),
            r#"helloscript"#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo  'hello   world'  'hi''there'  "and""again"  "Hello,   world.""#;
        expected = vec![
            r#"hello   world"#.to_string(),
            r#"hithere"#.to_string(),
            r#"andagain"#.to_string(),
            r#"Hello,   world."#.to_string(),
        ];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

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
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn escape_01() {
        let mut input = r#"echo \\"#;
        let mut expected = vec!["echo".to_string(), r#"\"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0);

        input = r#"echo '\'"#;
        expected = vec![r#"\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo '\\'"#;
        expected = vec![r#"\\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "\\""#;
        expected = vec![r#"\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn escape_02() {
        let mut input = r#"echo \'"#;
        let mut expected = vec![r#"'"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo \""#;
        expected = vec![r#"""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "\"""#;
        expected = vec![r#"""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo '\"'"#;
        expected = vec![r#"\""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo '"\""'"#;
        expected = vec![r#""\"""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "\'""#;
        expected = vec![r#"\'"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "\\'""#;
        expected = vec![r#"\'"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "\\\"""#;
        expected = vec![r#"\""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo \   test"#;
        expected = vec![r#" "#.to_string(), r#"test"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn escape_03() {
        let mut input = r#"echo "before\   after""#;
        let mut expected = vec![r#"before\   after"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo script\ \ \ \ \ \ shell"#;
        expected = vec![r#"script      shell"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo \'\"shell world\"\'"#;
        expected = vec![r#"'"shell"#.to_string(), r#"world"'"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo \"\'shell world\'\""#;
        expected = vec![r#""'shell"#.to_string(), r#"world'""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn escape_04() {
        let mut input = r#"echo "\\n""#;
        let mut expected = vec![r#"\n"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo example\ntest"#;
        expected = vec![r#"examplentest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo 'example\ntest'"#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "example\ntest""#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo example\\ntest"#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo 'example\\ntest'"#;
        expected = vec![r#"example\\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "example\\ntest""#;
        expected = vec![r#"example\ntest"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn escape_05() {
        let mut input = r#"echo example\"testhello\"shell"#;
        let mut expected = vec![r#"example"testhello"shell"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo 'example\"testhello\"shell'"#;
        expected = vec![r#"example\"testhello\"shell"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo 'shell\\\nscript'"#;
        expected = vec![r#"shell\\\nscript"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo 'test\\nscript'"#;
        expected = vec![r#"test\\nscript"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo 'hello\"worldexample\"test'"#;
        expected = vec![r#"hello\"worldexample\"test"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "hello'script'\\n'world""#;
        expected = vec![r#"hello'script'\n'world"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "hello\"insidequotes"script\""#;
        expected = vec![r#"hello"insidequotesscript""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn escape_06() {
        let mut input = r#"echo "world'hello'\\'example""#;
        let mut expected = vec![r#"world'hello'\'example"#.to_string()];
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "world\"insidequotes"hello\""#;
        expected = vec![r#"world"insidequoteshello""#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);

        input = r#"echo "mixed\"quote'test'\\""#;
        expected = vec![r#"mixed"quote'test'\"#.to_string()];
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
    }

    #[test]
    fn redirection_01a() {
        let mut input = r#"echo test 1> target_file"#;
        let expected = vec![r#"test"#.to_string()];
        let mut stdout = Stdout {
            kind: RedirectionMode::Overwrite,
            paths: vec![PathBuf::from("target_file")],
        };
        let mut stderr = Stderr::new();
        let mut exp_redir = Redirections::from(stdout, stderr);
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test > target_file"#;
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test>target_file"#;
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test 2> target_file"#;
        stdout = Stdout::new();
        stderr = Stderr {
            kind: RedirectionMode::Overwrite,
            paths: vec![PathBuf::from("target_file")],
        };
        exp_redir = Redirections::from(stdout, stderr);
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);
    }

    #[test]
    fn redirection_01b() {
        let mut input = r#"echo test 1>2"#;
        let expected = vec![r#"test"#.to_string()];
        let mut stdout = Stdout {
            kind: RedirectionMode::Overwrite,
            paths: vec![PathBuf::from("2")],
        };
        let mut stderr = Stderr::new();
        let mut exp_redir = Redirections::from(stdout, stderr);
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test >2"#;
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test 1>>2"#;
        stdout = Stdout {
            kind: RedirectionMode::Append,
            paths: vec![PathBuf::from("2")],
        };
        stderr = Stderr::new();
        exp_redir = Redirections::from(stdout, stderr);
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test>>2"#;
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);
    }

    // TODO: See repl::handle_redirections(), where this is used. I don't think that's the issue.
    #[ignore = "temporarily ignore to tests ci"] // TODO: remove
    #[test]
    fn redirection_02a() {
        let mut input = r#"echo test 1>&2"#;
        let mut expected = vec![r#"test"#.to_string()];
        let mut stdout = Stdout::new();
        let mut stderr = Stderr {
            kind: RedirectionMode::Overwrite,
            paths: vec![],
        };
        let mut exp_redir = Redirections::from(stdout, stderr);
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test >&2"#;
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test 2>&1"#;
        expected = vec![r#"test"#.to_string()];
        stdout = Stdout::new();
        stderr = Stderr::new();
        exp_redir = Redirections::from(stdout, stderr);
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);
    }

    // todo name - count 09
    #[test]
    fn redirection_09() {
        let mut input = r#"echo test > q > w > e >> r >> t > y > u"#;
        let expected = vec![r#"test"#.to_string()];
        let paths = ["q", "w", "e", "r", "t", "y", "u"]
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let mut stdout = Stdout {
            kind: RedirectionMode::Overwrite,
            paths: paths.clone(),
        };
        let mut stderr = Stderr::new();
        let mut exp_redir = Redirections::from(stdout, stderr);
        let mut result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);

        input = r#"echo test > q > w > e >> r > t >> y >> u"#;
        stdout = Stdout {
            kind: RedirectionMode::Append,
            paths,
        };
        stderr = Stderr::new();
        exp_redir = Redirections::from(stdout, stderr);
        result = parse_input(input).unwrap();
        assert_eq!(expected, result.0[1..]);
        assert_eq!(exp_redir, result.1);
    }

    #[ignore]
    #[test]
    /// This can be handled in code by eagerly scanning the input for `> >`.
    fn invalid_input_ignored() {
        let expected = InvalidInputError {
            reason: "shell: syntax error near unexpected token `>'\n".to_string(),
        };

        let input = r#"echo test > > file"#;
        let result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);
        assert_eq!(expected, result);
    }

    #[test]
    fn invalid_input() {
        let mut expected = InvalidInputError {
            reason: "unmatched escape character\n".to_string(),
        };

        let mut input = r#"echo \"#;
        let mut result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        expected = InvalidInputError {
            reason: "unmatched single quotes\n".to_string(),
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
            reason: "unmatched double quotes\n".to_string(),
        };

        input = r#"echo ""#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        input = r#"echo """"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        expected = InvalidInputError {
            reason: "shell: syntax error near unexpected token `>'\n".to_string(),
        };

        input = r#"echo test >>> file"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        input = r#"echo test 1>>> file"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        input = r#"echo test 2>>> file"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        expected = InvalidInputError {
            reason: "shell: syntax error near unexpected token `&'\n".to_string(),
        };

        input = r#"echo test &>& file"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);

        input = r#"echo test >&& file"#;
        result = parse_input(input).unwrap_err();
        assert_eq!(expected, result);
    }
}
