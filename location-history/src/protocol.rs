//! Data types for `Records.json`.

use serde::{de, Deserialize, Deserializer};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy)]
pub struct LngLat(pub f32, pub f32);

impl LngLat {
    #[must_use]
    pub fn lng(&self) -> f32 {
        self.0
    }

    #[must_use]
    pub fn lat(&self) -> f32 {
        self.1
    }
}

impl From<(f32, f32)> for LngLat {
    fn from((lng, lat): (f32, f32)) -> Self {
        Self(lng, lat)
    }
}

/// Location entry source.
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Source {
    Gps,
    Cell,
    Wifi,
    Unknown,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityType {
    Still,
    Unknown,
    InVehicle,
    OnFoot,
    Tilting,
    OnBicycle,
    ExitingVehicle,
    Walking,
    Running,
    InRoadVehicle,
    InRailVehicle,
    InFourWheelerVehicle,
    InTwoWheelerVehicle,
    InCar,
    InBus,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ActivityConfidence {
    #[serde(rename = "type")]
    pub typ: ActivityType,
    /// Confidence percentage.
    pub confidence: u8,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Activity {
    pub activity: Vec<ActivityConfidence>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceDesignation {
    Unknown,
    Primary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacAddr(pub [u8; 6]);

impl MacAddr {
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub const fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8) -> Self {
        Self([a, b, c, d, e, f])
    }
}

impl<'de> Deserialize<'de> for MacAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let int = String::deserialize(deserializer)?;
        let buf: [u8; 8] = int.parse::<u64>().map_err(de::Error::custom)?.to_be_bytes();

        if buf[0] != 0 || buf[1] != 0 {
            Err(de::Error::custom("mac address must be 6 bytes"))
        } else {
            Ok(Self::new(buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]))
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AccessPoint {
    pub mac: MacAddr,
    pub strength: i8,
    pub frequency_mhz: u16,
    #[serde(default)]
    pub is_connected: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WifiScan {
    pub access_points: Option<Vec<AccessPoint>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformType {
    Android,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FormFactor {
    Phone,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LocationMetadata {
    pub wifi_scan: Option<WifiScan>,
    pub active_wifi_scan: Option<WifiScan>,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct Location {
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
    #[serde(rename = "latitudeE7", with = "e7_scalar")]
    pub latitude: f32,
    #[serde(rename = "longitudeE7", with = "e7_scalar")]
    pub longitude: f32,
    pub accuracy: i32,
}

mod e7_scalar {
    use serde::{Deserialize, Deserializer};

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(clippy::cast_precision_loss)] // we only have 9 significant digits
        i32::deserialize(deserializer).map(|int| int as f32 / 10_000_000.0)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LocationEntry {
    #[serde(flatten)]
    pub location: Location,
    /// Velocity in meters per second.
    pub velocity: Option<u32>,
    /// Heading (degrees).
    pub heading: Option<u16>,
    pub source: Source,
    pub device_tag: i32,
    pub activity: Option<Vec<Activity>>,
    pub altitude: Option<i32>,
    pub vertical_accuracy: Option<i32>,
    pub device_designation: Option<DeviceDesignation>,
    pub active_wifi_scan: Option<WifiScan>,
    pub platform_type: Option<PlatformType>,
    pub os_level: Option<u8>,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub server_timestamp: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub device_timestamp: Option<OffsetDateTime>,
    pub battery_charging: Option<bool>,
    pub form_factor: Option<FormFactor>,
    pub location_metadata: Option<Vec<LocationMetadata>>,
    pub inferred_location: Option<Vec<Location>>,
    pub place_id: Option<String>,
}

impl LocationEntry {
    #[must_use]
    pub fn timestamp(&self) -> OffsetDateTime {
        self.location.timestamp
    }

    #[must_use]
    pub fn lnglat(&self) -> LngLat {
        LngLat(self.location.longitude, self.location.latitude)
    }
}
