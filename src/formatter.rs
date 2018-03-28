use measurement::{self, Measurement};
use format::{self, FormattingOptions};

/// Prints out the data gathered by the profiler. Uses
/// [`format::STREAMLINED`](format/static.STREAMLINED.html) as the
/// default format.
///
/// Prints out something like this:
/// ```text
/// ╶──┬╼ main                        - 100.0%, 300 ms/loop
///    ├──┬╼ physics simulation       -  66.7%, 200 ms/loop
///    │  ├───╼ moving things         -  50.0%, 100 ms/loop
///    │  └───╼ resolving collisions  -  50.0%, 100 ms/loop
///    └───╼ rendering                -  33.3%, 100 ms/loop
/// ```
pub fn print() {
    print_with_format(format::STREAMLINED, 0);
}

/// Prints out the data gathered by the profiler using a given format.
///
/// Prints out something like this:
/// ```text
/// ╶──┬╼ main                        - 100.0%, 300 ms/loop
///    ├──┬╼ physics simulation       -  66.7%, 200 ms/loop
///    │  ├───╼ moving things         -  50.0%, 100 ms/loop
///    │  └───╼ resolving collisions  -  50.0%, 100 ms/loop
///    └───╼ rendering                -  33.3%, 100 ms/loop
/// ```
pub fn print_with_format(ops: FormattingOptions, decimals: usize) {
    println!("{}", get_formatted_string(ops, decimals));
}

/// Returns what [`print`](fn.print.html) prints, if you want to put it somewhere else
/// than stdout.
///
/// Which is something like this:
/// ```text
/// ╶──┬╼ main                        - 100.0%, 300 ms/loop
///    ├──┬╼ physics simulation       -  66.7%, 200 ms/loop
///    │  ├───╼ moving things         -  50.0%, 100 ms/loop
///    │  └───╼ resolving collisions  -  50.0%, 100 ms/loop
///    └───╼ rendering                -  33.3%, 100 ms/loop
/// ```
pub fn get_formatted_string(ops: FormattingOptions, decimals: usize) -> String {
    let mut result = String::new();
    let children = measurement::get_measures();

    let construct_tree_branch = |measurement: &Measurement| -> String {
        let has_child;
        let not_last_leaf;
        if let Some(ref parent) = measurement.parent {
            let parent = parent.get_mut();
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
                    ops.ending_branch.chars().count() - ops.continuing_branch.chars().count();
                let mut branch_part = format!("{:width$}", "", width = width);

                let generation = (measurement.depth - 1 - d) as u32;
                if d > 0 && generation >= 1 {
                    if let Some(ancestor) = measurement.get_ancestor(generation) {
                        let ancestor = ancestor.get_mut();
                        let younger_ancestor = measurement.get_ancestor(generation - 1).unwrap();
                        let younger_ancestor = younger_ancestor.get_mut();
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

        let branch = construct_tree_branch(&measurement);
        let info_line;
        if let Some(duration) = measurement.get_duration_ns() {
            let count = measurement.durations.len();

            let parent_duration = if measurement.depth > 1 {
                let parent = measurement.parent.unwrap();
                let parent = parent.get_mut();
                match parent.get_duration_ns() {
                    Some(duration) => duration, // Parent has duration, use it
                    None => duration,           // Parent has no duration, use own
                }
            } else {
                duration // No parent, use own
            };

            if measurement.depth == 1 {
                main_count = count;
            }

            info_line = format!(
                "{:5.1}%, {:width$.decimals$} ms/loop, {} samples",
                100.0 * (duration as f64 / parent_duration as f64),
                (duration / main_count as u64) as f64 / 1_000_000.0,
                count,
                width = decimals + 3,
                decimals = decimals
            );
        } else {
            info_line = String::from("no data");
        }

        result += &format!(
            "{:max_width$} - {}\n",
            branch,
            info_line,
            max_width = max_width
        );
        index += 1;
    }
    result
}
