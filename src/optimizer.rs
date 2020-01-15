use crate::util::*;
use rand::{seq::SliceRandom, Rng};

// Population size
const POPULATION_SIZE: usize = 1000;

// Mutation rate
const MUTATION_RATE: f64 = 0.25;

// Ratio of "large mutations" (random u8 replacement) vs "small mutations" u8 increment / decrement.
const LARGE_MUTATION_RATIO: f64 = 0.25;

// Directly clone this ratio of top performers
const CLONE_RATIO: f64 = 0.05;

// Breed from this top percentage of the population
const BREEDING_POOL: f64 = 0.10;

pub struct Optimizer<T>
where
    T: FnMut(&[u8], &[u8]) -> ScoredInputPair,
{
    population: Vec<InputPair>,
    fitness: T,
    input_is_str: bool,
}

impl<T> Optimizer<T>
where
    T: FnMut(&[u8], &[u8]) -> ScoredInputPair,
{
    pub fn new(len: usize, fitness_function: T, input_is_str: bool) -> Self {
        Optimizer {
            population: inital_population(len, input_is_str),
            fitness: fitness_function,
            input_is_str,
        }
    }

    pub fn scored_population(&mut self) -> Vec<ScoredInputPair> {
        // Get fitness of all individuals
        let mut scored: Vec<ScoredInputPair> = Vec::with_capacity(self.population.len());

        for individual in self.population.iter() {
            let score = (self.fitness)(&individual.first, &individual.second);
            scored.push(score);
        }

        // Sort most fit to least fit
        // Unwrap OK since score cannot be NAN.
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        scored
    }

    pub fn step(&mut self) {
        // Get fitness of all individuals
        let scored = self.scored_population();

        // Calculate number to clone and number to breed
        let num_clone: usize = (POPULATION_SIZE as f64 * CLONE_RATIO) as usize;
        let breed_pool: usize = (POPULATION_SIZE as f64 * BREEDING_POOL) as usize;
        let breed_fill: usize = POPULATION_SIZE - num_clone;

        // Create the next generation
        let mut next_gen: Vec<InputPair> = Vec::with_capacity(self.population.len());

        // Clone the top contenders
        for score in scored.iter().take(num_clone) {
            next_gen.push(score.pair.clone());
        }

        // Breed and mutate the rest
        for _ in 0..breed_fill {
            // Select two individuals
            let parent_one = &scored[rand::thread_rng().gen_range(0, breed_pool)].pair;
            let parent_two = &scored[rand::thread_rng().gen_range(0, breed_pool)].pair;

            let mut child;
            if self.input_is_str {
                child = InputPair {
                    first: breed_str_slice(&parent_one.first, &parent_two.first),
                    second: breed_str_slice(&parent_one.second, &parent_two.second),
                    is_str: self.input_is_str,
                };
            } else {
                child = InputPair {
                    first: breed_slice(&parent_one.first, &parent_two.first),
                    second: breed_slice(&parent_one.second, &parent_two.second),
                    is_str: self.input_is_str,
                };
            }

            // Mutate
            if rand::thread_rng().gen_bool(MUTATION_RATE) {
                if rand::thread_rng().gen() {
                    if self.input_is_str {
                        mutate_str_slice(&mut child.first);
                    } else {
                        mutate_slice(&mut child.first);
                    }
                } else {
                    if self.input_is_str {
                        mutate_str_slice(&mut child.second);
                    } else {
                        mutate_slice(&mut child.second);
                    }
                }
            }

            next_gen.push(child);
        }

        self.population = next_gen;
    }
}

fn breed_slice(first: &[u8], second: &[u8]) -> Vec<u8> {
    let mut child: Vec<u8> = Vec::with_capacity(first.len());
    for n in 0..first.len() {
        if rand::thread_rng().gen() {
            child.push(first[n]);
        } else {
            child.push(second[n]);
        }
    }

    child
}

fn breed_str_slice(first: &[u8], second: &[u8]) -> Vec<u8> {
    let mut child: Vec<u8> = breed_slice(first, second);

    // Mutate until it's valid
    loop {
        match std::str::from_utf8(&child) {
            Ok(_) => return child,
            Err(_) => {}
        }
        mutate_slice(&mut child);
    }
}

fn mutate_slice(slice: &mut [u8]) {
    // OK to unwrap here, slice should never be empty
    let mutating_gene = slice.choose_mut(&mut rand::thread_rng()).unwrap();

    if rand::thread_rng().gen_bool(LARGE_MUTATION_RATIO) {
        // Large mutation, assign another random u8
        *mutating_gene = rand::thread_rng().gen();
    } else {
        // Small mutation, increment or decrement
        if rand::thread_rng().gen() {
            *mutating_gene = mutating_gene.wrapping_add(1);
        } else {
            *mutating_gene = mutating_gene.wrapping_sub(1);
        }
    }
}

fn mutate_str_slice(slice: &mut [u8]) {
    loop {
        mutate_slice(slice);

        match std::str::from_utf8(slice) {
            Ok(_) => return,
            Err(_) => continue,
        }
    }
}

fn inital_population(len: usize, is_str: bool) -> Vec<InputPair> {
    let mut population = Vec::with_capacity(POPULATION_SIZE);
    for _ in 0..POPULATION_SIZE {
        if is_str {
            population.push(random_str_individual(len));
        } else {
            population.push(random_individual(len));
        }
    }
    population
}

fn random_individual(len: usize) -> InputPair {
    InputPair {
        first: (0..len).map(|_| rand::random::<u8>()).collect(),
        second: (0..len).map(|_| rand::random::<u8>()).collect(),
        is_str: false,
    }
}

fn random_str_individual(len: usize) -> InputPair {
    use rand::distributions::Alphanumeric;

    // This will create an ascii strings, all under 127 value, we can translate it right to bytes
    let first: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect();
    let second: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .collect();

    InputPair {
        first: first.into_bytes(),
        second: second.into_bytes(),
        is_str: true,
    }
}

#[cfg(test)]
mod tests {
    use crate::optimizer::Optimizer;
    use crate::util::*;

    #[test]
    fn optimizer_test() {
        let target = b"GENETIC ALGOS!";
        let mut optimizer = Optimizer::new(
            target.len(),
            |first: &[u8], second: &[u8]| {
                let mut score: f64 = 0.0;
                for item in [first, second].iter() {
                    for (i, byte) in item.iter().enumerate() {
                        let diff = if &target[i] > byte {
                            target[i] - byte
                        } else if byte > &target[i] {
                            byte - target[i]
                        } else {
                            0
                        };
                        score = score - (diff as f64);
                    }
                }
                ScoredInputPair {
                    score,
                    highest: 0.0,
                    lowest: 0.0,
                    pair: InputPair {
                        first: first.to_vec(),
                        second: second.to_vec(),
                        is_str: false,
                    },
                }
            },
            false,
        );

        // Run one hundred generations
        for _ in 0..1000 {
            optimizer.step();
        }

        // This will be sorted
        let population = optimizer.scored_population();

        assert_eq!(population[0].pair.first, target);
        assert_eq!(population[0].pair.second, target);
    }
}
