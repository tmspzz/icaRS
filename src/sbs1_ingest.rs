use crate::asd_b_types::Message;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IngestCommand {
    Stop,
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SSB1StreamEvent {
    Connected,
    Disconnected,
    ConnectionError(String),
    Message(Message),
    ParseError(String),
}

pub async fn read_ssb1_stream(
    address: String,
    mut rx: UnboundedReceiver<IngestCommand>,
    tx: UnboundedSender<SSB1StreamEvent>,
) {
    let mut stream = match TcpStream::connect(&address).await {
        Ok(stream) => {
            let _ = tx.send(SSB1StreamEvent::Connected);
            stream
        }
        Err(error) => {
            let _ = tx.send(SSB1StreamEvent::ConnectionError(error.to_string()));
            let _ = tx.send(SSB1StreamEvent::Disconnected);
            return;
        }
    };

    let mut read_buf = [0_u8; 4096];
    let mut pending = String::new();

    loop {
        select! {
            command = rx.recv() => {
                match command {
                    Some(IngestCommand::Stop) | None => {
                        return;
                    }
                }
            }

            read_result = stream.read(&mut read_buf) => {
                match read_result {
                    Ok(0) => {
                        let _ = tx.send(SSB1StreamEvent::Disconnected);
                        return;
                    }
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&read_buf[..n]);
                        pending.push_str(&chunk);

                        while let Some(newline_index) = pending.find('\n') {
                                let line: String = pending.drain(..=newline_index).collect();
                                let line = line.trim_end_matches(&['\r', '\n'][..]);

                                if line.is_empty() {
                                    continue;
                                }

                            for result in  Message::parse(line) {
                                match result {
                                    Ok(message) => {
                                        if tx.send(SSB1StreamEvent::Message(message)).is_err() {
                                            return;
                                        }
                                    }
                                    Err(error) => {
                                        let _ = tx.send(SSB1StreamEvent::ParseError(error));
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(SSB1StreamEvent::ConnectionError(error.to_string()));
                        let _ = tx.send(SSB1StreamEvent::Disconnected);
                        return;
                    }
                }
            }
        }
    }
}
