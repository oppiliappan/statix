use std::{
    io,
    path::{Path, PathBuf},
};

use ignore::{overrides::OverrideBuilder, WalkBuilder};

/// Walks through target paths and returns an iterator of .nix files, respecting gitignore rules.
///
/// # Arguments
/// * `ignore_patterns` - Globs of file patterns to skip (e.g., "*.tmp", "build/*")
/// * `targets` - File or directory paths to walk through
/// * `unrestricted` - If true, don't respect .gitignore files; if false, respect them
pub fn walk_nix_files<P: AsRef<Path>>(
    ignore_patterns: Vec<String>,
    targets: &[P],
    unrestricted: bool,
) -> Result<impl Iterator<Item = PathBuf>, io::Error> {
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

        for pattern in &ignore_patterns {
            // Note: The `!` prefix has inverted semantics in OverrideBuilder compared to gitignore.
            // In OverrideBuilder: `!pattern` means "ignore files matching pattern"
            // In gitignore: `!pattern` means "don't ignore files matching pattern" (whitelist)
            // So we add `!` to make ignore_patterns actually ignore files.
            override_builder
                .add(&format!("!{}", pattern))
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        }

        let overrides = override_builder
            .build()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        builder.overrides(overrides);
    }

    let walker = builder.build();

    Ok(walker
        .filter_map(|result| match result {
            Ok(entry) => Some(entry),
            Err(err) => {
                eprintln!("Warning: Error reading directory entry: {}", err);
                None
            }
        })
        .filter_map(|entry| {
            let path = entry.path();
            (path.is_file() && matches!(path.extension(), Some(ext) if ext == "nix"))
                .then(|| path.to_path_buf())
        }))
}
