//! Formats for making the formatted outputs (see
//! [`get_formatted_string`](../fn.get_formatted_string.html))

/// A very streamlined format. This is the default format.
///
/// ```text
/// ╶──┬╼ main                        - 100.0%, 300 ms/loop
///    ├──┬╼ physics simulation       -  66.7%, 200 ms/loop
///    │  ├───╼ moving things         -  50.0%, 100 ms/loop
///    │  └───╼ resolving collisions  -  50.0%, 100 ms/loop
///    └───╼ rendering                -  33.3%, 100 ms/loop
/// ```
pub static STREAMLINED: FormattingOptions = FormattingOptions {
    starting_branch: "╶",
    continuing_branch: "│",
    branching_branch: "├",
    turning_branch: "└",
    ending_branch: "───╼",
    turning_ending_branch: "──┬╼",
};

/// Like `STREAMLINED` except with rounded corners.
///
/// ```text
/// ╶──┬╼ main                        - 100.0%, 300 ms/loop
///    ├──┬╼ physics simulation       -  66.7%, 200 ms/loop
///    │  ├───╼ moving things         -  50.0%, 100 ms/loop
///    │  ╰───╼ resolving collisions  -  50.0%, 100 ms/loop
///    ╰───╼ rendering                -  33.3%, 100 ms/loop
/// ```
pub static STREAMLINED_ROUNDED: FormattingOptions = FormattingOptions {
    starting_branch: "╶",
    continuing_branch: "│",
    branching_branch: "├",
    turning_branch: "╰",
    ending_branch: "───╼",
    turning_ending_branch: "──┬╼",
};

/// A format made out of -'s and |'s. Very compatible with small charsets!
///
/// ```text
/// ----- main                        - 100.0%, 300 ms/loop
///    |---- physics simulation       -  66.7%, 200 ms/loop
///    |  |---- moving things         -  50.0%, 100 ms/loop
///    |  \---- resolving collisions  -  50.0%, 100 ms/loop
///    \---- rendering                -  33.3%, 100 ms/loop
/// ```
pub static COMPATIBLE: FormattingOptions = FormattingOptions {
    starting_branch: "-",
    continuing_branch: "|",
    branching_branch: "|",
    turning_branch: "\\",
    ending_branch: "----",
    turning_ending_branch: "----",
};

/// This format is for those who like their lines doubled.
///
/// ```text
/// ═══╦═ main                        - 100.0%, 300 ms/loop
///    ╠══╦═ physics simulation       -  66.7%, 200 ms/loop
///    ║  ╠════ moving things         -  50.0%, 100 ms/loop
///    ║  ╚════ resolving collisions  -  50.0%, 100 ms/loop
///    ╚════ rendering                -  33.3%, 100 ms/loop
/// ```
pub static DOUBLED: FormattingOptions = FormattingOptions {
    starting_branch: "═",
    continuing_branch: "║",
    branching_branch: "╠",
    turning_branch: "╚",
    ending_branch: "════",
    turning_ending_branch: "══╦═",
};

/// This format is for debugging the formatting functionality.
///
/// ```text
/// >,,,, main                        - 100.0%, 300 ms/loop
///    +,,,, physics simulation       -  66.7%, 200 ms/loop
///    |  +.... moving things         -  50.0%, 100 ms/loop
///    |  -.... resolving collisions  -  50.0%, 100 ms/loop
///    -.... rendering                -  33.3%, 100 ms/loop
/// ```
pub static DEBUGGING: FormattingOptions = FormattingOptions {
    starting_branch: ">",
    continuing_branch: "|",
    branching_branch: "+",
    turning_branch: "-",
    ending_branch: "....",
    turning_ending_branch: ",,,,",
};

/// Defines the parts which are used to print out the formatted
/// string.
///
/// See the [`format`](format/index.html) module for options. You
/// can make your own, if you can parse the sparse instructions
/// below.
///
/// # Reference print (see Fields)
/// ```text
/// >,,,, main                      - 100.0%, 300 ms/loop
///   +,,,, physics simulation      -  66.7%, 200 ms/loop
///   | +.... moving things         -  50.0%, 100 ms/loop
///   | -.... resolving collisions  -  50.0%, 100 ms/loop
///   -.... rendering               -  33.3%, 100 ms/loop
/// ```
#[derive(Clone, Copy)]
pub struct FormattingOptions {
    /// See the reference-print, `starting_branch` is represented by ">"
    pub starting_branch: &'static str,
    /// See the reference-print, `continuing_branch` is represented by "|"
    pub continuing_branch: &'static str,
    /// See the reference-print, `branching_branch` is represented by "+"
    pub branching_branch: &'static str,
    /// See the reference-print, `turning_branch` is represented by "-"
    pub turning_branch: &'static str,
    /// See the reference-print, `ending_branch` is represented by "...."
    pub ending_branch: &'static str,
    /// See the reference-print, `turning_ending_branch` is represented by ",,,,"
    pub turning_ending_branch: &'static str,
}
