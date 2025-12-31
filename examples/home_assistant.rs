mod esphome {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

use std::{io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};

use anyhow::{Result, anyhow};
use prost::{Message, bytes::{Buf, BufMut, Bytes, BytesMut}, encoding::{decode_varint, encode_varint}};

use esphome::{HelloRequest, HelloResponse};

use crate::esphome::{AuthenticationRequest, AuthenticationResponse, ButtonCommandRequest, DeviceInfoRequest, DeviceInfoResponse, DisconnectRequest, DisconnectResponse, EntityCategory, ListEntitiesBinarySensorResponse, ListEntitiesButtonResponse, ListEntitiesDoneResponse, ListEntitiesRequest, ListEntitiesSwitchResponse, PingRequest, PingResponse, SensorStateResponse, SubscribeHomeAssistantStatesRequest, SubscribeHomeassistantServicesRequest, SubscribeStatesRequest};

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

#[derive(Debug)]
struct Frame {
    message_size: u64,
    type_id: u64
}

impl Frame {
    pub fn decode(buffer: &mut impl Buf) -> Result<Self> {
        let byte_zero = buffer.get_u8();
        if byte_zero != 0 {
            return Err(anyhow!("Expected first byte of frame to be zero, found {}", byte_zero));
        }

        let message_size = decode_varint(buffer)?;
        let type_id = decode_varint(buffer)?;

        Ok(Self {
            message_size, type_id
        })
    }
}

#[derive(Debug)]
enum Request {
    Hello(HelloRequest),
    Authentication(AuthenticationRequest),
    Disconnect(DisconnectRequest),
    Ping(PingRequest),
    DeviceInfo(DeviceInfoRequest),
    ListEntities(ListEntitiesRequest),
    SubscribeStates(SubscribeStatesRequest),
    SubscribeHomeassistantServices(SubscribeHomeassistantServicesRequest),
    SubscribeHomeAssistantStates(SubscribeHomeAssistantStatesRequest),
    ButtonCommand(ButtonCommandRequest)
}

#[derive(Debug)]
struct Response<M> {
    type_id: u64,
    message: M
}

fn read_request<R>(stream: &mut R) -> Result<Request>
    where R: BufRead
{
    let buf = stream.fill_buf()?;
    let mut buffer = Bytes::copy_from_slice(buf);
    println!("Frame buffer {} - {:02x?}", buf.len(), buf);

    let frame = Frame::decode(&mut buffer)?;
    let bytes_used = buf.len() - buffer.remaining();
    println!("Frame size:{} type:{} bytes_used:{}", frame.message_size, frame.type_id, bytes_used);

    stream.consume(bytes_used);

    let message_size = frame.message_size as usize;

    let buffer = if message_size > 0 {
        let buf = stream.fill_buf()?;
        if buf.len() < message_size {
            return Err(anyhow!("Buffer underrun; buf {}, message {}", buf.len(), message_size));
        }

        Bytes::copy_from_slice(&buf[..message_size])
    } else {
        Bytes::new()
    };

    println!("Message buffer {} - {:02x?}", buffer.len(), &buffer[..]);

    let result = match frame.type_id {
        1 => Ok(Request::Hello(HelloRequest::decode(buffer)?)),
        3 => Ok(Request::Authentication(AuthenticationRequest::decode(buffer)?)),
        5 => Ok(Request::Disconnect(DisconnectRequest::decode(buffer)?)),
        7 => Ok(Request::Ping(PingRequest::decode(buffer)?)),
        9 => Ok(Request::DeviceInfo(DeviceInfoRequest::decode(buffer)?)),
        11 => Ok(Request::ListEntities(ListEntitiesRequest::decode(buffer)?)),
        20 => Ok(Request::SubscribeStates(SubscribeStatesRequest::decode(buffer)?)),
        34 => Ok(Request::SubscribeHomeassistantServices(SubscribeHomeassistantServicesRequest::decode(buffer)?)),
        38 => Ok(Request::SubscribeHomeAssistantStates(SubscribeHomeAssistantStatesRequest::decode(buffer)?)),
        62 => Ok(Request::ButtonCommand(ButtonCommandRequest::decode(buffer)?)),
        _ => Err(anyhow!("Unhandled message id {}", frame.type_id))
    };

    stream.consume(message_size);

    result
}

fn send_response<S, M>(stream: &mut S, response: Response<M>) -> Result<()>
    where S: Write, M: Message
{
    let mut buffer = BytesMut::with_capacity(512);

    let message = response.message;
    let data_len = message.encoded_len() as u64;

    buffer.put_u8(0u8);
    encode_varint(data_len, &mut buffer);
    encode_varint(response.type_id, &mut buffer);
    message.encode(&mut buffer)?;

    let buf = buffer.freeze();
    let sz = stream.write(&buf)?;
    println!("Write {} bytes", sz);

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
            Request::Hello(_) => {
                send_response(&mut stream, Response {
                    type_id: 2,
                    message: HelloResponse {
                        api_version_major: 1,
                        api_version_minor: 13,
                        server_info: "Nest App".to_string(),
                        name: "Nest Thermostat".to_string()
                    }
                })?;
            }
            Request::Authentication(_) => {
                send_response(&mut stream, Response {
                    type_id: 4,
                    message: AuthenticationResponse {
                        invalid_password: false
                    }
                })?;
            }
            Request::Disconnect(_) => {
                send_response(&mut stream, Response {
                    type_id: 6,
                    message: DisconnectResponse { }
                })?;
                break;
            }
            Request::Ping(_) => {
                send_response(&mut stream, Response {
                    type_id: 8,
                    message: PingResponse { }
                })?;
            }
            Request::DeviceInfo(_) => {
                send_response(&mut stream, Response {
                    type_id: 10,
                    message: DeviceInfoResponse {
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
                    }
                })?;
            }
            Request::ListEntities(_) => {
                send_response(&mut stream, Response {
                    type_id: 61,
                    message: ListEntitiesButtonResponse {
                        object_id: "test_button_object_id".to_string(),
                        key: 0,
                        name: "Test Button".to_string(),
                        icon: "mdi:test-button-icon".to_string(),
                        disabled_by_default: false,
                        entity_category: EntityCategory::None as i32,
                        device_class: "test_button_device_class".to_string(),
                        device_id: 0
                    }
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

                send_response(&mut stream, Response {
                    type_id: 19,
                    message: ListEntitiesDoneResponse { }
                })?;
            }
            Request::SubscribeStates(_) => {
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
            Request::SubscribeHomeassistantServices(_) => {
                // ignore ?
            }
            Request::SubscribeHomeAssistantStates(_) => {
                // ignore ?
            },
            Request::ButtonCommand(cmd) => {
                println!("Button id:{} key:{}", cmd.device_id, cmd.key)
            }
        }
    }

    Ok(())
}
