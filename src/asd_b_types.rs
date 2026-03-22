
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MessageType {
    MSG,
    SEL,
    ID,
    AIR,
    STA,
    CLK,
}

impl MessageType {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "MSG" => Ok(MessageType::MSG),
            "SEL" => Ok(MessageType::SEL),
            "ID" => Ok(MessageType::ID),
            "AIR" => Ok(MessageType::AIR),
            "STA" => Ok(MessageType::STA),
            "CLK" => Ok(MessageType::CLK),
            other => Err(format!("Unknown message kind: {other}")),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum TransmissionType {
    EsIdentCat,
    EsSurfacePos,
    EsAirbornePos,
    EsAirbordSpeed,
    SurveillanceAlt,
    SurveillanceId,
    AirToAir,
    AllCall,
}

impl TransmissionType {
    fn parse(value: usize) -> Result<Self, String> {
        match value {
            1 => Ok(TransmissionType::EsIdentCat),
            2 => Ok(TransmissionType::EsSurfacePos),
            3 => Ok(TransmissionType::EsAirbornePos),
            4 => Ok(TransmissionType::EsAirbordSpeed),
            5 => Ok(TransmissionType::SurveillanceAlt),
            6 => Ok(TransmissionType::SurveillanceId),
            7 => Ok(TransmissionType::AirToAir),
            8 => Ok(TransmissionType::AllCall),
            num => Err(format!("Unknown MSG subtype: {num}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Msg {
    Msg1 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        call_sign: String,
    },
    Msg2 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        altitute: String,
        ground_speed: String,
        track: String,
        latitude: String,
        longitude: String,
        is_on_ground: String,
    },
    Msg3 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        altitute: String,
        latitude: String,
        longitude: String,
        alert: String,
        emergency: String,
        spi: String,
        is_on_ground: String,
    },
    Msg4 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        ground_speed: String,
        track: String,
        vertical_rate: String,
    },
    Msg5 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        altitute: String,
        alert: String,
        spi: String,
        is_on_ground: String,
    },
    Msg6 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        altitute: String,
        squak: String,
        alert: String,
        emergency: String,
        spi: String,
        is_on_ground: String,
    },
    Msg7 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        altitute: String,
        is_on_ground: String,
    },
    Msg8 {
        message_type: MessageType,
        transmission_type: TransmissionType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        is_on_ground: String,
    },
}

impl Msg {
    fn field(fields: &[String], idx: usize) -> String {
        fields.get(idx).cloned().unwrap_or_default()
    }

    fn parse_header(fields: &[String]) -> Result<(MessageType, TransmissionType), String> {
        let raw_message_type = fields
            .first()
            .ok_or_else(|| "Missing message type field".to_string())?;
        let message_type = MessageType::parse(raw_message_type)?;
        if message_type != MessageType::MSG {
            return Err(format!("Msg only supports MSG rows, got {:?}", message_type));
        }

        let raw_transmission_type = fields
            .get(1)
            .ok_or_else(|| "Missing transmission type field".to_string())?;
        let transmission_type: usize = raw_transmission_type
            .parse()
            .map_err(|_| format!("Invalid transmission type: {raw_transmission_type}"))?;

        Ok((message_type, TransmissionType::parse(transmission_type)?))
    }

