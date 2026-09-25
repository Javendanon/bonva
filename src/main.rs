use std::process::ExitCode;

fn main() -> ExitCode {
    match agent_quality::cli::main() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::json!({"status":"error", "error":error.to_string()})
            );
            ExitCode::from(2)
        }
    }
}
