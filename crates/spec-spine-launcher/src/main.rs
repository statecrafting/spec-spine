// Spec: specs/188-a-repository-pin-selects-the-engine/spec.md
//! The spec-spine launcher (spec 188): find the repository, read its engine
//! pin, resolve and verify that exact release, execute it. It implements no
//! governance verb and depends on no engine crate.

mod acquire;
mod app;
mod cli;
mod engine;
mod envelope;
mod exec;
mod failure;
mod hash;
mod paths;
mod repo;
mod target;

use std::process::ExitCode;

/// The launcher's own name, as `launcher --version` and the envelope's `tool`
/// report it. `launcher --version` starting with this is how a file is
/// recognised as a launcher (spec 188 section 3.6).
pub const NAME: &str = "spec-spine-launcher";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    ExitCode::from(app::run(std::env::args_os().skip(1).collect()))
}
