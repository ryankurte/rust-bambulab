use serde::{Deserialize, Serialize};


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    System {
        sequence_id: usize,
        command: SystemCommand,
    },
    Print {
        sequence_id: usize,
        command: PrintCommand,
    },
    Info {
        sequence_id: usize,
        command: InfoCommand,
    },
}

impl Command {
    pub fn system(sequence_id: usize, command: SystemCommand) -> Self {
        Self::System {
            sequence_id,
            command,
        }
    }

    pub fn print(sequence_id: usize, command: PrintCommand) -> Self {
        Self::Print {
            sequence_id,
            command,
        }
    }

    pub fn info(sequence_id: usize, command: InfoCommand) -> Self {
        Self::Info {
            sequence_id,
            command,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemCommand {}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintCommand {
    Pause,
    Resume,
    Stop,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfoCommand {
    GetVersion,
}
