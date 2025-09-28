//! Macros Used Throughout the Library

/// Evaluates the global variable `TEST` at compile-time
/// and replaces `true` with `break` and `false` with `continue`.
///
/// This can be used to build the project for testing from a shell script
/// or for the final use in production - for a release.
///
/// It's meant to be used in the main (repl) loop.
#[macro_export]
macro_rules! test_to_break_or_continue {
    () => {
        match TEST.get().is_some_and(|&test| test) {
            true => break,
            false => continue,
        }
    };
}

// TODO: Ideally, we only evaluate TEST once! I don't know if it can be improved more.
// ORIGINAL:
// macro_rules! test_to_break_or_continue {
//     ($cond:expr) => {
//         if $cond {
//             break;
//         } else {
//             continue;
//         }
//     };
// }
