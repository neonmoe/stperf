use measurement::{self, Measurement};

/// Formats for making the formatted outputs (see
/// [`get_formatted_string`](fn.get_formatted_string.html))
pub mod format {
    use super::FormattingOptions;

    /// A very streamlined format. This is the default format.
    ///
    /// ```text
    /// ╶──┬╼ main                - 100.0%, 300 ms/loop
    ///    ├──┬╼ inner thing      -  66.7%, 200 ms/loop
    ///    │  ├─╼ innest thing a  -  50.0%, 100 ms/loop
    ///    │  └─╼ innest thing b  -  50.0%, 100 ms/loop
    ///    └─╼ inner thing 2      -  33.3%, 100 ms/loop
    /// ```
    pub static STREAMLINED: FormattingOptions = FormattingOptions {
        starting_branch: "╶",
        continuing_branch: "│",
        branching_branch: "├",
        turning_branch: "└",
        ending_branch: "─╼",
        turning_ending_branch: "──┬╼",
    };

    /// Like `STREAMLINED` except with rounded corners.
    ///
    /// ```text
    /// ╶──┬╼ main                - 100.0%, 300 ms/loop
    ///    ├──┬╼ inner thing      -  66.7%, 200 ms/loop
    ///    │  ├─╼ innest thing a  -  50.0%, 100 ms/loop
    ///    │  ╰─╼ innest thing b  -  50.0%, 100 ms/loop
    ///    ╰─╼ inner thing 2      -  33.3%, 100 ms/loop
    /// ```
    pub static STREAMLINED_ROUNDED: FormattingOptions = FormattingOptions {
        starting_branch: "╶",
        continuing_branch: "│",
        branching_branch: "├",
        turning_branch: "╰",
        ending_branch: "─╼",
        turning_ending_branch: "──┬╼",
    };

    /// A format made out of +'s, -'s and |'s. Very compatible with small charsets!
    ///
    /// ```text
    /// ---+- main                - 100.0%, 300 ms/loop
    ///    +--+- inner thing      -  66.7%, 200 ms/loop
    ///    |  +-- innest thing a  -  50.0%, 100 ms/loop
    ///    |  +-- innest thing b  -  50.0%, 100 ms/loop
    ///    +-- inner thing 2      -  33.3%, 100 ms/loop
    /// ```
    pub static COMPATIBLE: FormattingOptions = FormattingOptions {
        starting_branch: "-",
        continuing_branch: "|",
        branching_branch: "+",
        turning_branch: "+",
        ending_branch: "--",
        turning_ending_branch: "--+-",
    };

    /// This format is for those who like their lines doubled.
    ///
    /// ```text
    /// ═══╦═ main                 - 100.0%, 300 ms/loop
    ///    ╠══╦═ inner thing       -  66.7%, 200 ms/loop
    ///    ║  ╠═══ innest thing a  -  50.0%, 100 ms/loop
    ///    ║  ╚═══ innest thing b  -  50.0%, 100 ms/loop
    ///    ╚═══ inner thing 2      -  33.3%, 100 ms/loop
    /// ```
    pub static DOUBLED: FormattingOptions = FormattingOptions {
        starting_branch: "═",
        continuing_branch: "║",
        branching_branch: "╠",
        turning_branch: "╚",
        ending_branch: "═══",
        turning_ending_branch: "══╦═",
    };

    /// This format is for debugging the formatting functionality.
    ///
    /// ```text
    /// >,,,, main                 - 100.0%, 300 ms/loop
    ///    +,,,, inner thing       -  66.7%, 200 ms/loop
    ///    |  +... innest thing a  -  50.0%, 100 ms/loop
    ///    |  -... innest thing b  -  50.0%, 100 ms/loop
    ///    -... inner thing 2      -  33.3%, 100 ms/loop
    /// ```
    pub static DEBUGGING: FormattingOptions = FormattingOptions {
        starting_branch: ">",
        continuing_branch: "|",
        branching_branch: "+",
        turning_branch: "-",
        ending_branch: "...",
        turning_ending_branch: ",,,,",
    };

}

