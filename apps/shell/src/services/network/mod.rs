//! Network and radios: connections, airplane mode and the tailnet.

pub mod connections;
pub mod rfkill;
pub mod tailscale;
mod tailscale_local;

use amane::{ErrorKind, ServiceError};

/// A few words for why joining a network failed, as Quick Settings' row
/// and the shell's notification both say it.
pub fn failure_label(failure: &ServiceError) -> &'static str {
    match failure.kind {
        ErrorKind::AuthenticationFailed => "Wrong password",
        ErrorKind::Unreachable | ErrorKind::Gone => "Not in range",
        ErrorKind::TimedOut => "Timed out",
        _ => "Couldn't join",
    }
}
