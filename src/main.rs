use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    process::{Command, Output},
    thread, time,
};

use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod data;

#[derive(Debug)]
pub struct QuafuTask {
    pub qubits: u32,
    pub circuit: String,
    pub shots: u32,
    pub task_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QuafuResponse {
    #[serde(rename(serialize = "task_id"))]
    pub task_id: String,
    pub status: String,
    pub measure: String,
    pub raw: String,
    pub res: String,
    pub server: usize,
}

impl QuafuTask {
    pub fn new(qubits: u32, circuit: String, shots: u32, task_id: String) -> Self {
        Self {
            qubits,
            circuit,
            shots,
            task_id,
        }
    }
}

fn save_source_file(code: &str, task_id: &str) -> io::Result<()> {
    let source = "/tmp/".to_string() + task_id + ".qasm";
    let mut file = File::create(source)?;
    file.write_all(code.as_bytes())
}

fn remove_source_target_files(task_id: &str) -> io::Result<()> {
    fs::remove_file(&format!("/tmp/{}.qasm", task_id))?;
    fs::remove_file(&format!("/tmp/{}.stats", task_id))?;
    fs::remove_file(&format!("/tmp/{}.state", task_id))?;
    Ok(())
}

fn run_program(task_id: &str, task: &QuafuTask) -> io::Result<Output> {
    Command::new("qpp-agent")
        .arg("-s")
        .arg(task.shots.to_string())
        .arg("-f")
        .arg(&format!("/tmp/{}.qasm", task_id))
        .arg("--simulator")
        .arg("sv")
        .arg("-o")
        .arg(&format!("/tmp/{}", task_id))
        .output()
}

fn read_output(task_id: &str, task: &QuafuTask) -> (Vec<usize>, String, String) {
    if 0 == task.shots {
        todo!()
    } else {
        let mut stats = data::Statistics {
            memory: HashMap::new(),
        };
        data::read_stats(&mut stats, &format!("/tmp/{}", task_id));
        data::print_stats(&stats)
    }
}

fn fetch_task(request_interval: u64, quafu_address: &str) {
    thread::sleep(time::Duration::from_secs(request_interval));
    // system_id=7 is the chip id of the quafu system
    let res = reqwest::blocking::get(&format!("{}scq_task/?system_id=7", quafu_address))
        .unwrap()
        .json::<Value>()
        .unwrap();

    match res {
        Value::Null => {}
        _ => {
            info!("Task {} received", res["task_id"].as_str().unwrap());
            let task = QuafuTask::new(
                res["qubits"].as_u64().unwrap() as u32,
                res["circuit"].as_str().unwrap().to_string(),
                res["shots"].as_u64().unwrap() as u32,
                res["task_id"].as_str().unwrap().to_string(),
            );

            save_source_file(&task.circuit, &task.task_id).unwrap();
            let response = match run_program(&task.task_id, &task) {
                Ok(exec_output) if exec_output.status.code() == Some(0) => {
                    let output = read_output(&task.task_id, &task);
                    info!("Task {} finished", task.task_id);
                    QuafuResponse {
                        task_id: task.task_id.clone(),
                        status: "finished".to_string(),
                        measure: format!("{:?}", output.0),
                        raw: output.1,
                        res: output.2,
                        server: 7,
                    }
                }
                Ok(_) => {
                    warn!("Task {} failed", task.task_id);
                    QuafuResponse {
                        task_id: task.task_id.clone(),
                        status: "failed".to_string(),
                        measure: "".to_string(),
                        raw: "".to_string(),
                        res: "".to_string(),
                        server: 7,
                    }
                }
                Err(e) => {
                    warn!("Task {} failed", task.task_id);
                    QuafuResponse {
                        task_id: task.task_id.clone(),
                        status: "failed".to_string(),
                        measure: "".to_string(),
                        raw: e.to_string(),
                        res: "".to_string(),
                        server: 7,
                    }
                }
            };

            let post_res = reqwest::blocking::Client::new()
                .post(&format!("{}scq_result/", quafu_address))
                .form(&response)
                .send()
                .unwrap();
            match post_res.status() {
                reqwest::StatusCode::OK => {
                    info!("Post task result {} succeed", &task.task_id);
                }
                _ => {
                    warn!("Post task {} failed", &task.task_id);
                    warn!("{:?}", post_res.text());
                }
            }

            match remove_source_target_files(&task.task_id) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Remove task {} failed", &task.task_id);
                    warn!("{}", e);
                }
            }
        }
    }
}

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    if std::path::Path::new(".env").exists() {
        dotenv::from_filename(".env").ok();
    }

    let quafu_addr = std::env::var("QUAFU_ADDR")
        .unwrap_or_else(|_| "http://120.46.209.71/qbackend/".to_string());

    info!("Quafu address: {}", quafu_addr);

    loop {
        fetch_task(1, &quafu_addr)
    }
}
