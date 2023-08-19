use bambu::{ConnectOpts, Printer};

/// Application update message type
#[derive(Clone, Debug)]
pub enum Message {
    // Update hostname
    SetHostname(String),

    // Update access code
    SetAccessCode(String),

    // Connect to printer
    Connect(ConnectOpts),

    // Receive connected printer instance
    Connected(Printer),

    // Disconnect from printer
    Disconnect,

    // Disconnect succeeded
    Disconnected,

    // Load log file
    Load(String),

    // Update chart yaw
    Yaw(f64),
    // Update chart pitch
    Pitch(f64),
    // Update chart yaw and pitch
    YawPitch(f64, f64),

    // UI tick
    Tick,

    Report(String),
}
