use rand::{seq::SliceRandom, Rng};

// TODO: Find optimal values for these consts

// Population size
const POPULATION_SIZE: usize = 100;

// Mutation rate
const MUTATION_RATE: f64 = 0.05;

// Ratio of "large mutations" (random u8 replacement) vs "small mutations" u8 increment / decrement.
const LARGE_MUTATION_RATIO: f64 = 0.25;

// Directly clone this ratio of top performers
const CLONE_RATIO: f64 = 0.25;

// Breed from this top percentage of the population
const BREEDING_POOL: f64 = 0.50;

pub struct Optimizer<T>
where
  T: Fn(&[u8], &[u8]) -> f64,
{
  population: Vec<(Vec<u8>, Vec<u8>)>,
  fitness: T,
}

impl<T> Optimizer<T>
where
  T: Fn(&[u8], &[u8]) -> f64,
{
  pub fn new(len: usize, fitness_function: T) -> Self {
    Optimizer {
      population: inital_population(len),
      fitness: fitness_function,
    }
  }

  // Get the population, ordered most fit to least fit.
  pub fn population(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
    // Get fitness of all individuals
    let mut scored: Vec<(f64, (Vec<u8>, Vec<u8>))> = Vec::with_capacity(self.population.len());

    for individual in self.population.iter() {
      let individual = individual.clone();
      let fitness = (self.fitness)(&individual.0, &individual.1);
      scored.push((fitness, individual));
    }

    // Sort most fit to least fit
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let mut result: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(self.population.len());
    for individual in scored.drain(..) {
      result.push(individual.1);
    }

    result
  }

  pub fn step(&mut self) {
    // Get fitness of all individuals
    let mut scored: Vec<(f64, (Vec<u8>, Vec<u8>))> = Vec::with_capacity(self.population.len());

    for individual in self.population.drain(..) {
      let fitness = (self.fitness)(&individual.0, &individual.1);
      scored.push((fitness, individual));
    }

    // Sort most fit to least fit
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Calculate number to clone and number to breed
    let num_clone: usize = (POPULATION_SIZE as f64 * CLONE_RATIO) as usize;
    let breed_pool: usize = (POPULATION_SIZE as f64 * BREEDING_POOL) as usize;
    let breed_fill: usize = POPULATION_SIZE - num_clone;

    // Create the next generation
    let mut next_gen: Vec<(Vec<u8>, Vec<u8>)> = Vec::with_capacity(self.population.len());

    // Clone the top contenders
    for n in 0..num_clone {
      next_gen.push(scored[n].1.clone());
    }

    // Breed and mutate the rest
    for _ in 0..breed_fill {
      // Select two individuals
      let first = &scored[rand::thread_rng().gen_range(0, breed_pool)].1;
      let second = &scored[rand::thread_rng().gen_range(0, breed_pool)].1;

      let mut child = (
        breed_slice(&first.0, &second.0),
        breed_slice(&first.1, &second.1),
      );

      // Mutate
      if rand::thread_rng().gen_bool(MUTATION_RATE) {
        if rand::thread_rng().gen() {
          mutate_slice(&mut child.0);
        } else {
          mutate_slice(&mut child.1);
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

fn inital_population(len: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
  let mut population = Vec::with_capacity(POPULATION_SIZE);
  for _ in 0..POPULATION_SIZE {
    population.push(random_individual(len));
  }
  population
}

fn random_individual(len: usize) -> (Vec<u8>, Vec<u8>) {
  (
    (0..len).map(|_| rand::random::<u8>()).collect(),
    (0..len).map(|_| rand::random::<u8>()).collect(),
  )
}

#[cfg(test)]
mod tests {
  use crate::optimizer::Optimizer;

  #[test]
  fn optimizer_test() {
    let target = b"GENETIC ALGOS!";
    let mut optimizer = Optimizer::new(target.len(), |first: &[u8], second: &[u8]| {
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
      score
    });

    // Run one thousand times
    for _ in 0..1000 {
      optimizer.step();
    }

    // This will be sorted
    let population = optimizer.population();

    assert_eq!(population[0].0, target);
    assert_eq!(population[0].1, target);
  }
}
