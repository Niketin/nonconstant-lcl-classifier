use clap::{
    App,
    AppSettings::{self, ArgRequiredElseHelp},
    Arg, SubCommand,
};
use indoc::indoc;

pub fn build_cli() -> App<'static, 'static> {
    let subcommand_find = get_subcommand_find();
    let subcommand_generate = get_subcommand_generate();
    let subcommand_create_cache = get_subcommand_create_sql_cache();
    let subcommand_import_problems_from_lcl_classifier_db = get_subcommand_import_problems_from_lcl_classifier_db();

    App::new("Nonconstant LCL classifier")
        .version("0.3.0")
        .setting(ArgRequiredElseHelp)
        .subcommands([
            subcommand_find,
            subcommand_generate,
            subcommand_create_cache,
            subcommand_import_problems_from_lcl_classifier_db
        ])
        .about("This tool can be used to find nonconstant lower bounds for LCL-problems in the LOCAL model")
        .long_about(indoc! {"
        This tool can be used to find nonconstant lower bounds for LCL-problems in the LOCAL model.

        It tries to find a counterexample multigraph, in which the problem is unsolvable in the PN model.
        This implies that the problem is not solvable in constant time in the LOCAL model.
        "})
}

fn get_subcommand_find() -> App<'static, 'static> {
    let min_nodes = Arg::with_name("min_nodes")
        .index(1)
        .help("Sets the minimum number of nodes for the generated graphs")
        .required(true);
    let max_nodes = Arg::with_name("max_nodes")
        .index(2)
        .help("Sets the maximum number of nodes for the generated graphs")
        .required(true);

    let all_graphs = Arg::with_name("all_graphs")
        .help("Finds all counterexamples at graph loop with fixed node count")
        .long_help(indoc! {"
            Finds all counterexamples at graph loop with fixed node count.

            Basically never breaks the inner-most loop:
                let problem = ...;
                for n in graph_sizes:
                    for graph in graphs_of_size_n: // THIS LOOP
                        find_counterexample(graph, problem);
                        ...
            "})
        .short("A")
        .long("all-graphs")
        .required(false);

    let all_graph_sizes = Arg::with_name("all_graph_sizes")
        .help("Finds all counterexamples with different node counts")
        .long_help(indoc! {"
            Finds all counterexamples with different node counts.

            Basically never breaks the 2nd inner-most loop:
                let problem = ...;
                for n in graph_sizes: // THIS LOOP
                    for graph in graphs_of_size_n:
                        find_counterexample(graph, problem);
                        ...
            "})
        .short("a")
        .long("all-graph-sizes")
        .required(false);

    let progress = Arg::with_name("progress")
        .help("Shows progress")
        .long_help(indoc! {"
            Shows progress.

            Using this flag multiple times (-pp) is not recommended as it can decrese the performance due to additional printing.
            "})
        .short("p")
        .long("show-progress")
        .multiple(true);

    let output_svg = Arg::with_name("output_svg")
        .help("Output counterexample graphs as svg to the given directory")
        .long("svg-dir")
        .takes_value(true);

    let print_stats = Arg::with_name("print_stats")
        .long("stats")
        .help("Prints different stats of results after finding them");

    let write_nonproven_results = Arg::with_name("write_nonproven_result")
        .help("Path where nonproven results will be written")
        .takes_value(true)
        .value_name("path_to_nonproven_results")
        .short("o")
        .long("write-nonproven");

    let sqlite_cache = Arg::with_name("sqlite_cache")
        .help("Path to an sqlite database that will be used as a cache")
        .long_help(indoc! {"
            Path to an sqlite database that will be used as a cache.

            This means that if the intermediate values already exist in the database,
            they are retrieved from there.
        "})
        .takes_value(true)
        .value_name("path")
        .short("c")
        .long("sqlite-cache");

    let subcommand_single = get_subcommand_single();
    let subcommand_class = get_subcommand_class();
    let subcommand_file = get_subcommand_from_stdin();

    SubCommand::with_name("find")
        .setting(AppSettings::SubcommandRequired)
        .about("Finds lower bound proofs for LCL-problems")
        .long_about(indoc! {"
        Find lower bound proofs for LCL-problems.

        This command generates bipartite multigraphs of size min_nodes..max_nodes.
        Then it checks for each problem, if there exists a graph that cannot be labeled within the constraints of the problem.
        TODO write about SAT problem
        "})
        .args(&[
            min_nodes,
            max_nodes,
            progress,
            all_graphs,
            all_graph_sizes,
            output_svg,
            print_stats,
            sqlite_cache,
            write_nonproven_results,
        ])
        .subcommands([subcommand_single, subcommand_class, subcommand_file])
}

fn get_subcommand_class() -> App<'static, 'static> {
    let active_degree = Arg::with_name("active_degree")
        .help("Degree of the active partition")
        .takes_value(true)
        .required(true);
    let passive_degree = Arg::with_name("passive_degree")
        .help("Degree of the passive partition")
        .takes_value(true)
        .required(true);
    let label_count = Arg::with_name("label_count")
        .help("Count of the labels used in the problems")
        .takes_value(true)
        .required(true);
    SubCommand::with_name("class")
        .about("Runs for a class of problems")
        .long_about(indoc! {"
            Runs for a class of problems.

            Class is defined by degree of active partition, degree of passive partition and label count.
            Each problem in the class will be generated."})
        .args(&[active_degree, passive_degree, label_count])
}

fn get_subcommand_single() -> App<'static, 'static> {
    let active_configurations = Arg::with_name("active_configurations")
        .short("A")
        .help("Sets the active configurations of the LCL-problem")
        .takes_value(true)
        .min_values(1)
        .max_values(1)
        .required(true);
    let passive_configurations = Arg::with_name("passive_configurations")
        .short("P")
        .help("Sets the passive configurations of the LCL-problem")
        .takes_value(true)
        .min_values(1)
        .max_values(1)
        .required(true);
    SubCommand::with_name("single")
        .about("Runs for a single problem")
        .args(&[active_configurations, passive_configurations])
}

fn get_subcommand_import_problems_from_lcl_classifier_db() -> App<'static, 'static> {
    let db_path = Arg::with_name("database_path")
        .help("Path to an PostgreSQL database used by the LCL-classifier")
        .long_help(indoc! {"
            Path to an PostgreSQL database used by the LCL-classifier.

            This is the database containing all the problems we are fetching.
        "})
        .value_name("database_path")
        .required(true);
    let active_degree = Arg::with_name("active_degree")
        .help("Degree of the active partition")
        .takes_value(true)
        .required(true);
    let passive_degree = Arg::with_name("passive_degree")
        .help("Degree of the passive partition")
        .takes_value(true)
        .required(true);
    let label_count = Arg::with_name("label_count")
        .help("Count of the labels used in the problems")
        .takes_value(true)
        .required(true);
    let modulo = Arg::with_name("modulo")
        .help("Only find subset of results")
        .long("mod")
        .takes_value(true)
        .min_values(2)
        .max_values(2)
        .value_names(&["reminder", "modulus"]);
    let purge = Arg::with_name("purge")
        .short("p")
        .long("purge")
        .help("Removes redundant problems");
    let normalize = Arg::with_name("normalize")
        .short("n")
        .long("normalize")
        .help("Normalizes problems");
    SubCommand::with_name("fetch_problems")
        .about("Fetch problems from LCL-classifier's database")
        .long_about(indoc! {"
            Fetch problems from LCL-classifier's database.

            Queries all problems that have constant lowerbound
            i.e. all problems for which we can possibly improve the lowerbound.
            For each problem, it tries to find a lowerbound of \"non-constant\".

            Problems are outputed to stdout.
        "})
        .args(&[
            purge,
            normalize,
            active_degree,
            passive_degree,
            label_count,
            modulo,
            db_path,
        ])
}

fn get_subcommand_from_stdin() -> App<'static, 'static> {
    let no_ignore = Arg::with_name("no_ignore")
        .short("n")
        .long("no-ignore")
        .help("Do not ignore problems with counterexamples");
    SubCommand::with_name("from_stdin")
        .about("Read problems from stdin")
        .long_about(indoc! {"
        Read problems from stdin.

        Problems have to be from same problem class.

        By defualt, uses only the problems that have no counter example yet, i.e. <graph_size> is 0.

        File should have one problem on each line.
        Each problem should be in the following format:
            [<graph_size>]: <problem>
        For example:
            2: ABC CCC; AA BB BC
            0: AB CC; AA BB BC
        A positive <graph_size> notates that there is a counter example of size <graph_size>.
        problem that has <graph_size> of 0 has no counter example yet.

        The '<problem>' part is of form:
            <configuration> ...; <configuration> ...
        For example:
            ABC CCC; AA BB BC
    "})
        .args(&[no_ignore])
}

fn get_subcommand_generate() -> App<'static, 'static> {
    let subcommand_problems = get_subcommand_problems();
    let subcommand_graphs = get_subcommand_graphs();
    SubCommand::with_name("gen")
        .about("Generate <subcommand> and save into file system")
        .setting(ArgRequiredElseHelp)
        .subcommands([subcommand_graphs, subcommand_problems])
}

fn get_subcommand_problems() -> App<'static, 'static> {
    let active_degree = Arg::with_name("active_degree")
        .help("Degree of the active partition")
        .takes_value(true)
        .required(true);
    let passive_degree = Arg::with_name("passive_degree")
        .help("Degree of the passive partition")
        .takes_value(true)
        .required(true);
    let label_count = Arg::with_name("label_count")
        .help("Count of the labels used in the problems")
        .takes_value(true)
        .required(true);
    let sqlite_cache = Arg::with_name("sqlite_cache")
        .help("Path to an sqlite database that will be used as an LCL problem cache")
        .long_help(indoc! {"
        Path to an sqlite database that will be used as an LCL problem cache.

        This means that if the class of LCL problems already exist in the database,
        the problems are retrieved from there."})
        .takes_value(true)
        .value_name("path")
        .short("c")
        .long("sqlite-cache");

    SubCommand::with_name("problems")
        .about("Generate LCL problems")
        .args(&[active_degree, passive_degree, label_count, sqlite_cache])
}

fn get_subcommand_graphs() -> App<'static, 'static> {
    let min_nodes = Arg::with_name("min_nodes")
        .help("Sets the minimum number of nodes for the generated graphs")
        .required(true);
    let max_nodes = Arg::with_name("max_nodes")
        .help("Sets the maximum number of nodes for the generated graphs")
        .required(true);
    let active_degree = Arg::with_name("active_degree")
        .help("Degree of the active partition")
        .required(true);
    let passive_degree = Arg::with_name("passive_degree")
        .help("Degree of the passive partition")
        .required(true);
    let sqlite_cache = Arg::with_name("sqlite_cache")
        .help("Path to an sqlite database that will be used as a graph cache")
        .long_help(indoc! {"
            Path to an sqlite database that will be used as a graph cache.

            This means that if the graphs already exist in the database,
            the graphs are retrieved from there.
        "})
        .takes_value(true)
        .value_name("path")
        .short("c")
        .long("sqlite-cache");
    SubCommand::with_name("graphs")
        .about("Generate biregular multigraphs and save into file system")
        .args(&[
            min_nodes,
            max_nodes,
            active_degree,
            passive_degree,
            sqlite_cache,
        ])
}

fn get_subcommand_create_sql_cache() -> App<'static, 'static> {
    let sqlite_cache = Arg::with_name("sqlite_cache")
        .help("Path to a new SQLite database")
        .long_help(indoc! {"
            Path to a new SQLite database that will be used as a cache.

            This means that if the graphs/problems already exist in the database,
            they can be retrieved from there.
        "})
        .takes_value(true)
        .value_name("path")
        .required(true);
    SubCommand::with_name("create_cache")
        .about("Generate SQLite database for caching")
        .args(&[sqlite_cache])
}
