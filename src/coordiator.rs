use crate::hex_db::get_flight_info;
use crate::sbs1_ingest::read_ssb1_stream;
use crate::sbs1_ingest::{IngestCommand, SSB1StreamEvent};
use crate::{AppCommand, AppEvent};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub async fn run_coordinator(
    address: String,
    mut command_rx: UnboundedReceiver<AppCommand>,
    event_tx: UnboundedSender<AppEvent>,
) {
    let (ingest_control_tx, ingest_control_rx) = unbounded_channel::<IngestCommand>();
    let (ssb1_stream_event_tx, mut ssb1_stream_event_rx) = unbounded_channel::<SSB1StreamEvent>();

    let ingest_handle = tokio::spawn(read_ssb1_stream(
        address,
        ingest_control_rx,
        ssb1_stream_event_tx.clone(),
    ));

    loop {
        tokio::select! {
            Some(command) = command_rx.recv() => {
                match command {
                    AppCommand::Quit => {
                        ingest_control_tx.send(IngestCommand::Stop).unwrap_or_default();
                        break
                    }
                    AppCommand::FetchFlightInfo{ hex_id } => {
                        let result = get_flight_info(hex_id).await.map_err(anyhow::Error::from);
                        // Send the result regardless of Ok/Err so the UI/State can react to the failure
                        let _ = event_tx.send(AppEvent::FlightInfoEvent(result));
                    }
                }
            }

            Some(ingest_event) = ssb1_stream_event_rx.recv() => {
                event_tx.send(AppEvent::Ingest(ingest_event)).unwrap_or(())
            }
        }
    }

    _ = ingest_handle.await;
}
