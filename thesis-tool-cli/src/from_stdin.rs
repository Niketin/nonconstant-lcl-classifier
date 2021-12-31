use itertools::Itertools;
use std::io::{self, BufRead};
use thesis_tool_lib::LclProblem;


/// Read LCL problems from stdin.
///
/// Stream is expected to have one problem per line, each problem ending in newline.
/// Ignores all problems that have a positive `n`.
///
/// Problem format:
/// <n>: <active_configuration>; <passive_configuration>
///
/// Example:
/// ```ignore
/// 2: AA AB BC CC; AC BB
/// 0: AA AB AC BB CC; AA AB AC BB BC CC
/// ```
pub fn from_stdin() -> Result<Vec<LclProblem>, Box<dyn std::error::Error>> {

    let stdin = io::stdin();
    let lines = stdin.lock().lines();

    Ok(lines
        .filter_map(|line_res| {
            let line = line_res.expect("Could not read line");

            let (n_str, problem_str) = line
                .split(':')
                .map(|x| x.trim())
                .collect_tuple()
                .expect("Line was not in correct format");
            let n: usize = n_str.parse().expect("Graph size was not an integer");

            if n > 0 {
                return None;
            }

            let (active, passive) = problem_str
                .split(";")
                .map(|x| x.trim())
                .collect_tuple()
                .expect("Problem was not in correct format");
            let problem =
                LclProblem::new(active, passive).expect("Could not parse the LCL problem");
            return Some(problem);
        })
        .collect_vec())
}