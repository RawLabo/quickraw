use super::*;

mod general;

pub struct FujifilmGeneral {
    info: ParsedRawInfo,
}

pub static FUJI_SENSOR_TABLE: phf::Map<&'static str, u8> = phf::phf_map! {
    "X-T1" => 0, // RBGBRG by default

    "X-T3" => 1, // GGRGGB
    "X-T4" => 1,
    "X-T30" => 1,
    "X-S10" => 1,
    "X-Pro3" => 1,
    "X-Pro4" => 1,
    "X-E4" => 1,
    "X100V" => 1,

    "GFX50R" => 100, // RGGB
    "GFX50S" => 100,
    "GFX100" => 100,
    "GFX50SII" => 100,
    "GFX100S" => 100,
};