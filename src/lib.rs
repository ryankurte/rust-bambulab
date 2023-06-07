

//! See: https://github.com/greghesp/ha-bambulab/tree/main/custom_components/bambu_lab

use std::{sync::Arc, task::Poll};

use rustls::{client::{ServerCertVerifier, ServerCertVerified}, ALL_VERSIONS, ALL_CIPHER_SUITES, ALL_KX_GROUPS};
use tracing::{debug, trace};
use clap::Parser;
use tokio::{task::JoinHandle, sync::mpsc::{unbounded_channel, UnboundedReceiver}};
use paho_mqtt::{AsyncClient, CreateOptions, SslOptionsBuilder, Error as MqttError, CreateOptionsBuilder, ConnectOptionsBuilder};
use futures::{StreamExt, Stream};

pub mod types;

#[derive(Clone, Debug, PartialEq, Parser)]
pub struct ConnectOpts {

    #[clap(short='n', long)]
    pub hostname: String,

    #[clap(short, long, default_value = "1883")]
    pub port: u16,
    
    #[clap(long)]
    pub access_code: String,
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

pub struct Printer {
    client: AsyncClient,
    rx: UnboundedReceiver<(String, Vec<u8>)>,
    _h: JoinHandle<()>,
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
            .password(opts.access_code)
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
                    },
                    _ => break,
                }
            }

            debug!("Exit connection task");
        });


        debug!("Subscribing to topic");
        // Subscribe to topic
        client.subscribe("#", 2).await?;

        Ok(Self{
            client,
            rx,
            _h,
        })
    }
}

impl Stream for Printer {
    type Item = (String, Vec<u8>);

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
