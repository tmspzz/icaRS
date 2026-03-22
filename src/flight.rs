#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Flight {
    pub asdb_hex_id: String,
    pub callsign: Option<String>,
    pub altitute: Option<String>,
    pub ground_speed: Option<String>,
    pub track: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub vertical_rate: Option<String>,
    pub squak: Option<String>,
    pub emergency: bool,
    pub is_on_ground: bool,
}

impl Flight {
    pub fn new(asdb_hex_id: impl Into<String>) -> Self {
        Self {
            asdb_hex_id: asdb_hex_id.into(),
            callsign: None,
            altitute: None,
            ground_speed: None,
            track: None,
            latitude: None,
            longitude: None,
            vertical_rate: None,
            squak: None,
            emergency: false,
            is_on_ground: false,
        }
    }

    pub fn asdb_hex_id(&self) -> &str {
        &self.asdb_hex_id
    }

    pub fn callsign(&self) -> Option<&str> {
        self.callsign.as_deref()
    }

    pub fn altitute(&self) -> Option<&str> {
        self.altitute.as_deref()
    }

    pub fn ground_speed(&self) -> Option<&str> {
        self.ground_speed.as_deref()
    }

    pub fn track(&self) -> Option<&str> {
        self.track.as_deref()
    }

    pub fn latitude(&self) -> Option<&str> {
        self.latitude.as_deref()
    }

    pub fn longitude(&self) -> Option<&str> {
        self.longitude.as_deref()
    }

    pub fn vertical_rate(&self) -> Option<&str> {
        self.vertical_rate.as_deref()
    }

    pub fn squak(&self) -> Option<&str> {
        self.squak.as_deref()
    }
}
