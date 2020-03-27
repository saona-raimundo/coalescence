//! Coalescent process.

// Types
use partitions::PartitionVec;
use rand_distr::Exp;

// Traits
use itertools::Itertools;
use markovian::traits::CMarkovChainTrait;
use rand::distributions::Distribution;
use rand::Rng;

/// n-Coalescent process in the space of partitions of the set {1, 2, ..., n}.
/// Starts with a finite partition of all singletons and it ends with a single set.
///
#[derive(Debug, Clone)]
pub struct Coalescent<R>
where
    R: Rng + Clone,
{
    state: PartitionVec<()>, // No selection
    rng: R,
}

impl<R> Coalescent<R>
where
    R: Rng + Clone,
{
    pub fn new(group_size: usize, rng: R) -> Self {
        let state: PartitionVec<()> =
            PartitionVec::from((0..group_size).map(|_| ()).collect::<Vec<()>>());

        Coalescent { state, rng }
    }
    pub fn rng(&mut self) -> &mut R {
        &mut self.rng
    }
    pub fn set_rng(&mut self, other_rng: R) -> &mut Self {
        self.rng = other_rng;
        self
    }

    /// Generates a realization from the current state until there is only one set
    /// in the partition. Note that the internal random number generator will change,
    /// but the state of the process will not change. This is why the process is not consummed.  
    pub fn generate_realization(&mut self) -> Vec<(f64, PartitionVec<()>)> {
        let initial_state = self.state().clone();
        let starting_point = vec![(0.0, initial_state.clone())];
        let result = starting_point.into_iter().chain(self.clone()).collect();
        self.set_state(initial_state);

        result
    }
}

impl<R> CMarkovChainTrait<PartitionVec<()>> for Coalescent<R>
where
    R: Rng + Clone,
{
    fn state(&self) -> &PartitionVec<()> {
        &self.state
    }
    fn set_state(&mut self, state: PartitionVec<()>) -> &mut Self {
        self.state = state;
        self
    }
}

impl<R> Iterator for Coalescent<R>
where
    R: Rng + Clone,
{
    type Item = (f64, PartitionVec<()>);

    fn next(&mut self) -> Option<Self::Item> {
        let current_partition_size = self.state.amount_of_sets();

        if current_partition_size == 1 {
            None
        } else {
            // Simulate time step

            let rate = current_partition_size as f64;
            let exp = Exp::new(rate).unwrap();
            let time_step = exp.sample(&mut rand::thread_rng());

            // Choose between possible transitions

            let next_partition_index: usize = self
                .rng()
                .gen_range(0, current_partition_size * (current_partition_size - 1) / 2);

            let (set_index_1, set_index_2) = (0..current_partition_size)
                .tuple_combinations()
                .nth(next_partition_index)
                .unwrap();

            // Join these sets

            let (value_index_1, _) = self
                .state
                .all_sets()
                .nth(set_index_1)
                .unwrap()
                .nth(0)
                .unwrap();
            let (value_index_2, _) = self
                .state
                .all_sets()
                .nth(set_index_2)
                .unwrap()
                .nth(0)
                .unwrap();

            // Update chain

            self.state.union(value_index_1, value_index_2);

            // Return

            Some((time_step, self.state.clone()))
        }
    }
}