//! Execution (spec 188 section 3.6): the original arguments, working
//! directory, environment and standard streams, plus `SPEC_SPINE_LAUNCHED`.

use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use crate::failure::{Failure, Res};

fn command(engine: &Path, args: &[OsString]) -> Command {
    let mut c = Command::new(engine);
    c.args(args).env("SPEC_SPINE_LAUNCHED", crate::VERSION);
    c
}

/// On Unix the launcher's process is replaced by the engine's (`execve`, the
/// safe `CommandExt::exec`), so the engine's exit status is the invocation's
/// and the streams and signals are the engine's own. Returns only on failure.
#[cfg(unix)]
pub fn run(engine: &Path, args: &[OsString]) -> Res<u8> {
    use std::io;
    use std::os::unix::process::CommandExt;
    use std::time::Duration;
    let mut last: Option<io::Error> = None;
    for _ in 0..20 {
        let err = command(engine, args).exec();
        if err.kind() == io::ErrorKind::ExecutableFileBusy {
            last = Some(err);
            std::thread::sleep(Duration::from_millis(25));
        } else {
            last = Some(err);
            break;
        }
    }
    Err(Failure::io(format!(
        "cannot execute {}: {}",
        engine.display(),
        last.map_or_else(String::new, |e| e.to_string())
    )))
}

/// Elsewhere the launcher waits and exits with the engine's status. A console
/// interrupt reaches the engine directly, being delivered to the whole process
/// group, and the launcher waits for it rather than dying first.
#[cfg(not(unix))]
pub fn run(engine: &Path, args: &[OsString]) -> Res<u8> {
    let status = command(engine, args)
        .status()
        .map_err(|e| Failure::io(format!("cannot execute {}: {e}", engine.display())))?;
    Ok(status
        .code()
        .and_then(|c| u8::try_from(c & 0xff).ok())
        .unwrap_or(4))
}
