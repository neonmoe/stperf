use measurement::{self, Measurement};

/// A very streamlined format. TODO: Add example
pub static FORMAT_STREAMLINED: FormattingOptions = FormattingOptions {
    starting_branch: "╶",
    continuing_branch: "│",
    branching_branch: "├",
    turning_branch: "└",
    ending_branch: "─╼",
    turning_ending_branch: "─┮",
};

/// Like `FORMAT_STREAMLINED` except with rounded corners. This is the default format. TODO: Add example
pub static FORMAT_STREAMLINED_ROUNDED: FormattingOptions = FormattingOptions {
    starting_branch: "╶",
    continuing_branch: "│",
    branching_branch: "├",
    turning_branch: "╰",
    ending_branch: "─╼",
    turning_ending_branch: "──┬╼",
};

/// This format is for those who like their lines doubled. TODO: Add example
pub static FORMAT_DOUBLED: FormattingOptions = FormattingOptions {
    starting_branch: "═",
    continuing_branch: "║",
    branching_branch: "╠",
    turning_branch: "╚",
    ending_branch: "══",
    turning_ending_branch: "═╦",
};

/// Defines the parts which are used to print out the formatted string
#[derive(Clone, Copy)]
pub struct FormattingOptions {
    starting_branch: &'static str,
    continuing_branch: &'static str,
    branching_branch: &'static str,
    turning_branch: &'static str,
    ending_branch: &'static str,
    turning_ending_branch: &'static str,
}

/// Prints out the data gathered by the profiler. Uses
/// `FORMAT_STREAMLINED_ROUNDED` as the default format.
pub fn print() {
    print_with_format(FORMAT_STREAMLINED_ROUNDED);
}

/// Prints out the data gathered by the profiler using a given format.
pub fn print_with_format(ops: FormattingOptions) {
    println!("{}", get_formatted_string(ops));
}

/// Returns what [`print`](fn.print.html) prints, if you want to put it somewhere else
/// than stdout.
pub fn get_formatted_string(ops: FormattingOptions) -> String {
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
                let mut branch_part = String::from("   ");

                let generation = (measurement.depth - 1 - d) as u32;
                if d > 0 && generation >= 1 {
                    if let Some(ancestor) = measurement.get_ancestor(generation) {
                        let ancestor = ancestor.lock().unwrap();
                        let younger_ancestor = measurement.get_ancestor(generation - 1).unwrap();
                        let younger_ancestor = younger_ancestor.lock().unwrap();
                        if !ancestor.is_last_child_name(&younger_ancestor.name, false) {
                            branch_part = format!("{}  ", ops.continuing_branch);
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
                "{:6.2}%, {:10.6} ms/loop",
                100.0 * (duration as f64 / parent_duration as f64),
                (duration * count as u32 / main_count as u32) as f64 / 1_000_000.0,
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
