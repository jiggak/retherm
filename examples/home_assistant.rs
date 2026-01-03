use std::io::Write;

use anyhow::Result;
use esphome_api::{proto::*, server::{ConnectStatus, DefaultRequestHandler, RequestHandler, start_server}};

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
    fn handle_request<S: Write>(&self, message: &ProtoMessage, stream: &mut S) -> Result<ConnectStatus> {
        match message {
            ProtoMessage::ListEntitiesRequest(_) => {
                write_message(stream, &ListEntitiesButtonResponse {
                    object_id: "test_button_object_id".to_string(),
                    key: 0,
                    name: "Test Button".to_string(),
                    icon: "mdi:test-button-icon".to_string(),
                    disabled_by_default: false,
                    entity_category: EntityCategory::None as i32,
                    device_class: "test_button_device_class".to_string(),
                    device_id: 0
                })?;

                write_message(stream, &ListEntitiesDoneResponse { })?;
            }
            _ => { }
        }

        Ok(ConnectStatus::Continue)
    }
}
