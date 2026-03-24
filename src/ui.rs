use crate::{
    sbs1_ingest::SSB1StreamEvent,
    state::{AppState, InfoPanelState, TimeStampedFlight},
};
use crate::{AppCommand, AppEvent};
use crossterm::{
    event::{self, poll, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Layout},
    prelude::{Constraint, CrosstermBackend, Direction, Frame, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Row, Table, TableState},
    Terminal,
};
use std::{io, time::Duration};
use time::UtcOffset;
use tokio::sync::mpsc::{error, UnboundedReceiver, UnboundedSender};

// use unicode_width::UnicodeWidthStr;
pub fn run_ui(
    mut event_rx: UnboundedReceiver<AppEvent>,
    command_tx: UnboundedSender<AppCommand>,
    app_state: &mut AppState,
) -> io::Result<()> {
    let mut terminal = setup_terminal()?;

    let polling_timeout = Duration::from_millis(100);

    loop {
        drain_events(&mut event_rx, app_state);
        terminal.draw(|frame| draw_ui(frame, app_state))?;

        if should_quit(polling_timeout, &command_tx, app_state)? {
            break;
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    terminal.show_cursor()?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn drain_events(event_rx: &mut UnboundedReceiver<AppEvent>, app_state: &mut AppState) {
    loop {
        match event_rx.try_recv() {
            Ok(event) => apply_app_event(event, app_state),
            Err(error::TryRecvError::Empty | error::TryRecvError::Disconnected) => break,
        }
    }
}

fn apply_app_event(event: AppEvent, app_state: &mut AppState) {
    match event {
        AppEvent::Ingest(event) => match event {
            SSB1StreamEvent::Connected => app_state.connected = true,
            SSB1StreamEvent::ParseError(error) => app_state.last_error = Some(format!("{error:?}")),
            SSB1StreamEvent::ConnectionError(error) => {
                app_state.last_error = Some(format!("{error:?}"))
            }
            SSB1StreamEvent::Disconnected => app_state.connected = false,
            SSB1StreamEvent::Message(message) => app_state.update_from_message(message),
        },
        AppEvent::FlightInfoEvent(result) => match result {
            Ok(flight_info) => {
                app_state.ui_state.info_panel_state = InfoPanelState::Open { model: flight_info }
            }
            Err(error) => {
                app_state.ui_state.info_panel_state = InfoPanelState::Error(format!("{:#}", error))
            }
        },
    }
}

fn draw_ui(frame: &mut Frame, app_state: &mut AppState) {
    let [content_area, help_bar_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .areas(frame.area());

    let [flight_list_area, details_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .areas(content_area);

    let [flight_area, status_area] =
        detail_layout(details_area, &app_state.ui_state.info_panel_state);

    app_state.clamp_selection();
    let sorted_flights = app_state.sorted_flights();
    let selected = selected_index(&sorted_flights, app_state.ui_state.selected);

    render_flight_list(frame, flight_list_area, &sorted_flights, selected);

    frame.render_widget(Clear, details_area);

    let table_flights = selected
        .and_then(|index| sorted_flights.get(index).copied())
        .map(|flight| vec![flight])
        .unwrap_or_default();
    frame.render_widget(table_widget(table_flights), flight_area);
    render_status_panel(frame, status_area, &app_state.ui_state.info_panel_state);
    render_help_bar(frame, help_bar_area);
}

fn detail_layout(area: Rect, info_panel_state: &InfoPanelState) -> [Rect; 2] {
    let [top, bottom] = match info_panel_state {
        InfoPanelState::Loading | InfoPanelState::Open { .. } | InfoPanelState::Error(_) => {
            [Constraint::Percentage(25), Constraint::Percentage(75)]
        }
        InfoPanelState::Closed => [Constraint::Percentage(100), Constraint::Percentage(0)],
    };

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([top, bottom])
        .areas(area)
}

fn selected_index(flights: &[&TimeStampedFlight], selected: usize) -> Option<usize> {
    if flights.is_empty() {
        None
    } else {
        Some(selected)
    }
}

fn render_flight_list(
    frame: &mut Frame,
    area: Rect,
    flights: &[&TimeStampedFlight],
    selected: Option<usize>,
) {
    let mut table_state = TableState::default();
    table_state.select(selected);
    frame.render_stateful_widget(list_table_widget(flights.to_vec()), area, &mut table_state);
}

fn render_status_panel(frame: &mut Frame, area: Rect, info_panel_state: &InfoPanelState) {
    let message = match info_panel_state {
        InfoPanelState::Loading => Some("Loading...".to_string()),
        InfoPanelState::Open { model } => Some(format!("Loaded: {:?}", model)),
        InfoPanelState::Error(error) => Some(format!("Error: {}", error)),
        InfoPanelState::Closed => None,
    };

    if let Some(message) = message {
        let paragraph = Paragraph::new(message)
            .block(Block::bordered().title("Flight Status"))
            .alignment(Alignment::Center);

        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
}

fn render_help_bar(frame: &mut Frame, area: Rect) {
    let help_bar = Paragraph::new(Line::from(
        " Up/Down: Select flight   I: Fetch flight info   Q: Quit ",
    ))
    .style(
        Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .alignment(Alignment::Center);

    frame.render_widget(help_bar, area);
}

fn should_quit(
    polling_timeout: Duration,
    command_tx: &UnboundedSender<AppCommand>,
    app_state: &mut AppState,
) -> io::Result<bool> {
    if !poll(polling_timeout)? {
        return Ok(false);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };

    Ok(handle_key_event(key.code, command_tx, app_state))
}

fn handle_key_event(
    key_code: KeyCode,
    command_tx: &UnboundedSender<AppCommand>,
    app_state: &mut AppState,
) -> bool {
    match key_code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            let _ = command_tx.send(AppCommand::Quit);
            true
        }
        KeyCode::Down => {
            app_state.select_next();
            app_state.ui_state.info_panel_state = InfoPanelState::Closed;
            false
        }
        KeyCode::Up => {
            app_state.select_prev();
            app_state.ui_state.info_panel_state = InfoPanelState::Closed;
            false
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            fetch_selected_flight_info(command_tx, app_state);
            false
        }
        _ => false,
    }
}

fn fetch_selected_flight_info(command_tx: &UnboundedSender<AppCommand>, app_state: &mut AppState) {
    let selected_hex_id = app_state
        .sorted_flights()
        .get(app_state.ui_state.selected)
        .copied()
        .map(|flight| flight.flight.asdb_hex_id.clone());

    if let Some(hex_id) = selected_hex_id {
        app_state.ui_state.info_panel_state = InfoPanelState::Loading;
        let _ = command_tx.send(AppCommand::FetchFlightInfo { hex_id });
    } else {
        app_state.ui_state.info_panel_state = InfoPanelState::Closed;
    }
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
