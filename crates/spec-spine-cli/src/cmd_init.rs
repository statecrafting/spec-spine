//! `spec-spine init`: scaffold a new adopter (spec 006).
//!
//! Core returns the files as data ([`spec_spine_core::scaffold_init`]); this is
//! where they are written. Without `--force`, a pre-existing file is skipped (not
//! an error; `init` is idempotent); with `--force`, every file is overwritten.

use std::fs;
use std::path::Path;

use spec_spine_core::scaffold_init_with;
use spec_spine_types::Error;

use crate::load_repo_config;

pub fn run(repo: &Path, force: bool, with_kit: bool) -> Result<u8, Error> {
    let cfg = load_repo_config(repo)?;
    let scaffold = scaffold_init_with(&cfg, with_kit)?;
    let specs_dir = cfg.layout.specs_dir.trim_end_matches('/').to_string();

    let mut written = 0usize;
    let mut skipped = 0usize;

    for file in &scaffold.files {
        let abs = repo.join(&file.rel_path);

        // Spec 074 3.5: `init` must not create a corpus its own compiler
        // refuses. Writing `specs/000-bootstrap` beside an existing
        // `specs/000-anything` reported "0 skipped" and left a tree the next
        // `compile` rejects with `V-004 numeric prefix '000' is shared by ...`.
        // The check is on the ORDINAL, not the directory name, because the
        // ordinal is what V-004 collides on.
        //
        // Skipping rather than refusing: `init` is additive by design and
        // already reports skipped files, and a repository that has a bootstrap
        // spec and runs `--with-kit` for the harness is doing something
        // legitimate. Refusing would make the kit uninstallable in exactly the
        // repositories most likely to want it.
        if let Some((dir, ordinal)) = scaffolded_spec_ordinal(&specs_dir, &file.rel_path)
            && let Some(existing) = sibling_with_ordinal(repo, &specs_dir, dir, ordinal)
        {
            outln!(
                "  skip (ordinal {ordinal} already used by {specs_dir}/{existing}): {}",
                file.rel_path
            );
            skipped += 1;
            continue;
        }

        if abs.exists() && !force && !file.overwrite {
            // Spec 074 3.3: an `append` file is reconciled rather than skipped.
            // The stanza binding the merge driver has to reach a repository
            // that already has a `.gitattributes`, which is most of them.
            if file.append {
                match append_block(&abs, file)? {
                    true => {
                        outln!("  append: {}", file.rel_path);
                        written += 1;
                    }
                    false => {
                        outln!("  skip (already present): {}", file.rel_path);
                        skipped += 1;
                    }
                }
                continue;
            }
            outln!("  skip (exists): {}", file.rel_path);
            skipped += 1;
            continue;
        }
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::Io(format!("create {}: {e}", parent.display())))?;
        }
        fs::write(&abs, &file.contents)
            .map_err(|e| Error::Io(format!("write {}: {e}", abs.display())))?;
        // Spec 074 3.4: the scaffold says whether a file is executable and the
        // writer applies it. Two shell scripts shipped at 644 while their own
        // documentation invoked them by path.
        if file.executable {
            set_executable(&abs)?;
        }
        outln!("  write: {}", file.rel_path);
        written += 1;
    }

    // Spec 074 3.5: do not point at a bootstrap spec this run declined to
    // write. Telling an adopter to customize a file `init` skipped is the same
    // class of defect as reporting "0 skipped" while writing a duplicate.
    let bootstrap = format!("{specs_dir}/000-bootstrap/spec.md");
    let next = if repo.join(&bootstrap).exists() {
        format!("Next: customize {bootstrap}, then run `spec-spine compile`.")
    } else {
        "Next: run `spec-spine compile`.".to_string()
    };
    outln!(
        "spec-spine init: {written} file(s) written, {skipped} skipped{}. {next}",
        if force { " (--force)" } else { "" }
    );
    Ok(0)
}

/// The spec directory a scaffolded path lives in and its ordinal, when the path
/// is a `spec.md` under the corpus root: `specs/000-bootstrap/spec.md` yields
/// `("000-bootstrap", "000")`.
fn scaffolded_spec_ordinal<'a>(specs_dir: &str, rel_path: &'a str) -> Option<(&'a str, &'a str)> {
    let rest = rel_path.strip_prefix(specs_dir)?.strip_prefix('/')?;
    let (dir, tail) = rest.split_once('/')?;
    if tail != "spec.md" {
        return None;
    }
    Some((dir, ordinal(dir)?))
}

/// The leading ASCII-digit run of an id, or `None` when it has none.
///
/// Taken by character rather than by byte: spec 070 fixed a panic where a
/// non-ASCII id was sliced at `[..3]`, and this is the same shape of read.
fn ordinal(id: &str) -> Option<&str> {
    let end = id
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(id.len());
    (end > 0).then(|| &id[..end])
}

/// Another spec directory already using `ordinal`, if one exists.
///
/// Reads the corpus root directly rather than compiling it: `init` runs in a
/// repository that may not compile yet, which is the whole reason it is being
/// run, so a read that needed a valid corpus would be unavailable exactly when
/// it is needed.
fn sibling_with_ordinal(
    repo: &Path,
    specs_dir: &str,
    scaffolded: &str,
    ordinal: &str,
) -> Option<String> {
    let entries = fs::read_dir(repo.join(specs_dir)).ok()?;
    let mut found: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name != scaffolded && crate::cmd_init::ordinal(name) == Some(ordinal))
        .collect();
    found.sort();
    found.into_iter().next()
}

/// Append a file's contents to an existing file, unless its marker is already
/// there. Returns whether anything was written.
///
/// Idempotent by marker rather than by whole-block comparison: an adopter who
/// reformats or comments the stanza must not receive a second copy, and the
/// driver name is the fact that matters.
fn append_block(abs: &Path, file: &spec_spine_core::scaffold::ScaffoldFile) -> Result<bool, Error> {
    let existing =
        fs::read_to_string(abs).map_err(|e| Error::Io(format!("read {}: {e}", abs.display())))?;
    let marker = file.append_marker.as_deref().unwrap_or(&file.contents);
    if existing.contains(marker) {
        return Ok(false);
    }
    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    if !next.is_empty() {
        next.push('\n');
    }
    next.push_str(&file.contents);
    fs::write(abs, next).map_err(|e| Error::Io(format!("append {}: {e}", abs.display())))?;
    Ok(true)
}

/// Give a written file the executable bit, where the platform has one.
///
/// Inert on Windows, which has no mode bits: the scaffold still carries the
/// flag as data, so the returned `Scaffold` is identical on every platform and
/// only the write differs.
#[cfg(unix)]
fn set_executable(abs: &Path) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(abs)
        .map_err(|e| Error::Io(format!("stat {}: {e}", abs.display())))?
        .permissions();
    let mode = perms.mode();
    // Mirror the read bits: a file readable by a class becomes executable by it,
    // so a restrictive umask is respected rather than overridden.
    perms.set_mode(mode | ((mode & 0o444) >> 2));
    fs::set_permissions(abs, perms).map_err(|e| Error::Io(format!("chmod {}: {e}", abs.display())))
}

#[cfg(not(unix))]
fn set_executable(_abs: &Path) -> Result<(), Error> {
    Ok(())
}
