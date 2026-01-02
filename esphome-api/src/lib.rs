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
    // zero byte + 2 varint (10 bytes each)
    const MAX_PREAMBLE_SIZE: usize = 21;

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
pub enum Request {
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

pub fn read_request<R>(stream: &mut R) -> Result<Request>
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

pub fn send_response<S, M>(stream: &mut S, response: Response<M>) -> Result<()>
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
