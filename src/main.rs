

use std::{io::Write, str::FromStr};

use clap::Parser;
use futures::StreamExt;
use tracing::{debug, info, trace, warn};
use tracing_subscriber::{filter::LevelFilter, EnvFilter, FmtSubscriber};

use bambu::{Printer, ConnectOpts, types::{Report, McPrintCommand, McPrintValue}};

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
    Log{
        /// Output file for writing received objects
        #[clap(long)]
        file: Option<String>,
    },
    /// Parse an existing log file
    Parse{
        /// Log file for parsing
        #[clap(long)]
        file: String,
    }
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
        Commands::Log{ file } => match file {
            Some(o) => Some(std::fs::File::create(o)?),
            _ => None,
        },
        Commands::Parse { file } => {
            let d = std::fs::read(&file)?;

            info!("Loaded {} bytes", d.len());

            let o: Vec<Report> = serde_json::from_slice(&d)?;

            info!("Parsed {} objects", o.len());

            let r: Vec<_> = o.iter()
            .filter_map(|i| match i {
                Report::McPrint { command, param, .. } if command == &McPrintCommand::PushInfo => Some(param.to_string()),
                _ => None,
            })
            .filter_map(|p| McPrintValue::from_str(&p).ok() )
            .filter_map(|v| match v {
                McPrintValue::BmcMeas { x, y, z_c, z_d } => Some((x, y, z_c, z_d)),
                _ => None,
            })
            .collect();

            let mut xs = Vec::<f32>::new();
            let mut ys = Vec::<f32>::new();

            for (x, y, ..) in r.iter() {
                if xs.iter().find(|v| v == &x).is_none() {
                    xs.push(*x);
                }

                if ys.iter().find(|v| v == &y).is_none() {
                    ys.push(*y);
                }
            }

            // Print level data
            print!("        ");
            for x in &xs {
                print!("{x:<7.01}");
            }
            println!("");

            for y in &ys {
                print!("{y:>5.01}: ");

                for x in &xs {
                    let v = r.iter().find(|v| v.0 == *x && v.1 == *y);

                    match v {
                        Some(v) => print!("{:>6.03} ", v.2),
                        None => print!("????? "),
                    }
                }

                println!("");
            }

            return Ok(())
        }
        _ => None,
    };

    debug!("Connecting to {}:{}", args.opts.hostname, args.opts.port);

    // Establish printer connection
    let mut p = Printer::connect(args.opts).await?;

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

            // Write to log if enabled
            if let Some(f) = &mut f {
                f.write(text.as_bytes())?;
            }
        }
    }

    Ok(())
}
