use std::{str::FromStr, default};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// TODO: rework everything to do with this
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Report {
    Info {
        command: InfoCommand,
        sequence_id: String,
        #[serde(flatten)]
        value: InfoValue,
    },
    Print {
        command: PrintCommand,
        sequence_id: String,
        #[serde(flatten)]
        value: PrintValue,
    },
    McPrint {
        command: McPrintCommand,
        sequence_id: String,
        param: Value,
    },
}

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PrintValue {
    #[serde(default, skip_serializing_if = "is_default")]
    ams: Option<Ams>,
    #[serde(default, skip_serializing_if = "is_default")]
    upgrade_state: Option<UpgradeState>,
    #[serde(default, skip_serializing_if = "is_default")]
    module: Vec<ModuleInfo>,
    #[serde(default, skip_serializing_if = "is_default")]
    cooling_fan_speed: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    fan_gear: Option<isize>,
    #[serde(default, skip_serializing_if = "is_default")]
    nozzle_temper: Option<f32>,
    #[serde(default, skip_serializing_if = "is_default")]
    bed_temper: Option<f32>,
    #[serde(default, skip_serializing_if = "is_default")]
    version: usize,
}

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Ams {
    #[serde(default)]
    ams: Vec<AmsInfo>,
    version: usize,
}
#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct UpgradeState {
    dis_state: usize,
    new_version_state: usize,
    ota_new_version_number: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfoValue {
    Module(Vec<ModuleInfo>),
}

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintCommand {
    #[default]
    PushStatus,
}


