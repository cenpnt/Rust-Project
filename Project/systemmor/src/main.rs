use sysinfo::*;
use systemmor::{display_cpu, display_memory, display_network, display_process, display_disk, display_temperature, display_battery, display_home};
use std::{io, thread::{self}, time::Duration};
use ratatui::{
    backend::CrosstermBackend,
    widgets::*,
    layout::{Layout, Constraint, Direction},
    Terminal,
    text::{Span, Line},
    style::*
};
use std::time::Instant;
use std::sync::mpsc;
use crossterm::{
    event::{self, KeyCode, DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode},
    execute,
};
use systemmor::App;

enum Event<T> {
    Input(T),
    Tick
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    CPU,
    Memory,
    Network,
    Process,
    Disk,
    Temperature,
    Battery,
    Quit,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::CPU => 1,
            MenuItem::Memory => 2,
            MenuItem::Network => 3,
            MenuItem::Process => 4,
            MenuItem::Disk => 5,
            MenuItem::Temperature => 6,
            MenuItem::Battery => 7,
            MenuItem::Quit => 8,
        }
    }
}
fn main() -> Result<(), io::Error> {
    enable_raw_mode().expect("can run in raw mode");
    
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
            
            if event::poll(timeout).expect("poll works") {
                if let event::Event::Key(key) = event::read().expect("can read event") {
                    tx.send(Event::Input(key)).expect("can send event");
                }
            }
            
            if last_tick.elapsed() >= tick_rate {
                if tx.send(Event::Tick).is_err() {
                    break;
                }
                last_tick = Instant::now();
            }
        }
    });
    
    execute!(io::stdout(), EnableMouseCapture).expect("can enable mouse capture");
    let mut app = App::default();
    let mut sys = System::new_all();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec![
        "Home",
        "CPU",
        "Memory",
        "Network",
        "Process",
        "Disk",
        "Temperature",
        "Battery",
        "Quit",];

    let mut active_menu_item = MenuItem::Home;

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ].as_ref()
                )
                .split(size);
            let menu = menu_titles.iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Line::from(vec![
                        Span::styled(first, Style::default().fg(Color::Yellow).add_modifier(Modifier::UNDERLINED)),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                }).collect();
            
            let tabs = Tabs::new(menu)
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .select(active_menu_item.into())
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            match active_menu_item {
                MenuItem::Home => {
                    display_home(rect, chunks[1])
                }
                MenuItem::CPU => {
                    sys.refresh_all();
                    display_cpu(&sys, rect, chunks[1])
                }
                MenuItem::Memory => {
                    sys.refresh_all();
                    display_memory(&sys, rect, chunks[1])
                }
                MenuItem::Network => {
                    sys.refresh_all();
                    display_network(&sys, rect, chunks[1], &mut app)
                }
                MenuItem::Process => {
                    sys.refresh_all();
                    display_process(&sys, rect, chunks[1], &mut app)
                }
                MenuItem::Disk => {
                    sys.refresh_all();
                    display_disk(&sys, rect, chunks[1])
                }
                MenuItem::Temperature => {
                    sys.refresh_all();
                    display_temperature(&sys, rect, chunks[1], &mut app)
                }
                MenuItem::Battery => {
                    sys.refresh_all();
                    display_battery(rect, chunks[1])
                }
                MenuItem::Quit => {
                    let block = Block::default()
                        .title("Quit")
                        .borders(Borders::ALL);
                    rect.render_widget(block, chunks[1]);
                }
            }

            rect.render_widget(tabs, chunks[0]);
        })?;
                
        match rx.recv() {
            Ok(event) => match event { //check if event is a keypress or tick
                Event::Input(event) => match event.code { // check if keypress is a key if not ignore
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen).expect("can disable mouse capture");
                        break;
                    }
                    KeyCode::Char('h') => {
                        active_menu_item = MenuItem::Home;
                    }
                    KeyCode::Char('c') => {
                        active_menu_item = MenuItem::CPU;
                    }
                    KeyCode::Char('m') => {
                        active_menu_item = MenuItem::Memory;
                    }
                    KeyCode::Char('n') => {
                        active_menu_item = MenuItem::Network;
                    }
                    KeyCode::Char('p') => {
                        active_menu_item = MenuItem::Process;
                    }
                    KeyCode::Char('d') => {
                        active_menu_item = MenuItem::Disk;
                    }
                    KeyCode::Char('t') => {
                        active_menu_item = MenuItem::Temperature;
                    }
                    KeyCode::Char('b') => {
                        active_menu_item = MenuItem::Battery;
                    }
                    KeyCode::Down => {
                        app.vertical_scroll = app.vertical_scroll.saturating_add(1);
                        app.vertical_scroll_state = app
                            .vertical_scroll_state
                            .position(app.vertical_scroll as u16);
                    }
                    KeyCode::Up => {
                        app.vertical_scroll = app.vertical_scroll.saturating_sub(1);
                        app.vertical_scroll_state = app
                            .vertical_scroll_state
                            .position(app.vertical_scroll as u16);
                    }
                    _ => {}
                },
                Event::Tick => {}
            },
            Err(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("RecvError: {:?}", err),
                ));
            }
        }
    }
    Ok(())
}
