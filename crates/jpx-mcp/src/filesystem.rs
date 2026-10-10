//! Filesystem access policy for MCP file tools.

use std::error::Error as StdError;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Effective access mode for [`FileAccessPolicy`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileAccessMode {
    /// File tools cannot read from the filesystem.
    Disabled,
    /// File tools may read any path allowed by the server process.
    Unrestricted,
    /// File tools may read only beneath configured canonical roots.
    AllowedRoots,
}

impl FileAccessMode {
    /// Stable name used by `engine_info` and diagnostics.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Unrestricted => "unrestricted",
            Self::AllowedRoots => "allowed_roots",
        }
    }
}

#[derive(Clone, Debug)]
enum FileAccessRules {
    Disabled,
    Unrestricted,
    AllowedRoots(Arc<[PathBuf]>),
}

/// Controls which paths MCP file tools may read.
///
/// The existing router constructors use [`FileAccessPolicy::unrestricted`] for
/// backward compatibility. Transport wrappers should select a safer default
/// where appropriate; the `jpx-mcp` binary disables file access for HTTP unless
/// at least one allowed root is supplied.
#[derive(Clone, Debug)]
pub struct FileAccessPolicy {
    rules: FileAccessRules,
}

impl FileAccessPolicy {
    /// Disable all filesystem reads by MCP file tools.
    pub fn disabled() -> Self {
        Self {
            rules: FileAccessRules::Disabled,
        }
    }

    /// Permit any readable absolute path visible to the server process.
    ///
    /// This preserves the historical behavior and is intended for trusted
    /// local transports such as stdio.
    pub fn unrestricted() -> Self {
        Self {
            rules: FileAccessRules::Unrestricted,
        }
    }

    /// Restrict filesystem reads to files beneath the supplied directories.
    ///
    /// Roots are canonicalized immediately, so relative roots are resolved
    /// against the process working directory and symlink roots resolve to their
    /// actual target. Every root must already exist and be a directory.
    pub fn restricted<I, P>(roots: I) -> Result<Self, FileAccessPolicyError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let mut canonical_roots = Vec::new();

        for root in roots {
            let root = root.as_ref();
            let canonical = root.canonicalize().map_err(|error| {
                FileAccessPolicyError::new(format!(
                    "Cannot use allowed root '{}': {error}. Check that the directory exists and is accessible.",
                    root.display()
                ))
            })?;

            if !canonical.is_dir() {
                return Err(FileAccessPolicyError::new(format!(
                    "Allowed root '{}' is not a directory. Pass a directory path to '--allow-root <DIRECTORY>'.",
                    canonical.display()
                )));
            }

            canonical_roots.push(canonical);
        }

        canonical_roots.sort();
        canonical_roots.dedup();

        if canonical_roots.is_empty() {
            return Err(FileAccessPolicyError::new(
                "No allowed roots were provided. Pass at least one directory, or use FileAccessPolicy::disabled() or FileAccessPolicy::unrestricted().",
            ));
        }

        Ok(Self {
            rules: FileAccessRules::AllowedRoots(canonical_roots.into()),
        })
    }

    /// Return the effective access mode.
    pub fn mode(&self) -> FileAccessMode {
        match self.rules {
            FileAccessRules::Disabled => FileAccessMode::Disabled,
            FileAccessRules::Unrestricted => FileAccessMode::Unrestricted,
            FileAccessRules::AllowedRoots(_) => FileAccessMode::AllowedRoots,
        }
    }

    /// Return the canonical allowed roots, or an empty slice for other modes.
    pub fn allowed_roots(&self) -> &[PathBuf] {
        match &self.rules {
            FileAccessRules::AllowedRoots(roots) => roots,
            FileAccessRules::Disabled | FileAccessRules::Unrestricted => &[],
        }
    }

    pub(crate) fn resolve_file(&self, requested: &Path) -> Result<PathBuf, String> {
        if matches!(self.rules, FileAccessRules::Disabled) {
            return Err(
                "File access is disabled for this server. Restart jpx-mcp with '--allow-root <DIRECTORY>' (repeatable) to enable evaluate_file for specific directories."
                    .to_string(),
            );
        }

        if !requested.is_absolute() {
            return Err(
                "File path must be absolute. Pass an absolute path beneath an allowed root, such as '/data/input.json'."
                    .to_string(),
            );
        }

        let canonical = requested.canonicalize().map_err(|error| {
            format!(
                "Cannot resolve file path '{}': {error}. Check that the file exists and every parent directory is accessible.",
                requested.display()
            )
        })?;

        if let FileAccessRules::AllowedRoots(roots) = &self.rules
            && !roots.iter().any(|root| canonical.starts_with(root))
        {
            let roots = roots
                .iter()
                .map(|root| root.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "File '{}' is outside the configured allowed roots: {roots}. Choose a file beneath an allowed root or restart jpx-mcp with an additional '--allow-root <DIRECTORY>'.",
                canonical.display()
            ));
        }

        Ok(canonical)
    }
}

impl Default for FileAccessPolicy {
    fn default() -> Self {
        Self::unrestricted()
    }
}

/// Error returned when an allowed-root policy cannot be constructed.
#[derive(Debug)]
pub struct FileAccessPolicyError {
    message: String,
}

impl FileAccessPolicyError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FileAccessPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for FileAccessPolicyError {}
