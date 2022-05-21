use clap::{value_t_or_exit, values_t, ArgMatches};
use itertools::Itertools;
use postgres_types::{FromSql, ToSql};
use nonconstant_lcl_classifier_lib::{
    lcl_problem::{Normalizable, Purgeable},
    LclProblem,
};

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "complexity")]
enum Complexity {
    #[postgres(name = "(1)")]
    Constant,
    #[postgres(name = "(log* n)")]
    LogStar,
    #[postgres(name = "(loglog n)")]
    LogLog,
    #[postgres(name = "(log n)")]
    Log,
    #[postgres(name = "(n)")]
    Linear,
    #[postgres(name = "unsolvable")]
    Unsolvable,
}

pub fn fetch_and_print_problems(sub_m: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let active_degree = value_t_or_exit!(sub_m, "active_degree", i16);
    let passive_degree = value_t_or_exit!(sub_m, "passive_degree", i16);
    let label_count = value_t_or_exit!(sub_m, "label_count", i16);
    let db_path = sub_m.value_of("database_path").unwrap();
    let modulo = values_t!(sub_m, "modulo", u16).ok();

    let modulo = modulo.map(|v| (v[0], v[1]));

    let mut problems = fetch_problems(db_path, active_degree, passive_degree, label_count, modulo)
        .unwrap_or_else(|_|
            panic!(
                "Failed to fetch problems from lcl classifier database at {}",
                db_path
            )
        );

    if sub_m.is_present("purge") {
        let old_count = problems.len();
        problems = problems.purge();
        eprintln!("Purging removed {} problems", old_count - problems.len());
    }

    if sub_m.is_present("normalize") {
        let old_count = problems.len();
        problems = problems.normalize();
        eprintln!(
            "Normalizing removed {} problems",
            old_count - problems.len()
        );
    }

    eprintln!("Fetched {} problems", problems.len());
    problems
        .iter()
        .for_each(|p| println!("0: {}", p.to_string()));
    Ok(())
}

/// Fetches all problems with constant determinate lower bound
///
/// The problems are fetched from the given LCL-classifier's database.
///
/// `database_path` should be of form
/// ```"postgresql://<user>:<password>@<host>:<port>"```
///
/// For example
/// ```"postgresql://postgres:pass@localhost/db"```
pub fn fetch_problems(
    database_path: &str,
    active_degree: i16,
    passive_degree: i16,
    label_count: i16,
    modulo: Option<(u16, u16)>,
) -> Result<Vec<LclProblem>, Box<dyn std::error::Error>> {
    use postgres::{Client, NoTls};
    let mut client = Client::connect(database_path, NoTls)?;

    let (remainder, modulus) = modulo.unwrap_or((0, 1));
    assert!(
        remainder < modulus,
        "Remainder ({}) should be less than modulus ({})",
        remainder,
        modulus
    );

    //TODO Make degree and label_count filters optional.

    let query_str = "
    SELECT id, active_degree, passive_degree, label_count, active_constraints, passive_constraints
    FROM problems
    WHERE
        is_tree = TRUE AND
        is_directed_or_rooted = FALSE AND
        det_lower_bound = $1 AND
        active_degree = $2 AND
        passive_degree = $3 AND
        label_count = $4 AND
        id % $5 = $6
    ORDER BY id";
    let query = client.query(
        query_str,
        &[
            &Complexity::Constant,
            &active_degree,
            &passive_degree,
            &label_count,
            &(modulus as i32),
            &(remainder as i32),
        ],
    )?;

    let mut problems = Vec::with_capacity(query.len());

    for row in query {
        let _id: i32 = row.get(0);
        let _active_degree: i16 = row.get(1);
        let _passive_degree: i16 = row.get(2);
        let _label_count: i16 = row.get(3);
        let active_constraints: Vec<String> = row.get(4);
        let passive_constraints: Vec<String> = row.get(5);

        let active_configuration = active_constraints.join(" ");
        let passive_configuration = passive_constraints.join(" ");
        problems.push(
            LclProblem::new(
                active_configuration.as_str(),
                passive_configuration.as_str(),
            )
            .expect("Could not parse an LCL problem from LCL classifier's database"),
        );
    }

    Ok(problems)
}

fn _configuration_string_from_lcl_classifier_format(encoding: &[String]) -> String {
    encoding.iter().map(|x| x.chars().join(" ")).join("\n")
}

#[cfg(test)]
mod tests {
    use postgres::{Client, NoTls};

    #[test]
    #[ignore = "Should be ran manually as db is not quaranteed"]
    fn test_db_connection() -> Result<(), Box<dyn std::error::Error>> {
        Client::connect("postgresql://postgres:pass@localhost/db", NoTls)?;

        Ok(())
    }
}
