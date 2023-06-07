

use clap::Parser;
use futures::StreamExt;
use tracing::{debug, info, trace, warn};
use tracing_subscriber::{filter::LevelFilter, EnvFilter, FmtSubscriber};

use bambu::{Printer, ConnectOpts, types::Report};

/// Bambu 3d printer MQTT command line connector
#[derive(Clone, Debug, PartialEq, Parser)]
pub struct Args {
    #[clap(flatten)]
    opts: ConnectOpts,
    
    /// Enable verbose logging
    #[clap(long, default_value = "debug")]
    log_level: LevelFilter,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load arguments
    let args = Args::parse();

    // Setup logging
    let filter = EnvFilter::from_default_env()
        .add_directive("paho_mqtt=warn".parse().unwrap())
        .add_directive(args.log_level.into());

    let _ = FmtSubscriber::builder()
        .compact()
        .without_time()
        .with_max_level(args.log_level)
        .with_env_filter(filter)
        .try_init();

    debug!("Connecting to {}:{}", args.opts.hostname, args.opts.port);

    // Establish printer connection
    let mut p = Printer::connect(args.opts).await?;

    debug!("Connected!");

    // Listen for messages
    loop {
        if let Some((topic, data)) = p.next().await {

            // Decode to text
            let text = match std::str::from_utf8(&data) {
                Ok(v) => v,
                Err(_e) => {
                    warn!("Non-text object on topic {topic}: {data:02x?}");
                    continue;
                }
            };

            let v: Report = match serde_json::from_slice(&data) {
                Ok(v) => v,
                Err(_e) => {
                    warn!("Failed to parse object on topic {topic}: {text:02x?}");
                    continue;
                }
            };

            debug!("RX {topic}: {v:?}");
        }
    }

    Ok(())
}
