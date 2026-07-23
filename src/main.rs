use clap::Parser;

#[derive(Parser)]
#[command(name = "backup", version = "0.1.0")]
struct Cli {}

fn main() {
    let _ = Cli::parse();
}
