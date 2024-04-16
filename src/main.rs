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

fn read_output(
    task_id: &str,
    task: &QuafuTask,
    qubits: u32,
    shots: u32,
) -> (Vec<usize>, String, String) {
    if 0 == task.shots {
        todo!()
    } else {
        let mut stats = data::Statistics {
            memory: HashMap::new(),
        };
        data::read_stats(&mut stats, &format!("/tmp/{}", task_id), qubits, shots);
        data::print_stats(&stats, qubits, shots)
    }
}

fn fetch_task(request_interval: u64, quafu_address: &str, system_id: usize) {
    thread::sleep(time::Duration::from_secs(request_interval));
    // system_id=7 is the chip id of the quafu system
    let res = match reqwest::blocking::get(&format!(
        "{}scq_task/?system_id={}",
        quafu_address, system_id
    ))
    .unwrap()
    .json::<Value>()
    {
        Ok(res) => res,
        Err(e) => {
            warn!("{}", format!("Fetch task failed: {}", e.to_string()));
            Value::Null
        }
    };

    match res {
        Value::Null => {}
        _ => {
            info!("Task {} received", res["task_id"].as_str().unwrap());
            let mut task = QuafuTask::new(
                res["qubits"].as_u64().unwrap() as u32,
                res["circuit"].as_str().unwrap().to_string(),
                res["shots"].as_u64().unwrap() as u32,
                res["task_id"].as_str().unwrap().to_string(),
            );

            if !task.circuit.contains("measure") {
                warn!("Task {} doesn't contain measure op", task.task_id);
                for qubit in 0..task.qubits {
                    task.circuit
                        .push_str(&format!("\r\nmeasure q[{}] -> c[{}];", qubit, qubit));
                }
            }

            save_source_file(&task.circuit, &task.task_id).unwrap();
            let response = match run_program(&task.task_id, &task) {
                Ok(exec_output) if exec_output.status.code() == Some(0) => {
                    let output = read_output(&task.task_id, &task, task.qubits, task.shots);
                    info!("Task {} finished", task.task_id);
                    QuafuResponse {
                        task_id: task.task_id.clone(),
                        status: "finished".to_string(),
                        measure: format!("{:?}", output.0),
                        raw: output.1,
                        res: output.2,
                        server: system_id,
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
                        server: system_id,
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
                        server: system_id,
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

    let quafu_ip = std::env::var("QUAFU_IP").unwrap_or_else(|_| "120.46.209.71".to_string());
    let system_id = std::env::var("SYSTEM_ID")
        .unwrap_or_else(|_| "7".to_string())
        .parse::<usize>()
        .unwrap();

    let quafu_addr = format!("http://{}/qbackend/", quafu_ip);

    info!("Quafu IP: {}", quafu_addr);
    info!("System id: {}", system_id);

    loop {
        fetch_task(1, &quafu_addr, system_id)
    }
}
