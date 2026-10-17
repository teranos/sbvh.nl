// tonic::Status is 176 bytes and appears in every RPC's Err variant. Boxing
// the framework's own error type to satisfy the lint would obscure every
// signature in the file for no benefit.
#![allow(clippy::result_large_err)]

use clap::Parser;
use qntx_grpc::plugin::proto::domain_plugin_service_server::DomainPluginServiceServer;
use qntx_grpc::plugin::shutdown_signal;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tracing::{info, warn, Level};
use tracing_subscriber::prelude::*;

mod attest;
mod mail;
mod meet;
mod openapi;
mod service;
mod telemetry;

use service::SbvhPluginService;

#[derive(Parser, Debug)]
// The node reads `--version` as qntx-<name>-plugin <version> after building it.
#[command(name = "qntx-sbvhAPI-plugin")]
#[command(about = "QNTX plugin — booking a call with SBVH")]
#[command(version)]
struct Args {
    /// gRPC server port
    #[arg(short, long, default_value = "9070")]
    port: u16,

    /// gRPC server address (overrides port)
    #[arg(short, long)]
    address: Option<String>,

    /// Log level (debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

const MAX_PORT_RETRIES: u16 = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let log_level = match args.log_level.as_str() {
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    // The Sentry layer is installed now and bound to a client at Initialize,
    // when the DSN arrives in the plugin's config. Before that it has nowhere
    // to send and drops what it is handed.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::from_level(log_level)),
        )
        .with(telemetry::layer())
        .init();

    info!("Initializing sbvhAPI");
    info!("  Version: {}", env!("CARGO_PKG_VERSION"));

    let listener = if let Some(address) = args.address {
        let addr: SocketAddr = address
            .parse()
            .map_err(|e| format!("Invalid address '{}': {}", address, e))?;
        TcpListener::bind(addr).await?
    } else {
        bind_with_retry(args.port).await?
    };

    let addr = listener.local_addr()?;
    info!("gRPC server listening on {}", addr);

    // The loader reads this line off stdout to learn which port was actually
    // bound — see pluginLogger.Write in plugin/grpc/discovery.go. Without it
    // QNTX assumes the port it asked for and connects to nothing.
    println!("QNTX_PLUGIN_PORT={}", addr.port());

    let service = SbvhPluginService::new();

    Server::builder()
        .add_service(DomainPluginServiceServer::new(service))
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown_signal())
        .await?;

    info!("sbvhAPI shutdown complete");
    telemetry::flush();
    Ok(())
}

/// The loader probes for a free port before launching, but two plugins starting
/// at once can still race for the same one. Walking up on AddrInUse and
/// announcing the result over stdout is what makes that survivable.
async fn bind_with_retry(mut port: u16) -> Result<TcpListener, Box<dyn std::error::Error>> {
    for attempt in 0..MAX_PORT_RETRIES {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        match TcpListener::bind(addr).await {
            Ok(listener) => {
                if attempt > 0 {
                    warn!("Bound to port {} after {} retries", port, attempt);
                }
                return Ok(listener);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                warn!(
                    "Port {} in use, trying {} (retry {}/{})",
                    port,
                    port + 1,
                    attempt + 1,
                    MAX_PORT_RETRIES
                );
                port += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(format!("Failed to bind after {} retries", MAX_PORT_RETRIES).into())
}
