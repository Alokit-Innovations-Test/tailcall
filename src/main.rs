// Required for the #[global_allocator] proc macro
#![allow(clippy::too_many_arguments)]

use std::cell::Cell;

use tailcall::core::tracing::default_tracing_tailcall;
use tailcall::core::Errata;
use tracing::subscriber::DefaultGuard;

thread_local! {
    static TRACING_GUARD: Cell<Option<DefaultGuard>> = const { Cell::new(None) };
}

fn run_blocking() -> anyhow::Result<()> {
}

fn main() -> anyhow::Result<()> {
    // enable tracing subscriber for current thread until this block ends
    // that will show any logs from cli itself to the user
    // despite of @telemetry settings that
    let _guard = tracing::subscriber::set_default(default_tracing_tailcall());
    let result = run_blocking();
    match result {
        Ok(_) => {}
        Err(error) => {
            // Ensure all errors are converted to Errata before being printed.
            let cli_error: Errata = error.into();
            tracing::error!("{}", cli_error.color(true));
            std::process::exit(exitcode::CONFIG);
        }
    }
    Ok(())
}
