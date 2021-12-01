use clap::{App, AppSettings, Arg, SubCommand};
use indoc::indoc;

pub fn build_cli() -> App<'static, 'static> {
    let min_nodes = Arg::with_name("min_nodes")
        .index(1)
        .help("Sets the maximum number of nodes for the generated graphs.")
        .required(true);
    let max_nodes = Arg::with_name("max_nodes")
        .index(2)
        .help("Sets the maximum number of nodes for the generated graphs.")
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

    let all = Arg::with_name("all")
        .help("Find all results.")
        .short("a")
        .long("all")
        .required(false);

    let simple_graphs_only = Arg::with_name("simple_graphs_only")
        .help("Generate only simple graphs.")
        .short("s")
        .long("simple-graphs-only")
        .required(false);

    let progress = Arg::with_name("progress")
        .help("Show progress.")
        .short("p")
        .long("show-progress")
        .multiple(true);

    let output_svg = Arg::with_name("output_svg")
        .help("If a lower-bound proof is found, output graph as svg to the path.")
        .long("svg")
        .takes_value(true);

    let verbosity = Arg::with_name("verbosity")
        .short("v")
        .help("Sets the level of verbosity")
        .multiple(true);

    let subcommand_single = SubCommand::with_name("single")
        .about("Run for a single problem")
        .args(&[active_configurations, passive_configurations]);
    let subcommand_class = SubCommand::with_name("class")
        .about("Run for a class of problems.")
        .long_about(indoc! {"
            Run for a class of problems.

            Class is defined by degree of active partition, degree of passive partition and label count.
            Each problem in the class will be generated."})
        .arg(problem_class);

    let subcommand_find = SubCommand::with_name("find")
        .setting(AppSettings::SubcommandRequired)
        .about("Find an unsolvable pair of graph and problem.")
        .args(&[
            min_nodes,
            max_nodes,
            progress,
            all,
            simple_graphs_only,
            output_svg,
            verbosity,
        ])
        .subcommands([subcommand_single, subcommand_class]);

    App::new("Thesis tool")
        .version("0.3.0")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(subcommand_find)
        .about("This tool can be used to find lower-bound proofs for LCL-problems.")
        .long_about(indoc! {"
        This tool can be used to find lower-bound proofs for LCL-problems.

        TODO
        "})
}
