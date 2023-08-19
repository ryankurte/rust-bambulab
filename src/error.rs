

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

