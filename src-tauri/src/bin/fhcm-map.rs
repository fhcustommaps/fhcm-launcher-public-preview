fn main() -> std::process::ExitCode {
    match fhcm_launcher::run_map_cli(std::env::args_os().skip(1)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("FHCM map: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
