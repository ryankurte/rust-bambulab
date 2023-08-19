use serde::{Deserialize, Serialize};

mod command;
pub use command::*;

mod report;
pub use report::*;

mod errors;
pub use errors::ERRORS;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Printing
    Printing = 0,
    /// Auto Bed Leveling
    Abl = 1,
    /// Heatbed Preheating
    HeatbedPreheat = 2,
    /// Sweeping XY Mech Mode
    SweepingXyMechMode = 3,
    /// Changing Filament
    ChangingFilament = 4,
    /// M400 Pause
    M400Pause = 5,
    /// Paused due to filament runout
    FilamentRunoutPause = 6,
    /// Heating Hotend
    HeatingHotend = 7,
    /// Calibrating Extrusion
    CalibratingExtrusion = 8,
    /// Scanning Bed Surface
    ScanningBedSurface = 9,
    /// Inspecting First Layer
    InspectingFirstLayer = 10,
    /// Identifying Build Plate Type
    IdentifyingBuildPlateType = 11,
    /// Calibrating Micro Lidar
    CalibratingMicroLidar = 12,
    /// Homing Toolhead
    HomingToolhead = 13,
    /// Cleaning Nozzle Tip
    CleaningNozzleTip = 14,
    /// Checking Extruder Temperature
    CheckingExtruderTemperature = 15,
    /// Printing was paused by the user
    UserPause = 16,
    /// Pause of front cover falling
    FrontCoverPause = 17,
    /// Calibrating Micro Lidar
    CalibratingMicroLidar2 = 18,
    /// Calibrating Extrusion Flow
    CalibratingExtrusionFlow = 19,
    /// Paused due to nozzle temperature malfunction
    NozzleTempMalfunction = 20,
    /// Paused due to heat bed temperature malfunction
    GearBedTempMalfunction = 21,
    /// Idle
    Idle = 255,
}
