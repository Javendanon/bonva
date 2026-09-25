use anyhow::{Context, Result};
use serde_json::{Value, json};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub fn run(
    command: &[String],
    cwd: &Path,
    env: &BTreeMap<String, String>,
    timeout: Duration,
    logs: &Path,
    name: &str,
) -> Result<Value> {
    fs::create_dir_all(logs)?;
    let out_path = logs.join(format!("{name}.stdout"));
    let err_path = logs.join(format!("{name}.stderr"));
    let output = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&out_path)?;
    let error = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&err_path)?;
    let mut cmd = Command::new(command.first().context("Empty command")?);
    cmd.args(&command[1..])
        .current_dir(cwd)
        .env_clear()
        .envs(env)
        .stdin(Stdio::null())
        .stdout(output)
        .stderr(error);
    #[cfg(unix)]
    cmd.process_group(0);
    let start = Instant::now();
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Ok(
                json!({"command":command,"exit_code":null,"status":"unavailable","stdout":"","stderr":e.to_string(),"elapsed_ms":start.elapsed().as_secs_f64()*1000.0}),
            );
        }
    };
    let mut timed_out = false;
    let exit = loop {
        if let Some(s) = child.try_wait()? {
            break s;
        }
        if start.elapsed() >= timeout {
            timed_out = true;
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(10));
    };
    // Do not leave background grandchildren holding the workspace after a command.
    #[cfg(unix)]
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    Ok(
        json!({"command":command,"exit_code":exit.code(),"status":if timed_out{"timeout"}else{"finished"},"stdout":String::from_utf8_lossy(&fs::read(out_path)?),"stderr":String::from_utf8_lossy(&fs::read(err_path)?),"elapsed_ms":start.elapsed().as_secs_f64()*1000.0}),
    )
}
pub fn success(result: &Value) -> Option<bool> {
    if result["status"] == "unavailable" {
        None
    } else {
        Some(result["status"] == "finished" && result["exit_code"] == 0)
    }
}
