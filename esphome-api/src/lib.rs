pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/esphome.rs"));
}

use std::io::{BufRead, Write};

use anyhow::{Result, anyhow};
use prost::{Message, bytes::{Buf, BufMut, Bytes, BytesMut}, encoding::{decode_varint, encode_varint}};

use crate::proto::*;

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
pub enum ProtoMessage {
    HelloRequest(HelloRequest),
    HelloResponse(HelloResponse),
    AuthenticationRequest(AuthenticationRequest),
    AuthenticationResponse(AuthenticationResponse),
    DisconnectRequest(DisconnectRequest),
    DisconnectResponse(DisconnectResponse),
    PingRequest(PingRequest),
    PingResponse(PingResponse),
    DeviceInfoRequest(DeviceInfoRequest),
    DeviceInfoResponse(DeviceInfoResponse),
    ListEntitiesRequest(ListEntitiesRequest),
    ListEntitiesButtonResponse(ListEntitiesButtonResponse),
    ListEntitiesDoneResponse(ListEntitiesDoneResponse),
    SubscribeStatesRequest(SubscribeStatesRequest),
    SubscribeHomeassistantServicesRequest(SubscribeHomeassistantServicesRequest),
    SubscribeHomeAssistantStatesRequest(SubscribeHomeAssistantStatesRequest),
    ButtonCommandRequest(ButtonCommandRequest)
}

impl ProtoMessage {
    fn message_id(&self) -> u64 {
        match self {
            Self::HelloRequest(_) => 1,
            Self::HelloResponse(_) => 2,
            Self::AuthenticationRequest(_) => 3,
            Self::AuthenticationResponse(_) => 4,
            Self::DisconnectRequest(_) => 5,
            Self::DisconnectResponse(_) => 6,
            Self::PingRequest(_) => 7,
            Self::PingResponse(_) => 8,
            Self::DeviceInfoRequest(_) => 9,
            Self::DeviceInfoResponse(_) => 10,
            Self::ListEntitiesRequest(_) => 11,
            Self::ListEntitiesDoneResponse(_) => 19,
            Self::ListEntitiesButtonResponse(_) => 61,
            Self::SubscribeStatesRequest(_) => 20,
            Self::SubscribeHomeassistantServicesRequest(_) => 34,
            Self::SubscribeHomeAssistantStatesRequest(_) => 38,
            Self::ButtonCommandRequest(_) => 62,
        }
    }

    pub fn decode<B: Buf>(message_id: u64, buffer: &mut B) -> Result<Self> {
        match message_id {
            1 => Ok(ProtoMessage::HelloRequest(HelloRequest::decode(buffer)?)),
            2 => Ok(ProtoMessage::HelloResponse(HelloResponse::decode(buffer)?)),
            3 => Ok(ProtoMessage::AuthenticationRequest(AuthenticationRequest::decode(buffer)?)),
            4 => Ok(ProtoMessage::AuthenticationResponse(AuthenticationResponse::decode(buffer)?)),
            5 => Ok(ProtoMessage::DisconnectRequest(DisconnectRequest::decode(buffer)?)),
            6 => Ok(ProtoMessage::DisconnectResponse(DisconnectResponse::decode(buffer)?)),
            7 => Ok(ProtoMessage::PingRequest(PingRequest::decode(buffer)?)),
            8 => Ok(ProtoMessage::PingResponse(PingResponse::decode(buffer)?)),
            9 => Ok(ProtoMessage::DeviceInfoRequest(DeviceInfoRequest::decode(buffer)?)),
            10 => Ok(ProtoMessage::DeviceInfoResponse(DeviceInfoResponse::decode(buffer)?)),
            11 => Ok(ProtoMessage::ListEntitiesRequest(ListEntitiesRequest::decode(buffer)?)),
            19 => Ok(ProtoMessage::ListEntitiesDoneResponse(ListEntitiesDoneResponse::decode(buffer)?)),
            20 => Ok(ProtoMessage::SubscribeStatesRequest(SubscribeStatesRequest::decode(buffer)?)),
            34 => Ok(ProtoMessage::SubscribeHomeassistantServicesRequest(SubscribeHomeassistantServicesRequest::decode(buffer)?)),
            38 => Ok(ProtoMessage::SubscribeHomeAssistantStatesRequest(SubscribeHomeAssistantStatesRequest::decode(buffer)?)),
            61 => Ok(ProtoMessage::ListEntitiesButtonResponse(ListEntitiesButtonResponse::decode(buffer)?)),
            62 => Ok(ProtoMessage::ButtonCommandRequest(ButtonCommandRequest::decode(buffer)?)),
            _ => Err(anyhow!("Unhandled message id {}", message_id))
        }
    }
}

pub trait MessageId {
    fn message_id() -> u64;
}

impl MessageId for HelloResponse {
    fn message_id() -> u64 { 2 }
}

impl MessageId for AuthenticationResponse {
    fn message_id() -> u64 { 4 }
}

impl MessageId for DisconnectResponse {
    fn message_id() -> u64 { 6 }
}

impl MessageId for PingResponse {
    fn message_id() -> u64 { 8 }
}

impl MessageId for DeviceInfoResponse {
    fn message_id() -> u64 { 10 }
}

impl MessageId for ListEntitiesDoneResponse {
    fn message_id() -> u64 { 19 }
}

impl MessageId for ListEntitiesButtonResponse {
    fn message_id() -> u64 { 61 }
}

fn encode_frame<M, B>(message: &M, buffer: &mut B) -> Result<()>
    where M: Message + MessageId, B: BufMut
{
    let message_id = M::message_id();
    let message_len = message.encoded_len();

    buffer.put_u8(0u8);
    encode_varint(message_len as u64, buffer);
    encode_varint(message_id, buffer);
    message.encode(buffer)?;

    Ok(())
}

#[derive(Debug)]
pub struct Response<M> {
    pub type_id: u64,
    pub message: M
}

// Decode:
// Give message ID, get type that impl Message trait
// * match block on message ID, with arms calling message type decode()
// * BUT after decode, I need to match against the message type
// Encode:
// Given type that impl Message trait, get message ID
// * new trait with message_id() impl (no &self arg)

pub fn read_request<R>(stream: &mut R) -> Result<ProtoMessage>
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

    let mut buffer = if message_size > 0 {
        let buf = stream.fill_buf()?;
        if buf.len() < message_size {
            return Err(anyhow!("Buffer underrun; buf {}, message {}", buf.len(), message_size));
        }

        Bytes::copy_from_slice(&buf[..message_size])
    } else {
        Bytes::new()
    };

    println!("Message buffer {} - {:02x?}", buffer.len(), &buffer[..]);

    let message = ProtoMessage::decode(frame.type_id, &mut buffer)?;
    stream.consume(message_size);

    Ok(message)
}

pub fn send_response<S, M>(stream: &mut S, message: &M) -> Result<()>
    where S: Write, M: Message + MessageId
{
    let mut buffer = BytesMut::with_capacity(512);
    encode_frame(message, &mut buffer)?;

    let buf = buffer.freeze();
    let sz = stream.write(&buf)?;
    println!("Write {} bytes", sz);

    Ok(())
}
