use anyhow::Result;
use esphome_api::{proto::*, server::{DefaultRequestHandler, RequestHandler, ResponseStatus, start_server}};

fn main() -> Result<()> {
    let handler = DefaultRequestHandler {
        delegate: MyRequestHandler { },
        password: None
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
