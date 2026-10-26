//! Renders or checks `artifacts/sacco_ledger-idl.json`.

use std::process::ExitCode;

/// Repo-root artifact path, resolved at compile time so the command works from any cwd.
const ARTIFACT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../artifacts/sacco_ledger-idl.json"
);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rendered = sacco_ledger_idl::render();

    match args.as_slice() {
        [] => {
            print!("{rendered}");
            ExitCode::SUCCESS
        }
        [flag, path] if flag == "--out" => match std::fs::write(path, rendered) {
            Ok(()) => {
                println!("wrote {path}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("cannot write {path}: {error}");
                ExitCode::FAILURE
            }
        },
        [flag] if flag == "--check" => {
            let committed = std::fs::read_to_string(ARTIFACT).unwrap_or_default();
            if committed == rendered {
                println!("{ARTIFACT} is current");
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "{ARTIFACT} is stale: run `cargo run -p sacco_ledger_idl -- --out {ARTIFACT}`"
                );
                ExitCode::FAILURE
            }
        }
        _ => {
            eprintln!("usage: sacco_ledger_idl [--out <path> | --check]");
            ExitCode::FAILURE
        }
    }
}
