fn main() {
    match asm_cli::execute(std::env::args().skip(1)) {
        Ok(result) => match serde_json::to_string_pretty(&result) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("failed to encode result: {error}");
                std::process::exit(1);
            }
        },
        Err(error) => {
            if let asm_cli::CliError::Usage(message) = &error {
                eprintln!("{message}");
                std::process::exit(2);
            }
            eprintln!(
                "{}",
                serde_json::json!({
                    "ok": false,
                    "error": error.to_string(),
                })
            );
            std::process::exit(1);
        }
    }
}
