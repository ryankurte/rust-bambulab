use std::{str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Report {
    Print(Print),
    McPrint {
        command: McPrintCommand,
        param: Value,
        sequence_id: String,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Print {
    pub ams: Value,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McPrintCommand {
    PushInfo,
}

#[derive(Clone, PartialEq, Debug)]
pub enum McPrintValue {
    AmsPeriod,
    AmsTask,
    Bmc,
    /// Bed levelling measurement
    BmcMeas {
        x: f32,
        y: f32,
        z_c: f32,
        z_d: f32,
    },
    Unknown(String),
}

impl McPrintValue {
    pub fn is_bmc_meas(&self) -> bool {
        match self {
            Self::BmcMeas { .. } => true,
            _ => false,
        }
    }
}

lazy_static::lazy_static! {
    static ref BMC_MEAS: Regex = Regex::new("X([0-9.]+) Y([0-9.]+),z_c=[ ]+(-*[0-9.]+)[ ]*,z_d=(-*[0-9.]+)").unwrap();

    static ref VAL_PAIR: Regex = Regex::new("([a-zA-Z_]+)=([0-9.]+)").unwrap();
}

impl FromStr for McPrintValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('\"').trim_end_matches('\"');

        // TODO: match and parse values
        if s.starts_with("[AMS][Period]") {
            return Ok(Self::AmsPeriod);
        } else if s.starts_with("[AMS][TASK]") {
            return Ok(Self::AmsTask);
        } else if s.starts_with("[BMC]") {
            let s = s.trim_start_matches("[BMC] ");

            if let Some(c) = BMC_MEAS.captures(s) {
                return Ok(Self::BmcMeas {
                    x: c.get(1)
                        .map(|v| f32::from_str(v.as_str()))
                        .unwrap()
                        .unwrap(),
                    y: c.get(2)
                        .map(|v| f32::from_str(v.as_str()))
                        .unwrap()
                        .unwrap(),
                    z_c: c
                        .get(3)
                        .map(|v| f32::from_str(v.as_str()))
                        .unwrap()
                        .unwrap(),
                    z_d: c
                        .get(4)
                        .map(|v| f32::from_str(v.as_str()))
                        .unwrap()
                        .unwrap(),
                });
            } else {
                return Ok(Self::Bmc);
            }
        }

        Ok(Self::Unknown(s.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_mc_params() {
        let tests = &[
            (
                "[AMS][Period]:(AMS0-S255)cmd_en=1;act_en=1;sta=0;sw=1-1-1-0;c_len=0.000m,cnt=0",
                McPrintValue::AmsPeriod,
            ),(
                "[AMS][TASK]ams num:1,ams_exist:0x1,tray_now: 255",
                McPrintValue::AmsTask,
            ),(
                "[AMS][TASK]tray_exist:0x7;tray_read_done:0x7,vailed:0x7,reading:0x0",
                McPrintValue::AmsTask,
            ),(
                "[AMS][TASK]ams0 temp:27.4;humidity:27%;humidity_idx:3",
                McPrintValue::AmsTask,
            ),(
                "[AMS][TASK]ams0 en=1,mode=0,sta=0",
                McPrintValue::AmsTask,
            ),(
                "[AMS][Period]:(AMS0-S255)cmd_en=1;act_en=1;sta=0;sw=1-1-1-0;c_len=0.000m,cnt=0",
                McPrintValue::AmsPeriod,
            ),(
                "[AMS][Period]:bldc_i=-0.00,u=0.00,spd=0.00;dw_spd=0.00;bdc_i=-0.00,u=0.00,spd=-0.00",
                McPrintValue::AmsPeriod,
            ),(
                "[BMC] z_t_cnt=119,p=0.373,rr=7.440,d=0.094,pos=234.7,272.7",
                McPrintValue::Bmc,
            ),(
                "[BMC] avr_rr=7.688645,avr_d_rr=0.501",
                McPrintValue::Bmc,
            ),(
                "[BMC] X231.0 Y236.0,z_c=      0.507      ,z_d=0.094",
                McPrintValue::BmcMeas{ x: 231.0, y: 236.0, z_c: 0.507, z_d: 0.094},
            ),(
                "[BMC] PX231.0 Y236.0,prev_z_c_diff=      -0.286      ",
                McPrintValue::Bmc,
            )
        ];

        for (s, t) in tests {
            println!("Parsing: {s}, Expected: {t:?}");

            let t1 = McPrintValue::from_str(s).unwrap();
            assert_eq!(&t1, t);
        }
    }
}
