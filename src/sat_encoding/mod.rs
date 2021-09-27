use crate::lcl_problem::LclProblem;
use crate::BiregularGraph;
use itertools::Itertools;

type Clause = Vec<i32>;
type Clauses = Vec<Clause>;
type Permutations = Vec<Vec<u8>>;

pub struct SatEncoder {
    lcl_problem: LclProblem,
    graph: BiregularGraph,
    active_permutations: Permutations,
    passive_permutations: Permutations,
}

impl SatEncoder {
    pub fn new(lcl_problem: LclProblem, graph: BiregularGraph) -> SatEncoder {
        let active_configurations_iter = &lcl_problem
            .active
            .get_configurations();
        let active_configurations = active_configurations_iter
            .into_iter()
            .map(|x| x.collect_vec());

        let passive_configurations_iter = &lcl_problem
            .passive
            .get_configurations();
        let passive_configurations = passive_configurations_iter
            .into_iter()
            .map(|x| x.collect_vec());

        let active_permutations = active_configurations
            .map(|x| {
                let k = x.len();
                x.iter().map(|x| **x).permutations(k).unique().collect_vec()
            })
            .flatten()
            .collect_vec();

        let passive_permutations = passive_configurations
            .map(|x| {
                let k = x.len();
                x.iter().map(|x| **x).permutations(k).unique().collect_vec()
            })
            .flatten()
            .collect_vec();

        SatEncoder {
            lcl_problem,
            graph,
            active_permutations,
            passive_permutations,
        }
    }

    pub fn encode(&self) -> Clauses {
        let mut clauses: Clauses = vec![];

        let active_permutations_len: usize = self.active_permutations.len();
        let passive_permutations_len: usize = self.passive_permutations.len();

        let symbols = self.lcl_problem.symbol_map.values().collect_vec();

        // Add clauses

        // 1. Adjacent nodes need to agree on the edge's label.
        // In other words, two adjacent nodes cannot label their shared edge differently.

        for node in &self.graph.partition_a {
            for neighbour in self.graph.graph.neighbors(*node) {
                let all_symbol_pairs = symbols.iter().permutations(2);

                for symbol_pair in all_symbol_pairs {
                    let var_node = self.var_label(
                        true,
                        node.index(),
                        neighbour.index(),
                        active_permutations_len,
                        passive_permutations_len,
                        **symbol_pair[0] as usize,
                    );
                    let var_neighbour = self.var_label(
                        true,
                        node.index(),
                        neighbour.index(),
                        active_permutations_len,
                        passive_permutations_len,
                        **symbol_pair[1] as usize,
                    );
                    clauses.extend(only_one(&[var_node, var_neighbour]));
                }
            }
        }

        // 2. Nodes need to have a valid labeling.

        // 2.1 Each active node has only one permutation
        for active_node in &self.graph.partition_a {
            let vars = (0..active_permutations_len)
                .map(|permutation_index| {
                    self.var_permutation(
                        true,
                        active_node.index(),
                        permutation_index,
                        active_permutations_len,
                        passive_permutations_len,
                    )
                })
                .collect_vec();
            clauses.extend(only_one(&vars));
        }

        // 2.2 Each passive node has only one permutation
        for passive_node in &self.graph.partition_a {
            let vars = (0..passive_permutations_len).map(|permutation_index| {
                self.var_permutation(
                    true,
                    passive_node.index(),
                    permutation_index,
                    active_permutations_len,
                    passive_permutations_len,
                )
            });
            clauses.extend(only_one(&vars.collect_vec()));
        }

        // 2.3 If a node has a labeling (a permutation of a configuration) then and only then
        // the labeling must hold true.

        // 2.3.1 Active nodes
        for active_node in &self.graph.partition_a {
            for (permutation_index, permutation) in self.active_permutations.iter().enumerate() {
                let var_permutation = self.var_permutation(
                    true,
                    active_node.index(),
                    permutation_index,
                    active_permutations_len,
                    passive_permutations_len,
                );

                for (neighbour_index, neighbour) in
                    self.graph.graph.neighbors(*active_node).enumerate()
                {
                    let var_label = self.var_label(
                        true,
                        active_node.index(),
                        neighbour.index(),
                        active_permutations_len,
                        passive_permutations_len,
                        permutation[neighbour_index] as usize,
                    );

                    clauses.extend(implies(var_permutation, var_label));
                }
            }
        }

        // 2.3.2 Passive nodes
        for passive_node in &self.graph.partition_b {
            for (permutation_index, permutation) in self.passive_permutations.iter().enumerate() {
                let var_permutation = self.var_permutation(
                    false,
                    passive_node.index(),
                    permutation_index,
                    active_permutations_len,
                    passive_permutations_len,
                );

                for (neighbour_index, neighbour) in
                    self.graph.graph.neighbors(*passive_node).enumerate()
                {
                    let var_label = self.var_label(
                        false,
                        passive_node.index(),
                        neighbour.index(),
                        active_permutations_len,
                        passive_permutations_len,
                        permutation[neighbour_index] as usize,
                    );

                    clauses.extend(implies(var_permutation, var_label));
                }
            }
        }

        // End adding clauses

        clauses
    }

