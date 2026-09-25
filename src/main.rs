fn main() {
    if let Err(error) = agent_quality::cli::main() {
        eprintln!(
            "{}",
            serde_json::json!({"status":"error", "error":error.to_string()})
        );
        std::process::exit(2);
    }
}