#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfoCommand {
    #[default]
    GetVersion,
}

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct AmsInfo {
    pub humidity: String,
    pub id: String,
    pub temp: String,
    pub tray: Vec<Tray>,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub hw_ver: String,
    pub name: String,
    pub sn: String,
    pub sw_ver: String,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Tray {
    pub id: String,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub info: TrayInfo,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TrayInfo {
    pub bed_temp: String,
    pub bed_temp_type: String,
    pub cali_idx: isize,
    pub cols: Vec<String>,
    pub ctype: usize,
    pub drying_temp: String,
    pub drying_time: String,
    pub nozzle_temp_max: String,
    pub nozzle_temp_min: String,
    pub remain: isize,
    pub tag_uid: String,
    pub tray_color: String,
    pub tray_diameter: String,
    pub tray_id_name: String,
    pub tray_info_idx: String,
    pub tray_sub_brands: String,
    pub tray_type: String,
    pub tray_uuid: String,
    pub tray_weight: String,
    pub xcam_info: String,
}

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McPrintCommand {
    #[default]
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

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

#[cfg(test)]
mod test {
    use super::*;

    use assert_json_diff::assert_json_include;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_report_bed_temper() {
        let raw = json!({ "print": { "sequence_id":"1275", "bed_temper":20.0, "command":"push_status", "msg":1} });
        let report = Report::Print {
            sequence_id: "1275".to_string(),
            command: PrintCommand::PushStatus,
            value: PrintValue{
                bed_temper: Some(20.0),
                ..Default::default()
            }
        };

        test_report_serde(raw, report);
    }

    #[test]
    fn test_report_ams() {
        let raw = json!({
            "print": {
                "ams": {
                    "ams": [
                        {
                            "humidity": "3",
                            "id": "0",
                            "temp": "26.0",
                            "tray": [
                                {
                                    "bed_temp": "45",
                                    "bed_temp_type": "1",
                                    "cali_idx": -1,
                                    "cols": [
                                        "FF6A13FF"
                                    ],
                                    "ctype": 0,
                                    "drying_temp": "55",
                                    "drying_time": "8",
                                    "id": "0",
                                    "nozzle_temp_max": "230",
                                    "nozzle_temp_min": "190",
                                    "remain": 0,
                                    "tag_uid": "976622BA00000100",
                                    "tray_color": "FF6A13FF",
                                    "tray_diameter": "1.75",
                                    "tray_id_name": "A00-A0",
                                    "tray_info_idx": "GFA00",
                                    "tray_sub_brands": "PLA Basic",
                                    "tray_type": "PLA",
                                    "tray_uuid": "A5D66E2375254E10AA0414CB8E301B3C",
                                    "tray_weight": "250",
                                    "xcam_info": "8813100EE803E8039A99193F"
                                },
                                {
                                    "id": "3"
                                }
                            ]
                        }
                    ],
                    "version": 564
                },
                "command": "push_status",
                "msg": 1,
                "sequence_id": "1479"
            }
        });

        let report = Report::Print {
            sequence_id: "1479".to_string(),
            command: PrintCommand::PushStatus,
            value: PrintValue{
                ams: Some(Ams{
                    ams: vec![AmsInfo {
                        humidity: "3".to_string(),
                        id: "0".to_string(),
                        temp: "26.0".to_string(),
                        tray: vec![
                            Tray {
                                id: "0".to_string(),
                                info: TrayInfo {
                                    bed_temp: "45".to_string(),
                                    bed_temp_type: "1".to_string(),
                                    cali_idx: -1,
                                    cols: vec!["FF6A13FF".to_string()],
                                    ctype: 0,
                                    drying_temp: "55".to_string(),
                                    drying_time: "8".to_string(),
                                    nozzle_temp_max: "230".to_string(),
                                    nozzle_temp_min: "190".to_string(),
                                    remain: 0,
                                    tag_uid: "976622BA00000100".to_string(),
                                    tray_color: "FF6A13FF".to_string(),
                                    tray_diameter: "1.75".to_string(),
                                    tray_id_name: "A00-A0".to_string(),
                                    tray_info_idx: "GFA00".to_string(),
                                    tray_sub_brands: "PLA Basic".to_string(),
                                    tray_type: "PLA".to_string(),
                                    tray_uuid: "A5D66E2375254E10AA0414CB8E301B3C".to_string(),
                                    tray_weight: "250".to_string(),
                                    xcam_info: "8813100EE803E8039A99193F".to_string(),
                                },
                            },
                            Tray {
                                id: "3".to_string(),
                                ..Default::default()
                            },
                        ],
                    }],
                    version: 564,
                }),
                ..Default::default()
            }
        };

        test_report_serde(raw, report);
    }

    #[test]
    fn test_report_upgrade_state() {
        let raw = json!({"print":{"command":"push_status","msg":1,"sequence_id":"65","upgrade_state":{"dis_state":1,"new_version_state":1,"ota_new_version_number":"01.06.01.00"}}});
        let report = Report::Print {
            command: PrintCommand::PushStatus,
            sequence_id: "65".to_string(),
            value: PrintValue {
                upgrade_state: Some(UpgradeState{
                    dis_state: 1,
                    new_version_state: 1,
                    ota_new_version_number: "01.06.01.00".to_string(),
                }),
                ..Default::default()
            },
        };

        test_report_serde(raw, report);
    }

    #[test]
    fn test_report_get_version() {
        let raw = json!({ "info": { "command": "get_version", 
        "module": [
            {"hw_ver": "", "name": "ota", "sn": "", "sw_ver": "01.06.00.00"},
            {"hw_ver": "AMS08", "name": "ams/0", "sn": "00600A2C0503248", "sw_ver": "00.00.06.32"}
        ], "sequence_id": "20016" }});
    
        let report = Report::Info {
            command: InfoCommand::GetVersion,
            sequence_id: "20016".to_string(),
            value: InfoValue::Module(vec![
                ModuleInfo {
                    name: "ota".to_string(),
                    sw_ver: "01.06.00.00".to_string(),
                    ..Default::default()
                },
                ModuleInfo {
                    hw_ver: "AMS08".to_string(),
                    name: "ams/0".to_string(),
                    sn: "00600A2C0503248".to_string(),
                    sw_ver: "00.00.06.32".to_string(),
                },
            ]),
        };

        test_report_serde(raw, report);
    }

    #[test]
    fn test_report_status_all() {
        let raw = json!({"print":{"command":"push_status","cooling_fan_speed":"0","fan_gear":0,"msg":1,"nozzle_temper":70.0,"sequence_id":"188"}});
        let report = Report::Print {
            command: PrintCommand::PushStatus,
            sequence_id: "188".to_string(),
            value: PrintValue{
                cooling_fan_speed: Some("0".to_string()), fan_gear: Some(0), nozzle_temper: Some(70.0), ..Default::default()
            }
        };

        test_report_serde(raw, report);
    }

    #[test]
    fn test_report_status_nozzle_temp() {
        let raw = json!({"print":{"command":"push_status","msg":1,"nozzle_temper":85.0,"sequence_id":"190"}});
        let report = Report::Print {
            command: PrintCommand::PushStatus,
            sequence_id: "190".to_string(),
            value: PrintValue{
                nozzle_temper: Some(85.0),
                ..Default::default()
            },
        };

        test_report_serde(raw, report);
    }


    /// Helper to test report serialisation and deserialisation
    fn test_report_serde(raw: Value, report: Report) {
        println!("report: {report:?}\r\njson: {raw}");

        // Check conversion against expected value
        let converted = serde_json::to_value(&report).expect("value conversion failed");
        assert_json_include!(actual: raw, expected: converted);

        // Test decoding
        let decoded: Report = serde_json::from_value(raw.clone()).expect("failed to decode");
        assert_eq!(decoded, report);
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
