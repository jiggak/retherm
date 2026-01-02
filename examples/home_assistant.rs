use std::{io::BufReader, net::{TcpListener, TcpStream}};

use anyhow::Result;
use esphome_api::{ProtoMessage, proto::*, read_request, send_response};

fn main() -> Result<()> {
    println!("Create listener");
    let listener = TcpListener::bind("0.0.0.0:6053")?;

    println!("Listen for incoming");
    for stream in listener.incoming() {
        println!("Connection established");
        handle_connection(stream?)?;
    }

    Ok(())
}

fn handle_connection(stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(stream);

    loop {
        // the robot suggested `AsyncReadExt::read_buf` from tokio to read straight
        // into the `BytesMut` instance. That looks way cleaner... might be time to
        // stop resisting tokio.
        // e.g. stream.read_buf(&mut message_buffer)?;

        let request = read_request(&mut reader)?;
        println!("Request: {:?}", request);

        let mut stream = reader.get_ref();

        match request {
            ProtoMessage::HelloRequest(_) => {
                send_response(&mut stream, &HelloResponse {
                    api_version_major: 1,
                    api_version_minor: 13,
                    server_info: "Nest App".to_string(),
                    name: "Nest Thermostat".to_string()
                })?;
            }
            ProtoMessage::AuthenticationRequest(_) => {
                send_response(&mut stream, &AuthenticationResponse {
                    invalid_password: false
                })?;
            }
            ProtoMessage::DisconnectRequest(_) => {
                send_response(&mut stream, &DisconnectResponse { })?;
                break;
            }
            ProtoMessage::PingRequest(_) => {
                send_response(&mut stream, &PingResponse { })?;
            }
            ProtoMessage::DeviceInfoRequest(_) => {
                send_response(&mut stream, &DeviceInfoResponse {
                    uses_password: false,
                    name: "Nest Thermostat".to_string(),
                    mac_address: "00:00:00:00:00:01".to_string(),
                    esphome_version: "2025.12.2".to_string(),
                    compilation_time: "".to_string(),
                    model: "Nest Thermostat".to_string(),
                    has_deep_sleep: false,
                    // When I used values for project_*, HA would not show
                    // any entities for the device
                    project_name: "".to_string(),
                    project_version: "".to_string(),
                    webserver_port: 0,
                    legacy_bluetooth_proxy_version: 0,
                    bluetooth_proxy_feature_flags: 0,
                    manufacturer: "Josh".to_string(),
                    friendly_name: "Nest App".to_string(),
                    legacy_voice_assistant_version: 0,
                    voice_assistant_feature_flags: 0,
                    suggested_area: "".to_string(),
                    bluetooth_mac_address: "".to_string(),
                    api_encryption_supported: false,
                    devices: vec![],
                    areas: vec![],
                    area: None,
                    zwave_proxy_feature_flags: 0,
                    zwave_home_id: 0
                })?;
            }
            ProtoMessage::ListEntitiesRequest(_) => {
                send_response(&mut stream, &ListEntitiesButtonResponse {
                    object_id: "test_button_object_id".to_string(),
                    key: 0,
                    name: "Test Button".to_string(),
                    icon: "mdi:test-button-icon".to_string(),
                    disabled_by_default: false,
                    entity_category: EntityCategory::None as i32,
                    device_class: "test_button_device_class".to_string(),
                    device_id: 0
                })?;

                // send_response(&mut stream, Response {
                //     type_id: 12,
                //     message: ListEntitiesBinarySensorResponse {
                //         object_id: "test_sensor_object_id".to_string(),
                //         key: 0,
                //         name: "Test Sensor".to_string(),
                //         device_class: "test_sensor_device_class".to_string(),
                //         is_status_binary_sensor: true,
                //         disabled_by_default: false,
                //         icon: "mdi:test-sensor-icon".to_string(),
                //         entity_category: EntityCategory::None as i32,
                //         device_id: 0
                //     }
                // })?;

                // send_response(&mut stream, Response {
                //     type_id: 17,
                //     message: ListEntitiesSwitchResponse {
                //         object_id: "test_switch_object_id".to_string(),
                //         key: 0,
                //         name: "test_switch".to_string(),
                //         icon: String::default(),
                //         assumed_state: false,
                //         disabled_by_default: false,
                //         entity_category: EntityCategory::None as i32,
                //         device_class: String::default(),
                //         device_id: 0
                //     }
                // })?;

                send_response(&mut stream, &ListEntitiesDoneResponse { })?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                // send_response(&mut stream, Response {
                //     type_id: 25,
                //     message: SensorStateResponse {
                //         key: 0,
                //         state: 1.0,
                //         missing_state: false,
                //         device_id: 0
                //     }
                // })?;
            }
            ProtoMessage::ButtonCommandRequest(cmd) => {
                println!("Button id:{} key:{}", cmd.device_id, cmd.key)
            }
            _ => { }
        }
    }

    Ok(())
}
