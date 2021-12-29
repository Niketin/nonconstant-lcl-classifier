use clap::{
    App,
    AppSettings::{self, ArgRequiredElseHelp},
    Arg, SubCommand,
};
use indoc::indoc;

pub fn build_cli() -> App<'static, 'static> {
    let subcommand_find = get_subcommand_find();
    let subcommand_generate = get_subcommand_generate();
    let subcommand_create_cache = get_subcommand_create_sql_cache(); //TODO add the functionality behind this command

    App::new("Thesis tool")
        .version("0.3.0")
        .setting(ArgRequiredElseHelp)
        .subcommands([
            subcommand_find,
            subcommand_generate,
            subcommand_create_cache,
        ])
        .about("This tool can be used to find lower bound proofs for LCL-problems")
        .long_about(indoc! {"
        This tool can be used to find lower bound proofs for LCL-problems.

        TODO
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

    let all = Arg::with_name("all")
        .help("Finds not only the first, but all lower bound proofs")
        .short("a")
        .long("all")
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
        .help("If a lower bound proof is found, output graph as svg to the path")
        .long("svg")
        .takes_value(true);

    let verbosity = Arg::with_name("verbosity")
        .short("v")
        .help("Sets the level of verbosity")
        .multiple(true);

    let print_stats = Arg::with_name("print_stats")
        .long("stats")
        .help("Prints different stats of results after finding them");

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
    let subcommand_fetch_from_lcl_classifier_db = get_subcommand_fetch_from_lcl_classifier_db();

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
            all,
            output_svg,
            verbosity,
            print_stats,
            sqlite_cache,
        ])
        .subcommands([subcommand_single, subcommand_class, subcommand_fetch_from_lcl_classifier_db])
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
        .required(true);
    let passive_configurations = Arg::with_name("passive_configurations")
        .short("P")
        .help("Sets the passive configurations of the LCL-problem")
        .takes_value(true)
        .min_values(1)
        .required(true);
    SubCommand::with_name("single")
        .about("Runs for a single problem")
        .args(&[active_configurations, passive_configurations])
}

fn get_subcommand_fetch_from_lcl_classifier_db() -> App<'static, 'static> {
    let db_path = Arg::with_name("database_path")
        .help("Path to an PostgreSQL database used by the LCL-classifier")
        .long_help(indoc! {"
            Path to an PostgreSQL database used by the LCL-classifier.

            This is the database containing all the problems we are fetching.
        "})
        .value_name("database_path")
        .required(true);
    let purge = Arg::with_name("purge")
        .short("p")
        .long("purge")
        .help("Removes redundant problems");
    let normalize = Arg::with_name("normalize")
        .short("n")
        .long("normalize")
        .help("Normalizes problems");
    SubCommand::with_name("from_classifier")
        .about("Fetch problems from LCL-classifier's database")
        .long_about(indoc! {"
            Fetch problems from LCL-classifier's database.

            Queries all problems that have unknown lowerbound
            i.e. all problems for which we can possibly improve the lowerbound.
            For each problem, it tries to find a lowerbound of \"non-constant\"
        "})
        .args(&[purge, normalize, db_path])
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
