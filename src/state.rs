use time::OffsetDateTime;

use crate::asd_b_types::Message;
use crate::flight::Flight;
use crate::hex_db::FlightInfo;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AppState {
    pub(crate) flight_store: HashMap<String, TimeStampedFlight>,
    pub(crate) connected: bool,
    pub(crate) ui_state: UIState,
    pub(crate) last_error: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UIState {
    pub(crate) selected: usize,
    pub(crate) info_panel_state: InfoPanelState,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InfoPanelState {
    Loading,
    Open { model: FlightInfo },
    Error(String),
    Closed,
}

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct InfoPanelState {
//     flight_info: FlightInfo,
// }

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TimeStampedFlight {
    pub flight: Flight,
    pub last_seen: OffsetDateTime,
    pub first_seen: OffsetDateTime,
    pub num_messages: u32,
}

impl TimeStampedFlight {
    pub fn new(asdb_hex_id: String) -> Self {
        TimeStampedFlight {
            flight: Flight::new(asdb_hex_id),
            last_seen: OffsetDateTime::now_utc(),
            first_seen: OffsetDateTime::now_utc(),
            num_messages: 0,
        }
    }

    pub fn update_from_message(&mut self, message: &Message) {
        self.flight.update_from_message(message);
        self.last_seen = OffsetDateTime::now_utc();
        self.num_messages += 1;
    }

    pub fn flight(&self) -> &Flight {
        &self.flight
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            flight_store: HashMap::new(),
            connected: false,
            ui_state: UIState {
                selected: 0,
                info_panel_state: InfoPanelState::Closed,
            },
            last_error: None,
        }
    }

    pub fn update_from_message(&mut self, message: Message) {
        let hex_ident = message.hex_ident();

        match self.flight_store.entry(hex_ident) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().update_from_message(&message);
            }
            Entry::Vacant(entry) => {
                let mut flight = TimeStampedFlight::new(entry.key().clone());
                flight.update_from_message(&message);
                entry.insert(flight);
            }
        };
    }

    pub fn sorted_flights(&self) -> Vec<&TimeStampedFlight> {
        let mut flights: Vec<&TimeStampedFlight> = self.flight_store.values().collect();
        flights.sort_unstable_by(|a, b| a.first_seen.cmp(&b.first_seen));
        flights
    }

    pub fn clamp_selection(&mut self) {
        let len = self.flight_store.len();
        if len == 0 {
            self.ui_state.selected = 0;
        } else if self.ui_state.selected >= len {
            self.ui_state.selected = len - 1;
        }
    }

    pub fn select_next(&mut self) {
        let len = self.flight_store.len();
        if len == 0 {
            self.ui_state.selected = 0;
            return;
        }
        self.ui_state.selected = (self.ui_state.selected + 1) % len;
    }

    pub fn select_prev(&mut self) {
        let len = self.flight_store.len();
        if len == 0 {
            self.ui_state.selected = 0;
            return;
        }
        self.ui_state.selected = if self.ui_state.selected == 0 {
            len - 1
        } else {
            self.ui_state.selected - 1
        };
    }
}
