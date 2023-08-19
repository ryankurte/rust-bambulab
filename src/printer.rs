

use futures::StreamExt;
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, SslOptionsBuilder,
};
use rustls::client::ServerCertVerifier;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, trace, error};

use crate::{ConnectOpts, Error};

/// Bambu printer handle
#[derive(Clone)]
pub struct Printer {
    opts: ConnectOpts,
    tx: UnboundedSender<Commands>,
}

#[derive(Clone, Debug)]
pub enum Commands {
    Subscribe(PrinterSender),
    Disconnect,
}

impl std::fmt::Debug for Printer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "printer")
    }
}

pub type PrinterSender = UnboundedSender<(String, String)>;
pub type PrinterReceiver = UnboundedReceiver<(String, String)>;

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
        let (tx, mut rx) = unbounded_channel::<Commands>();

        // Subscribe to topic
        debug!("Subscribing to topic");
        client.subscribe("#", 2).await?;

        // Start listener
        let _h = tokio::task::spawn(async move {
            debug!("Start connection task");

            let mut listeners: Vec<PrinterSender> = vec![];
            let _ = client;

            loop {
                tokio::select!(
                    // Listen for incoming MQTT messages
                    v = mqtt_rx.next() => {
                        match v {
                            Some(Some(v)) => {
                                let topic = v.topic().to_string();

                                let payload = match std::str::from_utf8(v.payload()) {
                                    Ok(v) => v.to_string(),
                                    Err(_) => {
                                        debug!("rx {}: {:02x?}", v.topic(), v.payload());
                                        continue;
                                    }
                                };

                                listeners.retain(|tx| {
                                    tx.send((topic.clone(), payload.clone())).is_ok()
                                });
                            },
                            _ => (),
                        }
                    },
                    // Listen for incoming commands
                    c = rx.recv() => {
                        match c {
                            Some(Commands::Subscribe(tx)) => listeners.push(tx),
                            Some(Commands::Disconnect) => break,
                            None => (),
                        }
                    },
                );
            }

            debug!("Exit connection task");

            if let Err(e) = client.disconnect(None).await {
                error!("Client disconnect error: {e:?}");
            }
        });

        Ok(Self { opts, tx })
    }

    /// Fetch listen channel for receiving events
    pub fn listen(&self) -> Result<PrinterReceiver, Error> {
        let (tx, rx) = unbounded_channel();

        // Send subscription handle
        self.tx.send(Commands::Subscribe(tx))
            .map_err(|_| Error::SendError)?;        

        // Return receiver
        Ok(rx)
    }

    /// Disconnect client
    pub async fn disconnect(self) -> Result<(), Error> {
        self.tx.send(Commands::Disconnect)
            .map_err(|_| Error::SendError)?;

        Ok(())
    }
}

impl std::hash::Hash for Printer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.opts.hostname.hash(state);
        self.opts.port.hash(state);
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

pub const BAMBU_ROOT: &str = "
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
