use clap::Parser;
use std::io::Write;

fn main() {
    let cli = asm_cli::Cli::parse();
    match asm_cli::execute(cli) {
        Ok(asm_cli::CliOutput::Json(result)) => match serde_json::to_string_pretty(&result) {
            Ok(output) => println!("{output}"),
            Err(error) => fail(&format!("failed to encode result: {error}")),
        },
        Ok(asm_cli::CliOutput::Text(output)) => {
            if let Err(error) = std::io::stdout().write_all(output.as_bytes()) {
                fail(&format!("failed to write stdout: {error}"));
            }
        }
        Err(error) => {
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

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1)
}
