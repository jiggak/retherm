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

use std::{thread::{self, JoinHandle}};

use anyhow::Result;
use esphome_api::{
    proto::*,
    server::{
        DefaultHandler, MessageSenderThread, MessageStreamProvider, RequestHandler, ResponseStatus, start_server
    }
};

use crate::events::{Event, EventHandler, EventOrigin, EventSender};

pub struct HomeAssistant<S> {
    event_sender: S,
    message_sender: MessageSenderThread
}

impl<S: EventSender> HomeAssistant<S> {
    pub fn new(event_sender: S) -> Self {
        Self {
            event_sender,
            message_sender: MessageSenderThread::new()
        }
    }

    pub fn start_listener<F, G>(&self, addr: &str, stream_factory: F) -> JoinHandle<Result<()>>
        where F: MessageStreamProvider<G> + Send + 'static, G: MessageStream + Send + 'static
    {
        let addr = addr.to_string();

        let connection_watcher = self.message_sender.clone();

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
            start_server(addr, &stream_factory, &connection_watcher, &handler)
        })
    }
}

impl<S: EventSender> EventHandler for HomeAssistant<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Hvac { state, origin } = event && origin == &EventOrigin::Backplate {
            let message = state.into();
            self.message_sender.send_message(ProtoMessage::ClimateStateResponse(message))?;
        }

        Ok(())
    }
}

impl<S: EventSender> RequestHandler for HomeAssistant<S> {
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
                let mut state = ClimateStateResponse::default();
                state.set_action(ClimateAction::Idle);
                state.set_fan_mode(ClimateFanMode::ClimateFanAuto);
                state.set_mode(ClimateMode::Heat);
                state.current_temperature = 20.0;
                state.target_temperature = 19.5;

                writer.write(&ProtoMessage::ClimateStateResponse(state))?;
            }
            ProtoMessage::ClimateCommandRequest(_cmd) => {
                // self.event_sender.send_event(Event::HVAC)?;
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

                writer.write(&ProtoMessage::ListEntitiesClimateResponse(message))?;

                let message = ListEntitiesDoneResponse::default();
                writer.write(&ProtoMessage::ListEntitiesDoneResponse(message))?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                let mut state = ClimateStateResponse::default();
                state.set_action(ClimateAction::Idle);
                state.set_fan_mode(ClimateFanMode::ClimateFanAuto);
                state.set_mode(ClimateMode::Heat);
                state.current_temperature = 20.0;
                state.target_temperature = 19.5;

                writer.write(&ProtoMessage::ClimateStateResponse(state))?;
            }
            ProtoMessage::ClimateCommandRequest(_cmd) => {
                // self.event_sender.send_event(Event::HVAC)?;
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}
