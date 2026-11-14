use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use dtnpm_core::{BpaClient, GrpcBpaClient, resolve_grpc_port};
use template_dtnpm_cla::{ClaConfig, TemplateCla};
use tracing::{info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "template-dtnpm-cla",
    about = "Standalone Convergence Layer Adapter (CLA) daemon for Hardy BPA"
)]
struct Args {
    /// gRPC endpoint of the target BPA (e.g. http://127.0.0.1:50051)
    #[arg(long)]
    bpa_grpc: Option<String>,

    /// Name used to register this CLA with the BPA
    #[arg(long, default_value = "template-cla")]
    name: String,

    /// Optional YAML or TOML configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Optional Bearer authentication token (deprecated for security: prefer --token-file)
    #[arg(long, env = "DTN_HDY_TOKEN")]
    token: Option<String>,

    /// Path to a file containing the Bearer authentication token
    #[arg(long, env = "DTN_HDY_TOKEN_FILE")]
    token_file: Option<PathBuf>,

    /// Optional Unix Domain Socket path for connecting to a protected local BPA
    #[arg(long, env = "DTN_HDY_SOCKET")]
    socket: Option<String>,

    /// Explicit opt-in flag to allow unauthenticated or unsigned traffic (violates strict RFC 9172)
    #[arg(
        long,
        env = "DTN_HDY_ALLOW_UNAUTHENTICATED_BPS",
        default_value = "false"
    )]
    allow_unauthenticated_bps_opt_in: bool,
}

fn load_config(path: Option<&Path>) -> anyhow::Result<ClaConfig> {
    let Some(path) = path else {
        return Ok(ClaConfig::default());
    };
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file {}", path.display()))?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => toml::from_str(&raw).context("Invalid TOML config"),
        _ => yaml_serde::from_str(&raw).context("Invalid YAML config"),
    }
}

fn resolve_bearer_token(
    inline_token: Option<String>,
    token_file: Option<&Path>,
) -> anyhow::Result<Option<String>> {
    if let Some(path) = token_file {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read token file {}: {e}", path.display()))?;
        let trimmed = content.trim().to_string();
        if trimmed.is_empty() {
            anyhow::bail!("Token file {} is empty", path.display());
        }
        return Ok(Some(trimmed));
    }

    if let Some(tok) = inline_token.as_ref().filter(|t| !t.trim().is_empty()) {
        warn!("Passing token via CLI argument is deprecated for security; prefer --token-file");
        return Ok(Some(tok.clone()));
    }

    Ok(None)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    if args.allow_unauthenticated_bps_opt_in {
        warn!(
            "SECURITY RISK: --allow-unauthenticated-bps-opt-in is active; unauthenticated traffic will be accepted!"
        );
    }

    let config = load_config(args.config.as_deref())?;
    config
        .validate()
        .map_err(|e| anyhow::anyhow!("Configuration validation failed: {e}"))?;

    let token = resolve_bearer_token(args.token, args.token_file.as_deref())?;

    let bpa_endpoint = match args.bpa_grpc {
        Some(ep) => ep,
        None => {
            let port = resolve_grpc_port(None);
            format!("http://127.0.0.1:{port}")
        }
    };

    info!(
        endpoint = %bpa_endpoint,
        socket = ?args.socket,
        cla_name = %args.name,
        "Connecting CLA to BPA daemon"
    );

    let grpc_client = Arc::new(GrpcBpaClient::with_auth(
        bpa_endpoint,
        token.as_deref(),
        args.socket.as_deref(),
    ));

    let cla = Arc::new(TemplateCla::new(config));
    let cla_init = cla.cla_init();

    let bpa_reg = grpc_client.as_registration();
    bpa_reg
        .register_cla(args.name.clone(), cla.clone(), None, cla_init)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to register CLA with BPA: {e}"))?;

    info!(cla_name = %args.name, "CLA registered successfully with BPA");
    info!("Running. Press Ctrl+C to terminate.");

    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received, unregistering CLA...");
    cla.unregister().await;
    info!("CLA cleanly unregistered and shut down.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::resolve_bearer_token;

    #[test]
    fn test_resolve_bearer_token_from_file() {
        let tmp = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(tmp.path(), "  secret-token-from-file \n").expect("write token");
        let token = resolve_bearer_token(None, Some(tmp.path())).expect("token resolved");
        assert_eq!(token.as_deref(), Some("secret-token-from-file"));
    }

    #[test]
    fn test_resolve_bearer_token_rejects_empty_file() {
        let tmp = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(tmp.path(), "   \n").expect("write empty token");
        let res = resolve_bearer_token(None, Some(tmp.path()));
        assert!(res.is_err());
    }
}
