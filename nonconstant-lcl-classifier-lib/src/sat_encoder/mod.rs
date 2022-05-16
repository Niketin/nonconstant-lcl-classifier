use crate::lcl_problem::LclProblem;
use crate::BiregularGraph;
use itertools::Itertools;
use petgraph::graph::EdgeReference;
use petgraph::visit::EdgeIndexable;
use petgraph::visit::EdgeRef;
use NodeOrderInEdgeRef::{ActivePassive, PassiveActive};

pub type Clause = Vec<i32>;
pub type Clauses = Vec<Clause>;
pub type Permutations = Vec<Vec<u8>>;

/// SAT problem encoder for LCL problems and biregular graphs.
///
/// `SatEncoder` can be used to encode LCL problems and biregular graphs into CNF DIMACS format.
/// This encoded form can be used as input to most SAT solvers.
/// Solving this encoded form tells if we can find a valid labelings for the graph.
///
/// More about SAT [here](https://en.wikipedia.org/wiki/Boolean_satisfiability_problem).
pub struct SatEncoder {
    graph: BiregularGraph,
    active_permutations: Permutations,
    passive_permutations: Permutations,
    labels: Vec<u8>,
}

enum NodeOrderInEdgeRef {
    ActivePassive,
    PassiveActive,
}

impl SatEncoder {
    /// Initializes new SatEncoder with an LCL problem and a biregular graph.
    ///
    /// Permutations of labels in every configuration are calculated and saved in
    /// the fields 'active_permutations' and 'passive_permutations' of the struct.
    /// Only unique permutations are saved.
    /// More about permutations in documentation of function [`crate::Configurations::get_permutations`].
    pub fn new(lcl_problem: &LclProblem, graph: BiregularGraph) -> SatEncoder {
        let active_permutations: Permutations = lcl_problem.active.get_permutations();
        let passive_permutations: Permutations = lcl_problem.passive.get_permutations();

        let labels_active = lcl_problem.active.get_labels_set();
        let labels_passive = lcl_problem.passive.get_labels_set();
        let labels = labels_active.union(&labels_passive).copied().collect_vec();

        SatEncoder {
            graph,
            active_permutations,
            passive_permutations,
            labels,
        }
    }

    /// Returns a reference of the inner graph.
    pub fn get_graph(&self) -> &BiregularGraph {
        &self.graph
    }

    /// Encodes LCL problem and a bipartite graph into CNF form.
    ///
    /// Returns clauses of type `Clauses`.
    pub fn encode(&self) -> Clauses {
        let mut clauses: Clauses = vec![];

        let active_permutations_len: usize = self.active_permutations.len();
        let passive_permutations_len: usize = self.passive_permutations.len();

        // 1. Adjacent nodes need to agree on the edge's label.
        // In other words, two adjacent nodes cannot label their shared edge differently.
        for node in &self.graph.partition_a {
            for incident_edge in self.graph.graph.edges(*node) {
                let all_label_pairs = self.labels.iter().permutations(2);

                for label_pair in all_label_pairs {
                    let var_node =
                        self.var_label(ActivePassive, incident_edge, *label_pair[0] as usize);
                    let var_neighbour =
                        self.var_label(PassiveActive, incident_edge, *label_pair[1] as usize);
                    clauses.extend(at_most_one(&[var_node, var_neighbour]));
                }
            }
        }

        // 2. Nodes need to have a valid labeling.

        // 2.1 Each active node has only one permutation
        for active_node in &self.graph.partition_a {
            let vars = (0..active_permutations_len)
                .map(|permutation_index| {
                    self.var_permutation(true, active_node.index(), permutation_index)
                })
                .collect_vec();
            clauses.extend(only_one(&vars));
        }

        // 2.2 Each passive node has only one permutation
        for passive_node in &self.graph.partition_b {
            let vars = (0..passive_permutations_len).map(|permutation_index| {
                self.var_permutation(false, passive_node.index(), permutation_index)
            });
            clauses.extend(only_one(&vars.collect_vec()));
        }

        // 2.3 If a node has a labeling (a permutation of a configuration) then and only then
        // the labeling must hold true.

        // 2.3.1 Active nodes
        for active_node in &self.graph.partition_a {
            for (permutation_index, permutation) in self.active_permutations.iter().enumerate() {
                let var_permutation =
                    self.var_permutation(true, active_node.index(), permutation_index);

                for (incident_edge_index, incident_edge) in
                    self.graph.graph.edges(*active_node).enumerate()
                {
                    let var_label = self.var_label(
                        ActivePassive,
                        incident_edge,
                        permutation[incident_edge_index] as usize,
                    );

                    clauses.extend(implies(var_permutation, var_label));
                }
            }
        }

        // 2.3.2 Passive nodes
        for passive_node in &self.graph.partition_b {
            for (permutation_index, permutation) in self.passive_permutations.iter().enumerate() {
                let var_permutation =
                    self.var_permutation(false, passive_node.index(), permutation_index);

                for (incident_edge_index, incident_edge) in
                    self.graph.graph.edges(*passive_node).enumerate()
                {
                    let var_label = self.var_label(
                        PassiveActive,
                        incident_edge,
                        permutation[incident_edge_index] as usize,
                    );

                    clauses.extend(implies(var_permutation, var_label));
                }
            }
        }

        clauses
    }

