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

use std::thread::{self, JoinHandle};

use anyhow::Result;
use esphome_api::{
    proto::*,
    server::{
        DefaultHandler, MessageSender, MessageStreamProvider,
        MessageThreadError, RequestHandler, ResponseStatus, start_server
    }
};

use crate::{
    config::HomeAssistantConfig,
    events::{Event, EventHandler, EventSender},
    state::ThermostatState
};

pub struct HomeAssistant {
    message_sender: MessageSender
}

impl HomeAssistant {
    pub fn new() -> Self {
        Self {
            message_sender: MessageSender::new()
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

        let delegate = HvacRequestHandler::new(
            thermostat_entity(config.get_object_id()),
            event_sender
        );

        let handler = DefaultHandler {
            delegate: delegate,
            server_info: config.server_info.clone(),
            node_name: config.get_node_name(),
            friendly_name: config.friendly_name.clone(),
            manufacturer: config.manufacturer.clone(),
            model: config.model.clone(),
            mac_address: config.get_mac_address()
        };

        thread::spawn(move || {
            start_server(addr, &stream_provider, &connection_observer, &handler)
        })
    }
}

impl EventHandler for HomeAssistant {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::State(state) = event {
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
    thermostat_entity: ListEntitiesClimateResponse,
    event_sender: S
}

impl<S: EventSender> HvacRequestHandler<S> {
    fn new(thermostat_entity: ListEntitiesClimateResponse, event_sender: S) -> Self {
        Self {
            thermostat_entity,
            event_sender
        }
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
                let message = self.thermostat_entity.clone();
                writer.write(&ProtoMessage::ListEntitiesClimateResponse(message))?;

                let message = ListEntitiesDoneResponse::default();
                writer.write(&ProtoMessage::ListEntitiesDoneResponse(message))?;
            }
            ProtoMessage::SubscribeStatesRequest(_) => {
                self.event_sender.send_event(Event::GetState)?;
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
                        ClimatePreset::Away => {
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

fn thermostat_entity(object_id: String) -> ListEntitiesClimateResponse {
    let mut entity = ListEntitiesClimateResponse::default();

    entity.object_id = object_id;
    entity.supported_modes = vec![
        ClimateMode::Off as i32,
        ClimateMode::Heat as i32,
        ClimateMode::Cool as i32
    ];
    entity.visual_min_temperature = ThermostatState::MIN_TEMP;
    entity.visual_max_temperature = ThermostatState::MAX_TEMP;
    entity.visual_target_temperature_step = 0.5;
    entity.visual_current_temperature_step = 0.5;
    entity.feature_flags =
        ClimateFeature::SUPPORTS_CURRENT_TEMPERATURE |
        ClimateFeature::SUPPORTS_ACTION;
    entity.supported_presets = vec![
        ClimatePreset::None as i32,
        ClimatePreset::Away as i32
    ];

    entity
}
