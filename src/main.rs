fn main() {
    if let Err(error) = perlchecker::cli::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
