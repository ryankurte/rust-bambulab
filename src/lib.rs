//! See: https://github.com/greghesp/ha-bambulab/tree/main/custom_components/bambu_lab

use clap::Parser;
use futures::{Stream, StreamExt};
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Error as MqttError, SslOptionsBuilder,
};
use rustls::client::ServerCertVerifier;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    task::JoinHandle,
};
use tracing::{debug, trace};

pub mod level;
pub mod types;

mod printer;
pub use printer::Printer;

mod error;
pub use error::Error;

/// Options for printer connection
#[derive(Clone, Debug, PartialEq, Parser)]
pub struct ConnectOpts {
    /// Hostname or IP address
    #[clap(short = 'n', long, env)]
    pub hostname: String,

    /// MQTT Port
    #[clap(short, long, default_value = "8883")]
    pub port: u16,

    /// Access code (see local connection page on printer)
    #[clap(long, env)]
    pub access_code: String,
}

impl Default for ConnectOpts {
    fn default() -> Self {
        Self {
            hostname: Default::default(),
            port: 8883,
            access_code: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {}
