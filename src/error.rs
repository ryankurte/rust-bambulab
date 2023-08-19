use paho_mqtt::Error as MqttError;

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum Error {
    /// MQTT error {0}
    Mqtt(MqttError),
    /// Channel send error
    SendError,
}

impl From<MqttError> for Error {
    fn from(value: MqttError) -> Self {
        Self::Mqtt(value)
    }
}