    fn from_iter<I, S>(iter: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let fields: Vec<String> = iter.into_iter().map(|s| s.as_ref().to_string()).collect();
        let (message_type, transmission_type) = Self::parse_header(&fields)?;

        let session_id = Self::field(&fields, 2);
        let aircraft_id = Self::field(&fields, 3);
        let hex_ident = Self::field(&fields, 4);
        let flight_id = Self::field(&fields, 5);
        let creation_date = Self::field(&fields, 6);
        let creation_time = Self::field(&fields, 7);
        let logged_date = Self::field(&fields, 8);
        let logged_timed = Self::field(&fields, 9);

        match transmission_type {
            TransmissionType::EsIdentCat => Ok(Msg::Msg1 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                call_sign: Self::field(&fields, 10),
            }),
            TransmissionType::EsSurfacePos => Ok(Msg::Msg2 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                altitute: Self::field(&fields, 11),
                ground_speed: Self::field(&fields, 12),
                track: Self::field(&fields, 13),
                latitude: Self::field(&fields, 14),
                longitude: Self::field(&fields, 15),
                is_on_ground: Self::field(&fields, 21),
            }),
            TransmissionType::EsAirbornePos => Ok(Msg::Msg3 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                altitute: Self::field(&fields, 11),
                latitude: Self::field(&fields, 14),
                longitude: Self::field(&fields, 15),
                alert: Self::field(&fields, 18),
                emergency: Self::field(&fields, 19),
                spi: Self::field(&fields, 20),
                is_on_ground: Self::field(&fields, 21),
            }),
            TransmissionType::EsAirbordSpeed => Ok(Msg::Msg4 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                ground_speed: Self::field(&fields, 12),
                track: Self::field(&fields, 13),
                vertical_rate: Self::field(&fields, 16),
            }),
            TransmissionType::SurveillanceAlt => Ok(Msg::Msg5 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                altitute: Self::field(&fields, 11),
                alert: Self::field(&fields, 18),
                spi: Self::field(&fields, 20),
                is_on_ground: Self::field(&fields, 21),
            }),
            TransmissionType::SurveillanceId => Ok(Msg::Msg6 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                altitute: Self::field(&fields, 11),
                squak: Self::field(&fields, 17),
                alert: Self::field(&fields, 18),
                emergency: Self::field(&fields, 19),
                spi: Self::field(&fields, 20),
                is_on_ground: Self::field(&fields, 21),
            }),
            TransmissionType::AirToAir => Ok(Msg::Msg7 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                altitute: Self::field(&fields, 11),
                is_on_ground: Self::field(&fields, 21),
            }),
            TransmissionType::AllCall => Ok(Msg::Msg8 {
                message_type,
                transmission_type,
                session_id,
                aircraft_id,
                hex_ident,
                flight_id,
                creation_date,
                creation_time,
                logged_date,
                logged_timed,
                is_on_ground: Self::field(&fields, 21),
            }),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message {
    MSG {
        msg: Msg,
    },
    SEL {
        message_type: MessageType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        call_sign: String,
    },
    ID {
        message_type: MessageType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        call_sign: String,
    },
    AIR {
        message_type: MessageType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
    },
    STA {
        message_type: MessageType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
        satus: String,
    },
    CLK {
        message_type: MessageType,
        session_id: String,
        aircraft_id: String,
        hex_ident: String,
        flight_id: String,
        creation_date: String,
        creation_time: String,
        logged_date: String,
        logged_timed: String,
    },
}

impl Message {
    fn field(fields: &[String], idx: usize) -> String {
        fields.get(idx).cloned().unwrap_or_default()
    }

    // Non-MSG rows usually start payload at index 2; SEL samples sometimes include an extra empty slot.
    fn non_msg_base_index(fields: &[String]) -> usize {
        if Self::field(fields, 2).is_empty() && !Self::field(fields, 3).is_empty() {
            3
        } else {
            2
        }
    }

