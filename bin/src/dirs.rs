use std::{
    io,
    path::{Path, PathBuf},
};

use ignore::{overrides::OverrideBuilder, WalkBuilder};

/// Walks through target paths and returns an iterator of .nix files, respecting gitignore rules.
pub fn walk_nix_files<P: AsRef<Path>>(
    ignore_patterns: Vec<String>,
    targets: &[P],
    unrestricted: bool,
) -> Result<impl Iterator<Item = PathBuf>, io::Error> {
    let walker = build_walker(targets, &ignore_patterns, unrestricted)?;

    Ok(walker
        .filter_map(|result| result.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file() && matches!(path.extension(), Some(ext) if ext == "nix")
        })
        .map(|entry| entry.path().to_path_buf()))
}

/// Creates a single `ignore::Walk` iterator for multiple targets with custom ignore patterns.
fn build_walker<P: AsRef<Path>>(
    targets: &[P],
    ignore_patterns: &[String],
    unrestricted: bool,
) -> Result<ignore::Walk, io::Error> {
    let mut targets_iter = targets.iter();

    let first_target = targets_iter
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No targets provided"))?;

    let mut builder = WalkBuilder::new(first_target.as_ref());

    for target in targets_iter {
        builder.add(target.as_ref());
    }

    builder.git_ignore(!unrestricted);

    // Add files/directories to ignore set passed in --ignore
    if !ignore_patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new("");

        for pattern in ignore_patterns {
            override_builder
                .add(&format!("!{}", pattern))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        }

        let overrides = override_builder
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        builder.overrides(overrides);
    }

    Ok(builder.build())
}
