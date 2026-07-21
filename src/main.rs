fn main() {
    if let Err(error) = openaula_cli::run() {
        eprintln!("Error: {error:#}");
        std::process::exit(1);
    }
}
