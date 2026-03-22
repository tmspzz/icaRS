use crate::asd_b_types::{Message, Msg};
use crate::flight::Flight;

impl Flight {
    pub fn new_from_message(message: Message) -> Self {
        Self::new(message.hex_ident())
    }

    pub fn update_from_message(&mut self, message: &Message) {
        #[inline]
        fn to_flag(value: &str) -> bool {
            matches!(value, "1" | "true" | "TRUE" | "True")
        }

        #[inline]
        fn update_opt_string(field: &mut Option<String>, value: &str) {
            if value.is_empty() {
                return;
            }
            match field {
                Some(current) if current == value => {}
                _ => *field = Some(value.to_owned()),
            }
        }

        #[inline]
        fn update_bool(field: &mut bool, value: bool) {
            if *field != value {
                *field = value;
            }
        }

        match message {
            Message::MSG { msg } => match msg {
                Msg::Msg1 {
                    hex_ident,
                    call_sign,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_opt_string(&mut self.callsign, call_sign.as_str());
                }
                Msg::Msg2 {
                    hex_ident,
                    altitute,
                    ground_speed,
                    track,
                    latitude,
                    longitude,
                    is_on_ground,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_opt_string(&mut self.altitute, altitute.as_str());
                    update_opt_string(&mut self.ground_speed, ground_speed.as_str());
                    update_opt_string(&mut self.track, track.as_str());
                    update_opt_string(&mut self.latitude, latitude.as_str());
                    update_opt_string(&mut self.longitude, longitude.as_str());
                    update_bool(&mut self.is_on_ground, to_flag(is_on_ground.as_str()));
                }
                Msg::Msg3 {
                    hex_ident,
                    altitute,
                    latitude,
                    longitude,
                    emergency,
                    is_on_ground,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_opt_string(&mut self.altitute, altitute.as_str());
                    update_opt_string(&mut self.latitude, latitude.as_str());
                    update_opt_string(&mut self.longitude, longitude.as_str());
                    update_bool(&mut self.emergency, to_flag(emergency.as_str()));
                    update_bool(&mut self.is_on_ground, to_flag(is_on_ground.as_str()));
                }
                Msg::Msg4 {
                    hex_ident,
                    ground_speed,
                    track,
                    vertical_rate,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_opt_string(&mut self.ground_speed, ground_speed.as_str());
                    update_opt_string(&mut self.track, track.as_str());
                    update_opt_string(&mut self.vertical_rate, vertical_rate.as_str());
                }
                Msg::Msg5 {
                    hex_ident,
                    altitute,
                    is_on_ground,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_opt_string(&mut self.altitute, altitute.as_str());
                    update_bool(&mut self.is_on_ground, to_flag(is_on_ground.as_str()));
                }
                Msg::Msg6 {
                    hex_ident,
                    altitute,
                    squak,
                    emergency,
                    is_on_ground,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_opt_string(&mut self.altitute, altitute.as_str());
                    update_opt_string(&mut self.squak, squak.as_str());
                    update_bool(&mut self.emergency, to_flag(emergency.as_str()));
                    update_bool(&mut self.is_on_ground, to_flag(is_on_ground.as_str()));
                }
                Msg::Msg7 {
                    hex_ident,
                    altitute,
                    is_on_ground,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_opt_string(&mut self.altitute, altitute.as_str());
                    update_bool(&mut self.is_on_ground, to_flag(is_on_ground.as_str()));
                }
                Msg::Msg8 {
                    hex_ident,
                    is_on_ground,
                    ..
                } => {
                    debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                    if self.asdb_hex_id != hex_ident.as_str() {
                        return;
                    }
                    update_bool(&mut self.is_on_ground, to_flag(is_on_ground.as_str()));
                }
            },
            Message::SEL {
                hex_ident,
                call_sign,
                ..
            }
            | Message::ID {
                hex_ident,
                call_sign,
                ..
            } => {
                debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                if self.asdb_hex_id != hex_ident.as_str() {
                    return;
                }
                update_opt_string(&mut self.callsign, call_sign.as_str());
            }
            Message::AIR { hex_ident, .. }
            | Message::STA { hex_ident, .. }
            | Message::CLK { hex_ident, .. } => {
                debug_assert_eq!(self.asdb_hex_id, *hex_ident);
                if self.asdb_hex_id != hex_ident.as_str() {
                    return;
                }
            }
        }
    }
}
