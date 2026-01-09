use anyhow::Result;
use esphome_api::{proto::*, server::{DefaultHandler, RequestHandler, ResponseStatus, SecurityMode, start_server}};

fn main() -> Result<()> {
    let handler = DefaultHandler {
        delegate: MyRequestHandler { },
        security: SecurityMode::encryption(
            "jfD5V1SMKAPXNC8+d6BvE1EGBHJbyw2dSc0Q+ymNMhU=",
            "hallway-thermostat",
            "01:02:03:04:05:06"
        )?,
        server_info: "Nest App 0.0.1".to_string(),
        node_name: "hallway-thermostat".to_string(),
        friendly_name: "Hallway Thermostat".to_string(),
        manufacturer: "Nest".to_string(),
        model: "Gen2 Thermostat".to_string(),
        mac_address: "01:02:03:04:05:06".to_string()
    };

    start_server("0.0.0.0:6053",&handler.security, &handler)?;

    Ok(())
}

struct MyRequestHandler;

impl RequestHandler for MyRequestHandler {
    fn handle_request<W: MessageWriter>(
        &self,
        message: &ProtoMessage,
        writer: &mut W
    ) -> Result<ResponseStatus> {
        match message {
            ProtoMessage::ListEntitiesRequest(_) => {
                writer.write(&ListEntitiesClimateResponse {
                    object_id: "test_climate_id".to_string(),
                    key: 0,
                    name: "".to_string(),
                    // Deprecated: use feature_flags
                    supports_current_temperature: false,
                    // Deprecated: use feature_flags
                    supports_two_point_target_temperature: false,
                    supported_modes: vec![
                        ClimateMode::Off as i32,
                        ClimateMode::Heat as i32,
                        ClimateMode::Cool as i32,
                        ClimateMode::HeatCool as i32
                    ],
                    visual_min_temperature: 9.0,
                    visual_max_temperature: 32.0,
                    visual_target_temperature_step: 0.5,
                    // Deprecated: use CLIMATE_PRESET_AWAY in supported_presets
                    legacy_supports_away: false,
                    // Deprecated: use feature_flags
                    supports_action: true, // whats this do?
                    supported_fan_modes: vec![
                        ClimateFanMode::ClimateFanOn  as i32,
                        ClimateFanMode::ClimateFanOff as i32,
                        ClimateFanMode::ClimateFanAuto as i32
                    ],
                    supported_swing_modes: vec![],
                    supported_custom_fan_modes: vec![],
                    // some values in ClimatePreset look relavent, e.g. "Away"
                    supported_presets: vec![],
                    supported_custom_presets: vec![],
                    disabled_by_default: false,
                    icon: "".to_string(),
                    entity_category: EntityCategory::None as i32,
                    visual_current_temperature_step: 0.5,
                    // Deprecated: use feature_flags
                    supports_current_humidity: false,
                    // Deprecated: use feature_flags
                    supports_target_humidity: false,
                    visual_min_humidity: 0.0,
                    visual_max_humidity: 0.0,
                    device_id: 0,
                    feature_flags: ClimateFeature::SUPPORTS_CURRENT_TEMPERATURE
                        | ClimateFeature::SUPPORTS_ACTION
                })?;

                /*
                writer.write(&ListEntitiesButtonResponse {
                    object_id: "test_button_object_id".to_string(),
                    key: 0,
                    name: "Test Button".to_string(),
                    icon: "mdi:test-button-icon".to_string(),
                    disabled_by_default: false,
                    entity_category: EntityCategory::None as i32,
                    device_class: "test_button_device_class".to_string(),
                    device_id: 0
                })?;
                */

                writer.write(&ListEntitiesDoneResponse { })?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                writer.write(&ClimateStateResponse {
                    key: 0,
                    mode: ClimateMode::Heat as i32,
                    current_temperature: 19.5,
                    target_temperature: 19.5,
                    target_temperature_low: 0.0,
                    target_temperature_high: 0.0,
                    unused_legacy_away: false,
                    action: ClimateAction::Heating as i32,
                    fan_mode: ClimateFanMode::ClimateFanAuto as i32,
                    swing_mode: 0,
                    custom_fan_mode: "".to_string(),
                    preset: 0,
                    custom_preset: "".to_string(),
                    current_humidity: 0.0,
                    target_humidity: 0.0,
                    device_id: 0
                })?;
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}
