use std::{str::FromStr};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// TODO: rework everything to do with this
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Report {
    Print{
        sequence_id: String,
        #[serde(flatten)]
        command: PrintCommand,
    },
    McPrint {
        command: McPrintCommand,
        param: Value,
        sequence_id: String,
    },
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag="command")]
pub enum PrintCommand {
    Ams(Value),
    PushStatus{
        bed_temper: f32,
    }
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
    fn parse_print_cmd() {
        let tests = &[
            (
                r#"{ "print": { "sequence_id":"1275", "bed_temper":20.0, "command":"push_status", "msg":1} }"#,
                Report::Print{ sequence_id: "1275".to_string(), command: PrintCommand::PushStatus{ bed_temper: 20.0 } }
            ),
            #[cfg(todo)]
            (
                r#"
                "{\"print\":{\"ams\":{\"ams\":[{\"humidity\":\"3\",\"id\":\"0\",\"temp\":\"26.0\",\"tray\":[{\"bed_temp\":\"45\",\"bed_temp_type\":\"1\",\"cali_idx\":-1,\"cols\":[\"FF6A13FF\"],\"ctype\":0,\"drying_temp\":\"55\",\"drying_time\":\"8\",\"id\":\"0\",\"nozzle_temp_max\":\"230\",\"nozzle_temp_min\":\"190\",\"remain\":0,\"tag_uid\":\"976622BA00000100\",\"tray_color\":\"FF6A13FF\",\"tray_diameter\":\"1.75\",\"tray_id_name\":\"A00-A0\",\"tray_info_idx\":\"GFA00\",\"tray_sub_brands\":\"PLA Basic\",\"tray_type\":\"PLA\",\"tray_uuid\":\"A5D66E2375254E10AA0414CB8E301B3C\",\"tray_weight\":\"250\",\"xcam_info\":\"8813100EE803E8039A99193F\"},{\"bed_temp\":\"0\",\"bed_temp_type\":\"0\",\"cali_idx\":-1,\"cols\":[\"443089FF\"],\"ctype\":0,\"drying_temp\":\"0\",\"drying_time\":\"0\",\"id\":\"1\",\"nozzle_temp_max\":\"240\",\"nozzle_temp_min\":\"190\",\"remain\":0,\"tag_uid\":\"0000000000000000\",\"tray_color\":\"443089FF\",\"tray_diameter\":\"0.00\",\"tray_id_name\":\"\",\"tray_info_idx\":\"GFL99\",\"tray_sub_brands\":\"\",\"tray_type\":\"PLA\",\"tray_uuid\":\"00000000000000000000000000000000\",\"tray_weight\":\"0\",\"xcam_info\":\"000000000000000000000000\"},{\"bed_temp\":\"0\",\"bed_temp_type\":\"0\",\"cali_idx\":-1,\"cols\":[\"FF5F00FF\"],\"ctype\":0,\"drying_temp\":\"0\",\"drying_time\":\"0\",\"id\":\"2\",\"nozzle_temp_max\":\"270\",\"nozzle_temp_min\":\"220\",\"remain\":0,\"tag_uid\":\"0000000000000000\",\"tray_color\":\"FF5F00FF\",\"tray_diameter\":\"0.00\",\"tray_id_name\":\"\",\"tray_info_idx\":\"GFG99\",\"tray_sub_brands\":\"\",\"tray_type\":\"PETG\",\"tray_uuid\":\"00000000000000000000000000000000\",\"tray_weight\":\"0\",\"xcam_info\":\"000000000000000000000000\"},{\"id\":\"3\"}]}],\"version\":564},\"command\":\"push_status\",\"msg\":1,\"sequence_id\":\"1479\"}}""#,
                Report::Print{ sequence_id: "1479".to_string(), command: PrintCommand::Ams(Value::Null) },
            )
        ];

        for (s, v) in tests {
            println!("s: {s} v: {v:?}");

            let _a: serde_json::Value = serde_json::from_str(s).unwrap();

            let _e = serde_json::to_string(v).expect("failed to encode");

            let r: Report = serde_json::from_str(s).expect("failed to decode");
            assert_eq!(r, *v);
        }
    }

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
