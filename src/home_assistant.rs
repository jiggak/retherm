/*
 * ReTherm - Home Assistant native interface for Gen2 Nest thermostat
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

use std::{sync::{Arc, Mutex}, thread::{self, JoinHandle}};

use anyhow::Result;
use esphome_api::{
    proto::*,
    server::{
        DefaultHandler, MessageSenderThread, MessageStreamProvider,
        MessageThreadError, RequestHandler, ResponseStatus, start_server
    }
};

use crate::{
    config::HomeAssistantConfig,
    events::{Event, EventHandler, EventSender},
    state::ThermostatState
};

pub struct HomeAssistant {
    message_sender: MessageSenderThread,
    state: Arc<Mutex<ThermostatState>>
}

impl HomeAssistant {
    pub fn new() -> Self {
        Self {
            message_sender: MessageSenderThread::new(),
            state: Arc::new(Mutex::new(ThermostatState::default()))
        }
    }

    pub fn start_listener<S>(
        &self,
        config: &HomeAssistantConfig,
        stream_provider: impl MessageStreamProvider<S> + Send + 'static,
        event_sender: impl EventSender + Send + 'static
    ) -> JoinHandle<Result<()>>
        where S: MessageStream + Send + 'static
    {
        let addr = config.listen_addr.clone();

        let connection_observer = self.message_sender.clone();

        let handler = DefaultHandler {
            delegate: HvacRequestHandler::new(event_sender, self.state.clone()),
            server_info: config.server_info.clone(),
            node_name: config.node_name.clone(),
            friendly_name: config.friendly_name.clone(),
            manufacturer: config.manufacturer.clone(),
            model: config.model.clone(),
            mac_address: "01:02:03:04:05:06".to_string()
        };

        thread::spawn(move || {
            start_server(addr, &stream_provider, &connection_observer, &handler)
        })
    }
}

impl EventHandler for HomeAssistant {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::State(state) = event {
            *self.state.lock().unwrap() = state.clone();

            let message = ProtoMessage::ClimateStateResponse(state.into());

            let result = self.message_sender.send_message(message);
            match result {
                // Ignoring non-connected errors
                Err(MessageThreadError::NonConnected) => { },
                r => r?
            }
        }

        Ok(())
    }
}

struct HvacRequestHandler<S> {
    event_sender: S,
    state: Arc<Mutex<ThermostatState>>
}

impl<S: EventSender> HvacRequestHandler<S> {
    fn new(event_sender: S, state: Arc<Mutex<ThermostatState>>) -> Self {
        Self { event_sender, state }
    }
}

impl<S: EventSender> RequestHandler for HvacRequestHandler<S> {
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
                    ClimateMode::Cool as i32
                ];
                message.visual_min_temperature = ThermostatState::MIN_TEMP;
                message.visual_max_temperature = ThermostatState::MAX_TEMP;
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
                message.supported_presets = vec![
                    ClimatePreset::None as i32,
                    ClimatePreset::Eco as i32
                ];

                writer.write(&ProtoMessage::ListEntitiesClimateResponse(message))?;

                let message = ListEntitiesDoneResponse::default();
                writer.write(&ProtoMessage::ListEntitiesDoneResponse(message))?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                let state = self.state.lock().unwrap().clone();
                writer.write(&ProtoMessage::ClimateStateResponse(state.into()))?;
            }
            ProtoMessage::ClimateCommandRequest(cmd) => {
                if cmd.has_mode {
                    let mode = cmd.mode().try_into()?;
                    self.event_sender.send_event(Event::SetMode(mode))?;
                }
                if cmd.has_target_temperature {
                    let temp = cmd.target_temperature;
                    self.event_sender.send_event(Event::SetTargetTemp(temp))?;
                }
                if cmd.has_preset {
                    match cmd.preset() {
                        ClimatePreset::Eco => {
                            self.event_sender.send_event(Event::SetAway(true))?;
                        }
                        _ => {
                            self.event_sender.send_event(Event::SetAway(false))?;
                        }
                    }
                }
            }
            _ => { }
        }

        Ok(ResponseStatus::Continue)
    }
}
