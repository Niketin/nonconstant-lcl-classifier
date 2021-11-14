use clap::{App, AppSettings, Arg, SubCommand};
use indoc::indoc;

pub fn build_cli() -> App<'static, 'static> {
    let graph_size_bound = Arg::with_name("graph_size_bound")
        .long("graph-sizes")
        .short("n")
        .takes_value(true)
        .number_of_values(2)
        .value_names(&["lower_bound", "upper_bound"])
        .help("Set bounds for graph sizes. The range is inclusive.")
        .required(true);

    let active_configurations = Arg::with_name("active_configurations")
        .short("A")
        .help("Sets the active configurations of the LCL-problem.")
        .takes_value(true)
        .min_values(1)
        .required(true);

    let passive_configurations = Arg::with_name("passive_configurations")
        .short("P")
        .help("Sets the passive configurations of the LCL-problem.")
        .takes_value(true)
        .min_values(1)
        .required(true);

    let problem_class = Arg::with_name("problem_class")
        .help(indoc! {"
            active_degree - degree of the active partition.
            passive_degree - degree of the passive partition.
            label_count - count of the labels used in the problems.
        "})
        .takes_value(true)
        .min_values(3)
        .max_values(3)
        .value_names(&["active_degree", "passive_degree", "label_count"])
        .required(true);

    let simple_graphs_only = Arg::with_name("simple_graphs_only")
        .help("Generate only simple graphs.")
        .short("s")
        .long("simple-graphs-only")
        .required(false);

    let progress = Arg::with_name("progress")
        .help("Show progress.")
        .short("p")
        .long("show-progress")
        .required(false);

    let output_svg = Arg::with_name("output_svg")
        .help("If unsatisfiable result is found, output graph as svg to the path.")
        .long("svg")
        .takes_value(true);

    let subcommand_single = SubCommand::with_name("single")
        .about("Run for a single problem")
        .args(&[active_configurations, passive_configurations]);
    let subcommand_class = SubCommand::with_name("class")
        .about("Run for a class of problems.")
        .long_about(indoc!{"
            Run for a class of problems.
            
            Class is defined by degree of active partition, degree of passive partition and label count.
            Each problem in the class will be generated."})
        .arg(problem_class);

    let subcommand_find = SubCommand::with_name("find")
        .setting(AppSettings::SubcommandRequired)
        .about("Find an unsolvable pair of graph and problem.")
        .args(&[graph_size_bound, progress, simple_graphs_only, output_svg])
        .subcommands([subcommand_single, subcommand_class]);

    App::new("Thesis tool")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(subcommand_find)
        .about("This tool can be used to find negative proofs of LCL-problems solvability on the Port Numbering model. TODO")
}