/// Defines the parts which are used to print out the formatted string. See the [`format`](format/index.html) module for options. You can make your own, if you can parse the sparse instructions below.
///
/// # Reference print (see Fields)
/// ```text
/// >,,,, main                 - 100.0%, 300 ms/loop
///    +,,,, inner thing       -  66.7%, 200 ms/loop
///    |  +... innest thing a  -  50.0%, 100 ms/loop
///    |  -... innest thing b  -  50.0%, 100 ms/loop
///    -... inner thing 2      -  33.3%, 100 ms/loop
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
    /// See the reference-print, `ending_branch` is represented by "..."
    pub ending_branch: &'static str,
    /// See the reference-print, `turning_ending_branch` is represented by ",,,,"
    pub turning_ending_branch: &'static str,
}

/// Prints out the data gathered by the profiler. Uses
/// `FORMAT_STREAMLINED_ROUNDED` as the default format.
pub fn print() {
    print_with_format(format::STREAMLINED, 0);
}

/// Prints out the data gathered by the profiler using a given format.
pub fn print_with_format(ops: FormattingOptions, decimals: usize) {
    println!("{}", get_formatted_string(ops, decimals));
}

/// Returns what [`print`](fn.print.html) prints, if you want to put it somewhere else
/// than stdout.
pub fn get_formatted_string(ops: FormattingOptions, decimals: usize) -> String {
    let mut result = String::new();
    let children = measurement::get_measures();

    let construct_tree_branch = |measurement: &Measurement| -> String {
        let has_child;
        let not_last_leaf;
        if let Some(ref parent) = measurement.parent {
            let parent = parent.lock().unwrap();
            has_child = measurement.has_children();
            not_last_leaf = !parent.is_last_child_name(&measurement.name, true);
        } else {
            has_child = false;
            not_last_leaf = false;
        }

        let mut branch = String::new();
        for d in 0..measurement.depth {
            if d == measurement.depth - 1 {
                branch += if measurement.depth == 1 {
                    ops.starting_branch
                } else if not_last_leaf {
                    ops.branching_branch
                } else {
                    ops.turning_branch
                }
            } else {
                let width =
                    ops.ending_branch.chars().count() + ops.continuing_branch.chars().count();
                let mut branch_part = format!("{:width$}", "", width = width);

                let generation = (measurement.depth - 1 - d) as u32;
                if d > 0 && generation >= 1 {
                    if let Some(ancestor) = measurement.get_ancestor(generation) {
                        let ancestor = ancestor.lock().unwrap();
                        let younger_ancestor = measurement.get_ancestor(generation - 1).unwrap();
                        let younger_ancestor = younger_ancestor.lock().unwrap();
                        if !ancestor.is_last_child_name(&younger_ancestor.name, false) {
                            branch_part =
                                format!("{:width$}", ops.continuing_branch, width = width);
                        }
                    }
                }

                branch += &branch_part
            }
        }

        branch += if has_child {
            ops.turning_ending_branch
        } else {
            ops.ending_branch
        };
        branch += " ";
        branch += &measurement.name;
        branch
    };

    let mut max_width = 0;
    for measurement in &children {
        let width = construct_tree_branch(&measurement).chars().count() + 1;
        if width > max_width {
            max_width = width;
        }
    }

    let mut index = 0;
    let mut main_count = 1;
    for measurement in children {
        if index == 0 {
            // Skip "root"
            index = 1;
            continue;
        }

        if let Some(duration) = measurement.get_avg_duration_ns() {
            let branch = construct_tree_branch(&measurement);
            let parent_duration = if measurement.depth > 1 {
                let parent = measurement.parent.unwrap();
                let parent = parent.lock().unwrap();
                match parent.get_avg_duration_ns() {
                    Some(duration) => duration, // Parent has duration, use it
                    None => duration,           // Parent has no duration, use own
                }
            } else {
                duration // No parent, use own
            };

            let count = measurement.durations.len();
            if measurement.depth == 1 {
                main_count = count;
            }

            let info_line = &format!(
                "{:5.1}%, {:width$.decimals$} ms/loop",
                100.0 * (duration as f64 / parent_duration as f64),
                (duration * count as u32 / main_count as u32) as f64 / 1_000_000.0,
                width = decimals + 3,
                decimals = decimals
            );

            result += &format!(
                "{:max_width$} - {}\n",
                branch,
                info_line,
                max_width = max_width
            );
        } else {
            result += &format!(" {}\n", measurement.name);
        }
        index += 1;
    }
    result
}
