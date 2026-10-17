//! Where the log goes when nobody is on the box.
//!
//! The node keeps a plugin's lines in a buffer and a file, and ships none of
//! them. So sbvhAPI ships its own, the way capy does: everything it logs from
//! info up is sent as a log line as well as written, and an error is raised
//! as an issue (CDR-005).
//!
//! The DSN is an ingest key. It arrives in the plugin's config section the
//! way github_token does, and a config without one ships nothing.

use std::time::Duration;

use sentry::integrations::tracing::EventFilter;
use tracing::Subscriber;
use tracing_subscriber::registry::LookupSpan;

/// The config key the DSN is read from, and the one naming the environment.
pub const DSN_KEY: &str = "sentry_dsn";
pub const ENVIRONMENT_KEY: &str = "sentry_environment";

/// The tracing layer. Installed once at startup; until `bind` runs it has no
/// client and drops what it sees.
///
/// An error is an issue. Everything from info up is a log line, and also a
/// breadcrumb, so the issue an error raises carries what led to it.
pub fn layer<S>() -> sentry::integrations::tracing::SentryLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    sentry::integrations::tracing::layer().event_filter(|md| match *md.level() {
        tracing::Level::ERROR => EventFilter::Event | EventFilter::Log,
        tracing::Level::WARN | tracing::Level::INFO => EventFilter::Breadcrumb | EventFilter::Log,
        _ => EventFilter::Ignore,
    })
}

/// Bind a client to the layer. The release is cleanAPI at this crate's
/// version, so an issue says which build raised it.
///
/// The guard is what keeps the client alive; drop it and the pipe closes.
pub fn bind(dsn: &str, environment: Option<&str>) -> sentry::ClientInitGuard {
    let release = format!("{}@{}", crate::service::PLUGIN, env!("CARGO_PKG_VERSION"));
    let environment = environment.unwrap_or("production").to_string();
    let mut options = sentry::ClientOptions::default();
    options.release = Some(release.clone().into());
    options.environment = Some(environment.clone().into());
    options.server_name = hostname().map(Into::into);
    options.attach_stacktrace = true;
    // Deprecated in name only: the tracing integration still reads it, and
    // without it the Log filter above goes nowhere.
    #[allow(deprecated)]
    {
        options.enable_logs = true;
    }
    let guard = sentry::init((dsn, options));
    tracing::info!(release, environment, "Shipping logs to Sentry");
    guard
}

/// Drain what is queued. Sentry sends in batches on its own clock, and a
/// process that ends before the batch goes loses the lines that say why.
pub fn flush() {
    if let Some(client) = sentry::Hub::current().client() {
        client.close(Some(Duration::from_secs(2)));
    }
}

fn hostname() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
