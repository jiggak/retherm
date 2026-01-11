/*
 * Nest UI - Home Assistant native thermostat interface
 * Copyright (C) 2026 Josh Kropf <josh@slashdev.ca>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{sync::{Arc, Mutex, mpsc::channel}, thread::{self, JoinHandle}};

use anyhow::Result;
use esphome_api::{
    proto::*,
    server::{
        DefaultHandler, MessageStreamFactory, RequestHandler, ResponseStatus, start_server
    }
};

use crate::events::{Event, EventHandler, EventSender};

pub struct HomeAssistant<S, M> {
    event_sender: S,
    message_stream: Arc<Mutex<Option<M>>>
}

impl<S: EventSender, M: MessageStream + Send + 'static> HomeAssistant<S, M> {
    pub fn new(event_sender: S) -> Self {
        Self {
            event_sender,
            message_stream: Arc::new(Mutex::new(None))
        }
    }

    fn send_message<A>(&self, message: &A) -> Result<()>
        where A: Message + MessageId
    {
        let mut stream = self.message_stream.lock().unwrap();
        if let Some(stream) = stream.as_mut() {
            stream.write(message)?;
        } else {
            println!("Message stream is not available");
        }

        Ok(())
    }

    pub fn start_listener<F>(&self, addr: &str, stream_factory: F) -> JoinHandle<Result<()>>
        where F: MessageStreamFactory<M> + Send + 'static
    {
        let addr = addr.to_string();

        let (stream_sender, stream_receiver) = channel();
        let message_stream = self.message_stream.clone();

        thread::spawn(move || {
            while let Ok(stream) = stream_receiver.recv() {
                let mut guard = message_stream.lock().unwrap();
                *guard = stream;
            }
        });

        let handler = DefaultHandler {
            delegate: MyHandler,
            server_info: "Nest App 0.0.1".to_string(),
            node_name: "test-thermostat".to_string(),
            friendly_name: "Test Thermostat".to_string(),
            manufacturer: "Nest".to_string(),
            model: "Gen2 Thermostat".to_string(),
            mac_address: "01:02:03:04:05:06".to_string()
        };

        thread::spawn(move || {
            start_server(addr, &stream_factory, stream_sender, &handler)
        })
    }
}

impl<S: EventSender, M: MessageStream + Send + 'static> EventHandler for HomeAssistant<S, M> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Temp(temp) = event {
            let mut message = ClimateStateResponse::default();
            message.set_action(ClimateAction::Idle);
            message.set_fan_mode(ClimateFanMode::ClimateFanAuto);
            message.set_mode(ClimateMode::Heat);
            message.current_temperature = *temp;
            message.target_temperature = 19.5;
            self.send_message(&message)?;
        }
        // receive specific event, send message to HA client
        Ok(())
    }
}

impl<S: EventSender, M: MessageStream + Send + 'static> RequestHandler for HomeAssistant<S, M> {
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

                writer.write(&message)?;

                writer.write(&ListEntitiesDoneResponse::default())?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                let mut state = ClimateStateResponse::default();
                state.set_action(ClimateAction::Idle);
                state.set_fan_mode(ClimateFanMode::ClimateFanAuto);
                state.set_mode(ClimateMode::Heat);
                state.current_temperature = 20.0;
                state.target_temperature = 19.5;

                writer.write(&state)?;
            }
            ProtoMessage::ClimateCommandRequest(_cmd) => {
                self.event_sender.send_event(Event::HVAC)?;
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}

struct MyHandler;

impl RequestHandler for MyHandler {
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

                writer.write(&message)?;

                writer.write(&ListEntitiesDoneResponse::default())?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                let mut state = ClimateStateResponse::default();
                state.set_action(ClimateAction::Idle);
                state.set_fan_mode(ClimateFanMode::ClimateFanAuto);
                state.set_mode(ClimateMode::Heat);
                state.current_temperature = 20.0;
                state.target_temperature = 19.5;

                writer.write(&state)?;
            }
            ProtoMessage::ClimateCommandRequest(_cmd) => {
                // self.event_sender.send_event(Event::HVAC)?;
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}