    /// Returns a string containing CNF DIMACS formatted clauses.
    ///
    /// # Useful links
    ///
    /// - [Specification](http://www.domagoj-babic.com/uploads/ResearchProjects/Spear/dimacs-cnf.pdf)
    /// - [Some site](https://people.sc.fsu.edu/~jburkardt/data/cnf/cnf.html)
    pub fn clauses_into_cnf_dimacs(&self, clauses: &Clauses, variable_count: usize) -> String {
        let mut result = String::new();
        result.push_str(&format!("p cnf{} {}\n", variable_count, clauses.len()));

        clauses.iter().for_each(|x| {
            let clause = format!("{} 0\n", x.iter().join(" "));
            result.push_str(&clause);
        });
        result
    }

    /// Returns a variable representing a permutation of labels in some configuration.
    ///
    /// # Parameters
    /// - `active` tells if the node is active or passive.
    /// - `node_index` is the index of the node in internal graph [`self.graph.graph`].
    /// - `permutation_index` is the index of permutation in its Configurations instance.
    fn var_permutation(&self, active: bool, node_index: usize, permutation_index: usize) -> i32 {
        let active_permutations_size = self.active_permutations.len();
        let passive_permutations_size = self.passive_permutations.len();
        let active_nodes_size = self.graph.partition_a.len();
        if active {
            let (active_index, _active_nodeindex) = self
                .graph
                .partition_a
                .iter()
                .find_position(|x| x.index() == node_index)
                .expect("Something went wrong :(");
            return (active_index * active_permutations_size + permutation_index + 1) as i32;
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

    /// Returns a variable representing an assigned label of an edge.
    ///
    /// The order of the nodes in `edge` is significant.
    /// In this encoding, both edges (v, w) and (w, v) need to have a same label.
    ///
    /// For an edge (v, w), the allowed labels are from the partition where v belongs.
    /// Respectively the allowed labels of (w, v) are from the partition where w belongs (the opposite partition of where v belongs).
    ///
    /// This is only for the purpose of encoding the problem as SAT.
    /// LCL itself maps labels for an edge independent of the order of edges (when undirected).
    ///
    /// # Parameters
    /// - `first_active` tells if the first node of the `edge` is active or passive. The second node is always in the opposite partition of the graph.
    /// - `edge` is the reference to the edge in internal graph [`self.graph.graph`].
    /// - `label` is the label of the label.
    fn var_label(
        &self,
        node_order: NodeOrderInEdgeRef,
        edge: EdgeReference<(), u32>,
        label: usize,
    ) -> i32 {
        let active_permutations_size = self.active_permutations.len();
        let passive_permutations_size = self.passive_permutations.len();
        let active_nodes_size = self.graph.partition_a.len();
        let passive_nodes_size = self.graph.partition_b.len();

        // Variables in range 1..(base + 1) are reserved for permutations.
        let base = (active_nodes_size * active_permutations_size
            + passive_nodes_size * passive_permutations_size
            + 1) as i32;

        let labels_count = self.labels.len();

        let v = edge.id().index() * labels_count + label;

        match node_order {
            ActivePassive => return base + (v as i32),
            PassiveActive => (),
        }

        // Variables in range base..base+active_passive_label_variables_size
        // are reserved for labels over edge from active node to passive node.
        let active_passive_label_variables_size =
            (self.graph.graph.edge_count() * labels_count) as i32;
        return base + active_passive_label_variables_size + (v as i32);
    }

    fn clause_to_string(&self, clause: &Clause) -> String {
        format!(
            "({})",
            clause.iter().map(|x| self.var_to_string(*x)).join(" || ")
        )
    }

    /// Variable to a human-readable string.
    ///
    /// There are 4 types of variables:
    /// - Active node permutation
    ///   - Output: "<sign>A<node_index>_<permutation_index>"
    ///   - Example: "-A3_4"
    /// - Passive node permutation
    ///   - Output: "<sign>P<node_index>_<permutation_index>"
    ///   - Example: "-P1_3"
    /// - Label of an edge between active and passive node
    ///   - Output: "<sign>AP_<edge_index>_<label>"
    ///   - Example: " AP_3_3"
    /// - Label of an edge between passive and active node
    ///   - Output: "<sign>PA_<edge_index>_<label>"
    ///   - Example: "-AP_2_1"
    ///
    fn var_to_string(&self, variable: i32) -> String {
        let is_positive = variable > 0;
        let variable_abs = variable.abs() as usize;
        let sign_str = if is_positive { " " } else { "-" };

        // Active node Permutation
        let active_nodes_len = self.graph.partition_a.len();
        let active_permutations_len = self.active_permutations.len();
        let active_permutation_variables_len = active_nodes_len * active_permutations_len;
        let range_active_node_permutation = 1..active_permutation_variables_len + 1;

        if range_active_node_permutation.contains(&variable_abs) {
            let active_index = (variable_abs - 1) / active_permutations_len;
            let permutation_index = (variable_abs - 1) % active_permutations_len;
            return format!("{}A{}_{}", sign_str, active_index, permutation_index);
        }

        // Passive node Permutation
        let passive_nodes_len = self.graph.partition_b.len();
        let passive_permutations_len = self.passive_permutations.len();
        let passive_permutation_variables_len = passive_nodes_len * passive_permutations_len;
        let base = active_permutation_variables_len + 1;
        let range_passive_node_permutation = base..base + passive_permutation_variables_len;

        if range_passive_node_permutation.contains(&variable_abs) {
            let passive_index = (variable_abs - base) / passive_permutations_len;
            let permutation_index = (variable_abs - base) % passive_permutations_len;
            return format!("{}P{}_{}", sign_str, passive_index, permutation_index);
        }

        // The variable was not representing a permutation.
        // It must be either of the following types.

        let labels_count = self.labels.len();

        // Labels of edge "starting from" active nodes
        let base = base + passive_permutation_variables_len;
        let range_active_edge_labels = base..base + (self.graph.graph.edge_count() * labels_count);
        if range_active_edge_labels.contains(&variable_abs) {
            let edge_index =
                EdgeIndexable::from_index(&self.graph.graph, (variable_abs - base) / labels_count);
            let temp = (variable_abs - base) % (passive_nodes_len * labels_count);
            let label = temp % labels_count;
            return format!("{}AP_{}_{}", sign_str, edge_index.index(), label);
        }

        // Labels of edge "starting from" passive nodes
        let base = base + self.graph.graph.edge_count() * labels_count;
        let range_passive_edge_labels = base..base + (self.graph.graph.edge_count() * labels_count);
        if range_passive_edge_labels.contains(&variable_abs) {
            let edge_index =
                EdgeIndexable::from_index(&self.graph.graph, (variable_abs - base) / labels_count);
            let temp = (variable_abs - base) % (active_nodes_len * labels_count);
            let label = temp % labels_count;
            return format!("{}PA_{}_{}", sign_str, edge_index.index(), label);
        }

        unreachable!();
    }

    /// Prints clauses in a human-readable format.
    ///
    /// `clauses` must be from the same instance of SatEncoder
    /// because the information of graph and LCL problem is needed for this.
    pub fn print_clauses(&self, clauses: &Clauses) {
        clauses
            .iter()
            .for_each(|ref clause| println!("{} &&", self.clause_to_string(clause)));
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
