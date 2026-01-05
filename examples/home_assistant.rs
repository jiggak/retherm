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

    start_server(handler)?;

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

                writer.write(&ListEntitiesDoneResponse { })?;
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}
