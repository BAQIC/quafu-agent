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

pub fn print_stats(stats: &Statistics) -> (Vec<usize>, String, String) {
    // get the vec of strings from 1 to length of the memory
    let measure = (0..stats.memory.iter().nth(0).unwrap().0.len()).collect();

    let mut json = json!({});
    stats.memory.iter().for_each(|(key, value)| {
        json[key] = json!(value);
    });

    (measure, json.to_string(), json.to_string())
}
