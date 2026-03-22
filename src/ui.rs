use crate::{
    sbs1_ingest::SSB1StreamEvent,
    state::{AppState, InfoPanelVisibility, TimeStampedFlight},
};
use crate::{AppCommand, AppEvent};
use crossterm::{
    event::{self, poll, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::Layout,
    prelude::{Constraint, CrosstermBackend, Direction},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Terminal,
};
use std::{io, time::Duration};
use time::UtcOffset;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

// use unicode_width::UnicodeWidthStr;
pub fn run_ui(
    mut event_rx: UnboundedReceiver<AppEvent>,
    command_tx: UnboundedSender<AppCommand>,
    app_state: &mut AppState,
) -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let polling_timeout = Duration::from_millis(100);

    loop {
        loop {
            match event_rx.try_recv() {
                Ok(AppEvent::Ingest(event)) => {
                    match event {
                        SSB1StreamEvent::Connected => {
                            // println!("{event:?}");
                            app_state.connected = true;
                        }
                        SSB1StreamEvent::ParseError(error) => println!("{error:?}"),
                        SSB1StreamEvent::ConnectionError(error) => println!("{error:?}"),
                        SSB1StreamEvent::Disconnected => app_state.connected = false,
                        SSB1StreamEvent::Message(m) => app_state.update_from_message(m),
                    }
                }
                Ok(AppEvent::FlightInfoEvent(event)) => {
                    //panic!();
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => {
                    break;
                }
            }

            // println!("{app_state:?}")
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(f.area());

            app_state.clamp_selection();
            let sorted_flights = app_state.sorted_flights();

            let selected = if sorted_flights.is_empty() {
                None
            } else {
                Some(app_state.ui_state.selected)
            };

            let mut table_state = TableState::default();
            table_state.select(selected);

            f.render_stateful_widget(
                list_table_widget(sorted_flights.clone()),
                chunks[0],
                &mut table_state,
            );

            let table_flights = selected
                .and_then(|index| sorted_flights.get(index).copied())
                .map(|flight| vec![flight])
                .unwrap_or_default();
            f.render_widget(table_widget(table_flights), chunks[1]);
        })?;

        if poll(polling_timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        print!("Got Q!");
                        _ = command_tx.send(AppCommand::Quit);
                        break;
                    }
                    KeyCode::Down => {
                        app_state.select_next();
                    }
                    KeyCode::Up => {
                        app_state.select_prev();
                    }
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        app_state
                            .sorted_flights()
                            .get(app_state.ui_state.selected)
                            .copied()
                            .and_then(|f| {
                                Some(command_tx.send(AppCommand::FetchFlightInfo {
                                    hex_id: f.flight.asdb_hex_id.clone(),
                                }))
                            });
                        app_state.ui_state.info_panel_visibility = InfoPanelVisibility::Loading
                    }
                    _ => {}
                }
            }
        }
    }

    terminal.show_cursor()?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn get_flight_info(time_stamped_flight: &TimeStampedFlight) -> Option<()> {
    panic!()
}

fn list_table_widget(flights: Vec<&TimeStampedFlight>) -> Table<'_> {
    let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let format = time::format_description::parse(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
         sign:mandatory]:[offset_minute]:[offset_second]",
    )
    .unwrap_or_default();

    let rows = flights.iter().map(move |flight| {
        Row::new(vec![
            flight.flight.asdb_hex_id.clone(),
            flight.flight.callsign.clone().unwrap_or_default(),
            flight.num_messages.to_string(),
            flight
                .last_seen
                .clone()
                .to_offset(offset)
                .format(&format)
                .unwrap_or_default(),
        ])
    });

    Table::new(
        rows,
        [
            Constraint::Min(6),
            Constraint::Min(8),
            Constraint::Length(6),
            Constraint::Min(30),
        ],
    )
    .header(
        Row::new(vec!["Hex ID", "Callsign", "Msgs", "Last Seen"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .row_highlight_style(Style::default().bg(Color::Blue))
    .block(Block::default().title("Flights").borders(Borders::ALL))
}

fn table_widget(flights: Vec<&TimeStampedFlight>) -> Table<'_> {
    let lenghts = constraints_len_calculator(&flights);
    let constraints = [
        Constraint::Min(lenghts.0),
        Constraint::Min(lenghts.1),
        Constraint::Min(lenghts.2),
        Constraint::Min(lenghts.3),
        Constraint::Min(lenghts.4),
        Constraint::Min(lenghts.5),
        Constraint::Min(lenghts.6),
        Constraint::Min(lenghts.7),
        Constraint::Min(lenghts.8),
        Constraint::Min(lenghts.9),
        Constraint::Min(lenghts.10),
        Constraint::Min(lenghts.11),
    ];

    Table::new(flights, constraints)
        .header(Row::new(vec![
            "Hex ID", "Callsign", "Alt", "Speed", "Track", "Lat", "Lon", "VRate", "Squawk", "E",
            "G", "Msgs",
        ]))
        .block(Block::default().title("Flight").borders(Borders::ALL))
}

fn constraints_len_calculator(
    time_stamped_flights: &Vec<&TimeStampedFlight>,
) -> (u16, u16, u16, u16, u16, u16, u16, u16, u16, u16, u16, u16) {
    // let flights = time_stamped_flights
    //     .into_iter()
    //     .map(TimeStampedFlight::flight)

    // let call_sign_lenght = flights.map(Flight::callsign)
    //     .map(Option::unwrap_or_default)
    //     .map(UnicodeWidthStr::width)
    //     .max()
    //     .unwrap_or_default();

    (6, 6, 5, 5, 4, 9, 9, 5, 4, 1, 1, 5)
}

impl Into<Row<'_>> for &TimeStampedFlight {
    fn into(self) -> Row<'static> {
        let TimeStampedFlight {
            flight,
            num_messages,
            ..
        } = self;
        Row::new(vec![
            flight.asdb_hex_id.clone(),
            flight.callsign.clone().unwrap_or_default(),
            flight.altitute.clone().unwrap_or_default(),
            flight.ground_speed.clone().unwrap_or_default(),
            flight.track.clone().unwrap_or_default(),
            flight.latitude.clone().unwrap_or_default(),
            flight.longitude.clone().unwrap_or_default(),
            flight.vertical_rate.clone().unwrap_or_default(),
            flight.squak.clone().unwrap_or_default(),
            if flight.emergency {
                "Y".to_string()
            } else {
                String::new()
            },
            if flight.is_on_ground {
                "Y".to_string()
            } else {
                String::new()
            },
            num_messages.to_string(),
        ])
    }
}