    fn _clauses_into_cnf_dimacs(&self, clauses: &Clauses, variable_count: usize) -> String {
        let mut result = String::new();
        result.push_str(&format!("p cnf{} {}\n", variable_count, clauses.len()));

        clauses.iter().for_each(|x| {
            let clause = format!("{} 0\n", x.iter().join(" "));
            result.push_str(&clause);
        });
        result
    }

    fn var_permutation(
        &self,
        active: bool,
        node_index: usize,
        permutation_index: usize,
        active_permutations_size: usize,
        passive_permutations_size: usize,
    ) -> i32 {
        /*
        version 1   version 2
        v_A_1_1     v_1
        v_A_1_2     v_2
        v_A_1_3     v_3
        v_A_2_1     v_4

        total number of variables: a_nodes * a_permutations + p_nodes * p_permutations
        */

        let active_nodes_size = self.graph.partition_a.len();
        if active {
            let (active_index, _active_nodeindex) = self
                .graph
                .partition_a
                .iter()
                .find_position(|x| x.index() == node_index)
                .expect("Something went wrong :(");
            return (active_index * active_permutations_size + permutation_index) as i32;
        }

        let (passive_index, _passive_nodeindex) = self
            .graph
            .partition_b
            .iter()
            .find_position(|x| x.index() == node_index)
            .expect("Something went wrong :(");

        let _passive_nodes_size = self.graph.partition_b.len();

        return (active_nodes_size * active_permutations_size
            + passive_index * passive_permutations_size
            + permutation_index
            + 1) as i32;
    }

    fn var_label(
        &self,
        first_active: bool,
        node_index_0: usize,
        node_index_1: usize,
        active_permutations_size: usize,
        passive_permutations_size: usize,
        symbol: usize,
    ) -> i32 {
        let active_nodes_size = self.graph.partition_a.len();
        let passive_nodes_size = self.graph.partition_b.len();

        // Variables in range 0..base are reserved for permutations.
        let base = (active_nodes_size * active_permutations_size
            + passive_nodes_size * passive_permutations_size) as i32;

        let symbols_size = self.lcl_problem.symbol_map.len();

        let (active_node, passive_node) = match first_active {
            true => (node_index_0, node_index_1),
            false => (node_index_1, node_index_0),
        };

        let (active_index, _active_nodeindex) = self
            .graph
            .partition_a
            .iter()
            .find_position(|x| x.index() == active_node)
            .expect("Something went wrong :(");

        let (passive_index, _passive_nodeindex) = self
            .graph
            .partition_b
            .iter()
            .find_position(|x| x.index() == passive_node)
            .expect("Something went wrong :(");

        if first_active {
            let v = active_index * passive_nodes_size * symbols_size
                + passive_index * symbols_size
                + symbol;
            return base + (v as i32);
        }

        let v =
            passive_index * active_nodes_size * symbols_size + active_index * symbols_size + symbol;

        // Variables in range base..base+active_passive_label_variables_size
        // are reserved for labels over edge from active node to passive node.
        let active_passive_label_variables_size =
            (active_nodes_size * passive_nodes_size * symbols_size) as i32;
        return base + active_passive_label_variables_size + (v as i32);
    }
}

fn at_least_one(variables: &[i32]) -> Clauses {
    vec![variables.into_iter().copied().collect_vec()]
}

fn at_most_one(variables: &[i32]) -> Clauses {
    variables.iter().map(|x| -x).combinations(2).collect_vec()
}

fn only_one(variables: &[i32]) -> Clauses {
    [at_least_one(variables), at_most_one(variables)].concat()
}

fn implies(variable_0: i32, variable_1: i32) -> Clauses {
    vec![vec![-variable_0, variable_1]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_least_one() {
        let a = vec![1, 2, 3, 4];
        let left = at_least_one(&a);
        let right = vec![vec![1, 2, 3, 4]];
        assert_eq!(left, right);
    }

    #[test]
    fn test_at_most_one() {
        let a = vec![1, 2, 3, 4];
        let left = at_most_one(&a);
        let right = vec![
            vec![-1, -2],
            vec![-1, -3],
            vec![-1, -4],
            vec![-2, -3],
            vec![-2, -4],
            vec![-3, -4],
        ];
        assert_eq!(left, right);
    }

    #[test]
    fn test_implies() {
        assert_eq!(implies(1, 2), vec![vec![-1, 2]]);
    }
}
