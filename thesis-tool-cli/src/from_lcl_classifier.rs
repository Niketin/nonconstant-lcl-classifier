use itertools::Itertools;
use postgres_types::{FromSql, ToSql};
use thesis_tool_lib::LclProblem;

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
    assert!(remainder < modulus, "Remainder ({}) should be less than modulus ({})", remainder, modulus);

    //TODO Make degree and label_count filters optional.

    let query_str = format!("
    SELECT id, active_degree, passive_degree, label_count, active_constraints, passive_constraints
    FROM problems
    WHERE
        is_tree = TRUE AND
        actives_all_same = FALSE AND
        passives_all_same = FALSE AND
        is_directed_or_rooted = FALSE AND
        det_lower_bound = $1 AND
        active_degree = $2 AND
        passive_degree = $3 AND
        label_count = $4 AND
        id % $5 = $6
    ORDER BY id"
    );
    let query = client.query(
        query_str.as_str(),
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
        let active_constraints: Vec<String> = row.get(4); // In lcl-classifier format
        let passive_constraints: Vec<String> = row.get(5); // In lcl-classifier format

        let active_configuration =
            configuration_string_from_lcl_classifier_format(&active_constraints);
        let passive_configuration =
            configuration_string_from_lcl_classifier_format(&passive_constraints);
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

fn configuration_string_from_lcl_classifier_format(encoding: &Vec<String>) -> String {
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
