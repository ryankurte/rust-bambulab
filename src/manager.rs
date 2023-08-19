use crate::ConnectOpts;


/// Bambu printer manager
pub struct PrinterManager {

}

pub enum Commands {
    /// Connect to a printer
    Connect(ConnectOpts),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrinterHandle {

}

impl PrinterManager {
    
}
