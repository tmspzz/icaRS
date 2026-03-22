use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
pub struct FlightInfo {
    #[serde(rename = "ICAOTypeCode")]
    pub icao_type_code: String,
    #[serde(rename = "Manufacturer")]
    pub manufacturer: String,
    #[serde(rename = "ModeS")]
    pub mode_s: String,
    #[serde(rename = "OperatorFlagCode")]
    pub operator_flag_code: String,
    #[serde(rename = "RegisteredOwners")]
    pub registered_owners: String,
    #[serde(rename = "Registration")]
    pub registration: String,
    #[serde(rename = "Type")]
    pub aircraft_type: String,
}

#[derive(Debug, Error)]
pub enum HexDbError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),
}

pub async fn get_flight_info(asdb_hex_id: String) -> Result<FlightInfo, HexDbError> {
    let client = Client::new();
    let response = client
        .get(format!("https://hexdb.io/api/v1/aircraft/{}", asdb_hex_id))
        .send()
        .await?;
    let flight_info = response.json::<FlightInfo>().await?;
    println!("{:?}", flight_info);
    Ok(flight_info)
}
