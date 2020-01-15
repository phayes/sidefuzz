// Misc utility functions used by various parts of the program

use crate::wasm::WasmModule;
use std::f64::{NAN, NEG_INFINITY};

#[derive(Debug, Clone, Default)]
pub struct InputPair {
    pub first: Vec<u8>,
    pub second: Vec<u8>,
    pub is_str: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ScoredInputPair {
    pub score: f64,
    pub highest: f64,
    pub lowest: f64,
    pub pair: InputPair,
}

impl ScoredInputPair {
    pub fn generate(
        module: &mut WasmModule,
        first: Vec<u8>,
        second: Vec<u8>,
        is_str: bool,
    ) -> Self {
        // First
        let first_instructions = module.count_instructions(&first);
        let first_instructions = match first_instructions {
            Ok(count) => count,
            Err(_) => {
                // WASM trapped, score is negative infinity
                return ScoredInputPair {
                    score: NEG_INFINITY,
                    highest: NAN,
                    lowest: NAN,
                    pair: InputPair {
                        first,
                        second,
                        is_str,
                    },
                };
            }
        };

        // Second
        let second_instructions = module.count_instructions(&second);
        let second_instructions = match second_instructions {
            Ok(count) => count,
            Err(_) => {
                // WASM trapped, score is negative infinity
                return ScoredInputPair {
                    score: NEG_INFINITY,
                    highest: NAN,
                    lowest: NAN,
                    pair: InputPair {
                        first,
                        second,
                        is_str,
                    },
                };
            }
        };

        // Differences, highest, lowest etc.
        let highest;
        let lowest;
        if first_instructions >= second_instructions {
            highest = first_instructions;
            lowest = second_instructions;
        } else {
            highest = second_instructions;
            lowest = first_instructions;
        }
        let diff = highest - lowest;

        // TODO: Add Enum FirstHighest, SecondHighest and use to print

        ScoredInputPair {
            score: diff as f64,
            highest: highest as f64,
            lowest: lowest as f64,
            pair: InputPair {
                first,
                second,
                is_str,
            },
        }
    }
}

// Given a t-value, the the p-value from it.
//
// This currently uses t-tables, in the future it will use an actual formula.
pub(crate) fn p_value_from_t_value(t: f64) -> f64 {
    // TODO: use formula instead of table.

    if t <= 0.0 {
        return 1.0; // 0% confidence.
    }

    // Assume infinite degrees of freedom
    // Two tailed t test
    let t_table = vec![
        (10.000, 0.0), // 100% confidence
        (3.91, 0.0001),
        (3.291, 0.001),
        (3.090, 0.002),
        (2.807, 0.005),
        (2.576, 0.01),
        (2.326, 0.02),
        (1.960, 0.05),
        (1.645, 0.1),
        (1.282, 0.2),
        (1.036, 0.3),
        (0.842, 0.4),
        (0.674, 0.5),
        (0.253, 0.6),
        (0.0, 1.0), // 0% confidence
    ];

    for (t_value, p_value) in t_table {
        if t > t_value {
            return p_value;
        }
    }

    panic!("Invalid t value");
}
