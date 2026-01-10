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
                let mut message = ListEntitiesClimateResponse::default();
                message.object_id = "test_climate_id".to_string();
                message.supported_modes = vec![
                    ClimateMode::Off as i32,
                    ClimateMode::Heat as i32,
                    ClimateMode::Cool as i32,
                    ClimateMode::HeatCool as i32
                ];
                message.visual_min_temperature = 9.0;
                message.visual_max_temperature = 32.0;
                message.visual_target_temperature_step = 0.5;
                message.visual_current_temperature_step = 0.5;
                message.supported_fan_modes = vec![
                    ClimateFanMode::ClimateFanOn  as i32,
                    ClimateFanMode::ClimateFanOff as i32,
                    ClimateFanMode::ClimateFanAuto as i32
                ];
                message.feature_flags =
                    ClimateFeature::SUPPORTS_CURRENT_TEMPERATURE |
                    ClimateFeature::SUPPORTS_ACTION;

                writer.write(&ProtoMessage::ListEntitiesClimateResponse(message))?;

                let message = ListEntitiesDoneResponse::default();
                writer.write(&ProtoMessage::ListEntitiesDoneResponse(message))?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                let mut message = ClimateStateResponse::default();
                message.set_action(ClimateAction::Idle);
                message.set_fan_mode(ClimateFanMode::ClimateFanAuto);
                message.set_mode(ClimateMode::Heat);
                message.current_temperature = 20.0;
                message.target_temperature = 19.5;

                writer.write(&ProtoMessage::ClimateStateResponse(message))?;
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}
