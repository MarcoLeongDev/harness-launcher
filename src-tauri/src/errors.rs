//! Typed application errors with stable machine-readable codes.
//!
//! IPC boundaries keep speaking `Result<T, String>` (Tauri serializes the
//! `Err` as a plain string for the panels), but fallible helpers SHOULD
//! return `Result<T, AppError>` so Rust can match on [`AppError::code`]
//! instead of substring-sniffing English text. `Display` renders the
//! user-safe message (the same wording the stringly errors used, so console
//! output is unchanged); user files and secrets never appear in variants.
use thiserror::Error;

/// Machine-readable failure taxonomy for launcher operations.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum AppError {
    /// Caller-supplied port is outside 1..=65535.
    #[error("port must be between 1 and 65535")]
    InvalidPort,
    /// No free loopback port in the scanned window.
    #[error("no free port found near {port}")]
    NoFreePort { port: u16 },
    /// Caller-supplied version string failed [`crate::versions::is_valid_version_name`].
    #[error("invalid version name: {name}")]
    InvalidVersion { name: String },
}

impl AppError {
    /// Stable kebab-case code for logs and future localized mappings.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::InvalidPort => "invalid-port",
            AppError::NoFreePort { .. } => "no-free-port",
            AppError::InvalidVersion { .. } => "invalid-version",
        }
    }
}

/// IPC boundary conversion: typed errors cross into Tauri commands as
/// their user-safe message, unchanged from the legacy strings.
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::AppError;

    #[test]
    fn codes_are_stable() {
        assert_eq!(AppError::InvalidPort.code(), "invalid-port");
        assert_eq!(AppError::NoFreePort { port: 3080 }.code(), "no-free-port");
        assert_eq!(
            AppError::InvalidVersion {
                name: "../evil".into()
            }
            .code(),
            "invalid-version"
        );
    }

    #[test]
    fn display_matches_legacy_strings() {
        assert_eq!(
            AppError::InvalidPort.to_string(),
            "port must be between 1 and 65535"
        );
        assert_eq!(
            AppError::NoFreePort { port: 3080 }.to_string(),
            "no free port found near 3080"
        );
        assert_eq!(
            AppError::InvalidVersion {
                name: "../evil".into()
            }
            .to_string(),
            "invalid version name: ../evil"
        );
    }

    #[test]
    fn converts_into_ipc_string() {
        let message: String = AppError::InvalidPort.into();
        assert_eq!(message, "port must be between 1 and 65535");
    }
}