    fn from_iter<I, S>(iter: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let fields: Vec<String> = iter.into_iter().map(|s| s.as_ref().to_string()).collect();
        let raw_type = fields
            .first()
            .ok_or_else(|| "Missing message type field".to_string())?;
        let message_type = MessageType::parse(raw_type)?;

        match message_type {
            MessageType::MSG => Ok(Message::MSG {
                msg: Msg::from_iter(fields.iter().map(|s| s.as_str()))?,
            }),
            MessageType::SEL => {
                let i = Self::non_msg_base_index(&fields);
                Ok(Message::SEL {
                    message_type,
                    session_id: Self::field(&fields, i),
                    aircraft_id: Self::field(&fields, i + 1),
                    hex_ident: Self::field(&fields, i + 2),
                    flight_id: Self::field(&fields, i + 3),
                    creation_date: Self::field(&fields, i + 4),
                    creation_time: Self::field(&fields, i + 5),
                    logged_date: Self::field(&fields, i + 6),
                    logged_timed: Self::field(&fields, i + 7),
                    call_sign: Self::field(&fields, i + 8),
                })
            }
            MessageType::ID => {
                let i = Self::non_msg_base_index(&fields);
                Ok(Message::ID {
                    message_type,
                    session_id: Self::field(&fields, i),
                    aircraft_id: Self::field(&fields, i + 1),
                    hex_ident: Self::field(&fields, i + 2),
                    flight_id: Self::field(&fields, i + 3),
                    creation_date: Self::field(&fields, i + 4),
                    creation_time: Self::field(&fields, i + 5),
                    logged_date: Self::field(&fields, i + 6),
                    logged_timed: Self::field(&fields, i + 7),
                    call_sign: Self::field(&fields, i + 8),
                })
            }
            MessageType::AIR => {
                let i = Self::non_msg_base_index(&fields);
                Ok(Message::AIR {
                    message_type,
                    session_id: Self::field(&fields, i),
                    aircraft_id: Self::field(&fields, i + 1),
                    hex_ident: Self::field(&fields, i + 2),
                    flight_id: Self::field(&fields, i + 3),
                    creation_date: Self::field(&fields, i + 4),
                    creation_time: Self::field(&fields, i + 5),
                    logged_date: Self::field(&fields, i + 6),
                    logged_timed: Self::field(&fields, i + 7),
                })
            }
            MessageType::STA => {
                let i = Self::non_msg_base_index(&fields);
                Ok(Message::STA {
                    message_type,
                    session_id: Self::field(&fields, i),
                    aircraft_id: Self::field(&fields, i + 1),
                    hex_ident: Self::field(&fields, i + 2),
                    flight_id: Self::field(&fields, i + 3),
                    creation_date: Self::field(&fields, i + 4),
                    creation_time: Self::field(&fields, i + 5),
                    logged_date: Self::field(&fields, i + 6),
                    logged_timed: Self::field(&fields, i + 7),
                    satus: Self::field(&fields, i + 8),
                })
            }
            MessageType::CLK => {
                let i = Self::non_msg_base_index(&fields);
                Ok(Message::CLK {
                    message_type,
                    session_id: Self::field(&fields, i),
                    aircraft_id: Self::field(&fields, i + 1),
                    hex_ident: Self::field(&fields, i + 2),
                    flight_id: Self::field(&fields, i + 3),
                    creation_date: Self::field(&fields, i + 4),
                    creation_time: Self::field(&fields, i + 5),
                    logged_date: Self::field(&fields, i + 6),
                    logged_timed: Self::field(&fields, i + 7),
                })
            }
        }
    }

    pub fn parse(input: &str) -> Vec<Result<Self, String>> {
        input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| Self::from_iter(line.split(',')))
            .collect()
    }

    pub fn hex_ident(&self) -> String {
        match self {
            Self::MSG { msg } => {
                match msg {
                    Msg::Msg1 { hex_ident, .. } => hex_ident.clone(),
                    Msg::Msg2 { hex_ident, .. } => hex_ident.clone(),
                    Msg::Msg3 { hex_ident, .. } => hex_ident.clone(),
                    Msg::Msg4 { hex_ident, .. } => hex_ident.clone(),
                    Msg::Msg5 { hex_ident, .. } => hex_ident.clone(),
                    Msg::Msg6 { hex_ident, .. } => hex_ident.clone(),
                    Msg::Msg7 { hex_ident, .. } => hex_ident.clone(),
                    Msg::Msg8 { hex_ident, .. } => hex_ident.clone()
                }
            }
            Self::SEL { hex_ident, .. } |
            Self::ID { hex_ident, ..} |
            Self::AIR { hex_ident, ..} |
            Self::STA { hex_ident, ..} |
            Self::CLK { hex_ident, ..} => { hex_ident.clone() }
        }
    }
}
