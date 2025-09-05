mod cli;
mod types;
mod utils;

fn main() -> anyhow::Result<()> {
    cli::run()
}
