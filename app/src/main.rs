mod app;
mod handler;
mod state;
use state::run_app;

fn main() {
    let _ = run_app();
}
