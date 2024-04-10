use std::{collections::HashMap, fs};

use serde_json::json;

pub struct Statistics {
    pub memory: HashMap<String, usize>,
}

pub fn read_stats(stats: &mut Statistics, source: &str) {
    for line in fs::read_to_string(source.to_string() + ".stats")
        .unwrap()
        .lines()
    {
        let complex = line
            .split(" ")
            .filter_map(|s| s.parse::<usize>().ok())
            .collect::<Vec<_>>();

        stats.memory.insert(
            complex[..complex.len() - 1]
                .iter()
                .map(|c| c.to_string())
                .collect::<String>()
                .chars()
                .collect::<String>(),
            complex.last().unwrap().clone(),
        );
    }
}

pub fn print_stats(stats: &Statistics, qubits: u32, shots: u32) -> (Vec<usize>, String, String) {
    // get the vec of strings from 1 to length of the memory
    match stats.memory.iter().nth(0) {
        Some(v) => {
            let mut json = json!({});
            stats.memory.iter().for_each(|(key, value)| {
                json[key] = json!(value);
            });
            ((0..v.0.len()).collect(), json.to_string(), json.to_string())
        }
        None => {
            let mut json = json!({});
            json["0".repeat(qubits as usize)] = json!(shots);
            (
                (0..qubits as usize).collect(),
                json.to_string(),
                json.to_string(),
            )
        }
    }
}
