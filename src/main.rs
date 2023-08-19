use std::{io::Write, str::FromStr};

use clap::Parser;
use futures::StreamExt;
use tracing::{debug, info, warn};
use tracing_subscriber::{filter::LevelFilter, EnvFilter, FmtSubscriber};

use bambu::{
    level::{LevelMap, Point},
    types::{McPrintCommand, McPrintValue, Report},
    ConnectOpts, Printer,
};

/// Bambu 3d printer MQTT command line connector
#[derive(Clone, Debug, PartialEq, Parser)]
pub struct Args {
    #[clap(flatten)]
    opts: ConnectOpts,

    #[clap(subcommand)]
    cmd: Commands,

    /// Enable verbose logging
    #[clap(long, default_value = "debug")]
    log_level: LevelFilter,
}

#[derive(Clone, Debug, PartialEq, Parser)]
pub enum Commands {
    /// Connect to the printer and log incoming messages
    Log {
        /// Output file for writing received objects
        #[clap(long)]
        file: Option<String>,
    },
    /// Parse an existing log file
    Parse {
        /// Log file for parsing
        #[clap(long)]
        file: String,
    },
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

    debug!("Connected!");

    // Setup log file
    let mut f = match args.cmd {
        Commands::Log { file } => match file {
            Some(o) => Some(std::fs::File::create(o)?),
            _ => None,
        },
        Commands::Parse { file } => {
            let d = std::fs::read(file)?;

            info!("Loaded {} bytes", d.len());

            let o: Vec<Report> = serde_json::from_slice(&d)?;

            info!("Parsed {} objects", o.len());

            let r: Vec<_> = o
                .iter()
                .filter_map(|i| match i {
                    Report::McPrint { command, param, .. }
                        if command == &McPrintCommand::PushInfo =>
                    {
                        Some(param.to_string())
                    }
                    _ => None,
                })
                .filter_map(|p| McPrintValue::from_str(&p).ok())
                .filter_map(|v| match v {
                    McPrintValue::BmcMeas { x, y, z_c, z_d } => Some(Point::new(x, y, z_c, z_d)),
                    _ => None,
                })
                .collect();

            let l = LevelMap::new(r);
            println!("{l}");

            return Ok(());
        }
        _ => None,
    };

    debug!("Connecting to {}:{}", args.opts.hostname, args.opts.port);

    // Establish printer connection
    let p = Printer::connect(args.opts).await?;
    let mut l = p.listen()?;

    // Listen for messages
    loop {
        if let Some((topic, data)) = l.recv().await {
            // Decode to object
            let v: Report = match serde_json::from_slice(&data.as_bytes()) {
                Ok(v) => v,
                Err(_e) => {
                    warn!("Failed to parse object on topic {topic}: {data:02x?}");
                    continue;
                }
            };

            debug!("RX {topic}: {v:?}");

            // Write to log if enabled
            if let Some(f) = &mut f {
                f.write(data.as_bytes())?;
            }
        }
    }

    Ok(())
}
