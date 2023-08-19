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

/// Options for printer connection
#[derive(Clone, Debug, PartialEq, Parser)]
pub struct ConnectOpts {
    /// Hostname or IP address
    #[clap(short = 'n', long)]
    pub hostname: String,

    /// MQTT Port
    #[clap(short, long, default_value = "8883")]
    pub port: u16,

    /// Access code (see local connection page on printer)
    #[clap(long)]
    pub access_code: String,
}

impl Default for ConnectOpts {
    fn default() -> Self {
        Self { hostname: Default::default(), port: 8883, access_code: Default::default() }
    }
}

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum Error {
    /// MQTT error {0}
    Mqtt(MqttError),
}

impl From<MqttError> for Error {
    fn from(value: MqttError) -> Self {
        Self::Mqtt(value)
    }
}

/// Handle for connected printer
pub struct Printer {
    opts: ConnectOpts,
    client: AsyncClient,
    rx: UnboundedReceiver<(String, Vec<u8>)>,
    _h: JoinHandle<()>,
}

impl std::fmt::Debug for Printer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "printer")
    }
}

pub struct NullTlsVerifier;

impl ServerCertVerifier for NullTlsVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        debug!("cert: {:?}", end_entity);

        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

const BAMBU_ROOT: &str = "
-----BEGIN CERTIFICATE-----
MIIDZTCCAk2gAwIBAgIUV1FckwXElyek1onFnQ9kL7Bk4N8wDQYJKoZIhvcNAQEL
BQAwQjELMAkGA1UEBhMCQ04xIjAgBgNVBAoMGUJCTCBUZWNobm9sb2dpZXMgQ28u
LCBMdGQxDzANBgNVBAMMBkJCTCBDQTAeFw0yMjA0MDQwMzQyMTFaFw0zMjA0MDEw
MzQyMTFaMEIxCzAJBgNVBAYTAkNOMSIwIAYDVQQKDBlCQkwgVGVjaG5vbG9naWVz
IENvLiwgTHRkMQ8wDQYDVQQDDAZCQkwgQ0EwggEiMA0GCSqGSIb3DQEBAQUAA4IB
DwAwggEKAoIBAQDL3pnDdxGOk5Z6vugiT4dpM0ju+3Xatxz09UY7mbj4tkIdby4H
oeEdiYSZjc5LJngJuCHwtEbBJt1BriRdSVrF6M9D2UaBDyamEo0dxwSaVxZiDVWC
eeCPdELpFZdEhSNTaT4O7zgvcnFsfHMa/0vMAkvE7i0qp3mjEzYLfz60axcDoJLk
p7n6xKXI+cJbA4IlToFjpSldPmC+ynOo7YAOsXt7AYKY6Glz0BwUVzSJxU+/+VFy
/QrmYGNwlrQtdREHeRi0SNK32x1+bOndfJP0sojuIrDjKsdCLye5CSZIvqnbowwW
1jRwZgTBR29Zp2nzCoxJYcU9TSQp/4KZuWNVAgMBAAGjUzBRMB0GA1UdDgQWBBSP
NEJo3GdOj8QinsV8SeWr3US+HjAfBgNVHSMEGDAWgBSPNEJo3GdOj8QinsV8SeWr
3US+HjAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBAQABlBIT5ZeG
fgcK1LOh1CN9sTzxMCLbtTPFF1NGGA13mApu6j1h5YELbSKcUqfXzMnVeAb06Htu
3CoCoe+wj7LONTFO++vBm2/if6Jt/DUw1CAEcNyqeh6ES0NX8LJRVSe0qdTxPJuA
BdOoo96iX89rRPoxeed1cpq5hZwbeka3+CJGV76itWp35Up5rmmUqrlyQOr/Wax6
itosIzG0MfhgUzU51A2P/hSnD3NDMXv+wUY/AvqgIL7u7fbDKnku1GzEKIkfH8hm
Rs6d8SCU89xyrwzQ0PR853irHas3WrHVqab3P+qNwR0YirL0Qk7Xt/q3O1griNg2
Blbjg3obpHo9
-----END CERTIFICATE-----
";

impl Printer {
    /// Connect to a printer via MQTT
    pub async fn connect(opts: ConnectOpts) -> Result<Self, Error> {
        debug!("Connecting to: {opts:?}");

        // Setup MQTT connection
        let mqtt_options = CreateOptionsBuilder::new()
            .server_uri(format!("mqtts://{}:{}", opts.hostname, opts.port))
            .persistence(paho_mqtt::PersistenceType::None)
            .finalize();

        let mut client = AsyncClient::new(mqtt_options)?;

        let tls_config = SslOptionsBuilder::new()
            .verify(false)
            .enable_server_cert_auth(false)
            .finalize();

        let connect_opts = ConnectOptionsBuilder::new()
            .ssl_options(tls_config)
            .user_name("bblp")
            .password(opts.access_code.clone())
            .finalize();

        // Setup MQTT connection
        debug!("Connecting");
        client.connect(connect_opts).await?;

        // Setup subscriber task
        let mut mqtt_rx = client.get_stream(1000);
        let (tx, rx) = unbounded_channel::<(String, Vec<u8>)>();

        // Start listener
        let _h = tokio::task::spawn(async move {
            debug!("Start connection task");

            loop {
                match mqtt_rx.next().await {
                    Some(Some(v)) => {
                        trace!("rx {}: {:02x?}", v.topic(), v.payload());
                        if let Err(e) = tx.send((v.topic().to_string(), v.payload().to_vec())) {
                            println!("Failed to forward incoming packet: {e:?}");
                        }
                    }
                    _ => break,
                }
            }

            debug!("Exit connection task");
        });

        debug!("Subscribing to topic");
        // Subscribe to topic
        client.subscribe("#", 2).await?;

        Ok(Self { opts, client, rx, _h })
    }

    pub async fn disconnect(self) -> Result<(), Error> {
        // Disconnect from broker
        self.client.disconnect(None).await?;

        Ok(())
    }
}

/// [Stream] implementation, pollable for parsed printer events
impl Stream for Printer {
    type Item = (String, Vec<u8>);

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

impl std::hash::Hash for Printer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.opts.hostname.hash(state);
        self.opts.port.hash(state);
    }
}

#[cfg(test)]
mod tests {
    
}
