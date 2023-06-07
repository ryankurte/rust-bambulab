
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Report {
    Print(Print),
    McPrint(Value),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Print {
    pub ams: Value,
}