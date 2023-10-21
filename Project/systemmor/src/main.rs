use sysinfo::{System, SystemExt, CpuExt, ProcessExt, NetworkExt, ComponentExt, DiskExt};
use std::{io, thread::{self}, time::Duration};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Tabs, Paragraph, *},
    layout::{Layout, Constraint, Direction},
    Terminal,
    text::{Span, Line},
    style::{Style, Color, Modifier},
    symbols::*,
    prelude::*,
};
use std::time::Instant;
use std::sync::mpsc;
use crossterm::{
    event::{self, KeyCode, DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode},
    execute,
};

#[derive(Default)]
struct App {
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
}
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
                    let block = Block::default()
                        .title("Home")
                        .borders(Borders::ALL);
                    rect.render_widget(block, chunks[1]);

                    let home_text = vec![
                        Line::from(vec![
                            Span::raw("Welcome to System Monitor!"),
                        ]),
                        Line::from(vec![
                            Span::raw("Press "),
                            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(" to exit."),
                        ]),
                        Line::from(vec![
                            Span::raw("Press "),
                            Span::styled("c, m, n, p, d, t", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(" and"),
                            Span::styled(" b", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(" to choose what to display."),
                        ]),
                    ];

                    let home_paragraph = Paragraph::new(home_text)
                        .block(Block::default().borders(Borders::ALL).title("Home"))
                        .style(Style::default().fg(Color::White))
                        .alignment(ratatui::layout::Alignment::Center);
                    rect.render_widget(home_paragraph, chunks[1]);
                }
                MenuItem::CPU => {
                    sys.refresh_all();
                    let mut cpu_usage = vec![];
                    let mut cpu_all: f32 = 0.0;

                    for (i, cpu) in sys.cpus().iter().enumerate() {
                        let cpu_stat = format!("CPU {} {:.2}%", i, cpu.cpu_usage());
                        cpu_all += cpu.cpu_usage();
                        cpu_usage.push(Line::from(vec![
                            Span::raw(cpu_stat),
                        ]));
                    }

                    let cpu_avg = format!("Average CPU Usage: {:.2}%", cpu_all / sys.cpus().len() as f32);
                    cpu_usage.insert(0, Line::from(vec![
                        Span::raw(cpu_avg),
                    ]));
                    let chunk_cpu = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(30),
                                Constraint::Percentage(70),
                            ].as_ref()
                        )
                        .split(chunks[1]);

                    let cpu_paragraph = Paragraph::new(cpu_usage)
                        .block(Block::default().borders(Borders::ALL).title("CPU Usage"))
                        .style(Style::default().fg(Color::White))
                        .alignment(ratatui::layout::Alignment::Left);

                    let chunk2_cpu = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Percentage(50),
                                Constraint::Percentage(50),
                            ].as_ref()
                        )
                        .split(chunk_cpu[1]);

                    let mut cpu_chart_data: Vec<(f64, f64)> = vec![];
                    let avg_cpu_chart = cpu_all / sys.cpus().len() as f32;
                    for (i, cpu) in sys.cpus().iter().enumerate() {
                        let cpu_stat = i as f64; // X-axis value
                        let cpu_usage = cpu.cpu_usage() as f64; // Y-axis value
                        cpu_chart_data.push((cpu_stat, cpu_usage));
                    }

                    let cpu_avg: Vec<(f64, f64)> = vec![(0.0, cpu_all as f64 / sys.cpus().len() as f64), (sys.cpus().len() as f64 , avg_cpu_chart as f64)];
    
                    let datasets = vec![
                        // Dataset::default()
                        //     .name("All CPU Usage")
                        //     .marker(symbols::Marker::Braille)
                        //     .graph_type(GraphType::Line)
                        //     .style(Style::default().fg(Color::Magenta))
                        //     .data(&cpu_chart_data),
                        Dataset::default()
                            .name("Average CPU Usage")
                            .marker(symbols::Marker::Braille)
                            .graph_type(GraphType::Line)
                            .style(Style::default().fg(Color::Yellow))
                            .data(&cpu_avg)
                ];

                    let chart = Chart::new(datasets)
                        .block(Block::default().title("CPU Chart").borders(Borders::ALL))
                        .x_axis(Axis::default()
                        .title(Span::styled("X Axis", Style::default().fg(Color::Red)))
                        .style(Style::default().fg(Color::White))
                        .bounds([0.0, 20.0])
                        .labels(["0.0", "10.0", "15.0"].iter().cloned().map(Span::from).collect()))
                        .y_axis(Axis::default()
                        .title(Span::styled("Y Axis", Style::default().fg(Color::Red)))
                        .style(Style::default().fg(Color::White))
                        .bounds([0.0, 30.0])
                        .labels(["0.0", "10.0", "25.0"].iter().cloned().map(Span::from).collect()));

                    let mut cpu_bar_data: Vec<(String, u64)> = vec![];
                    for (i, cpu) in sys.cpus().iter().enumerate() {
                        let cpu_stat = format!("C{}", i);
                        cpu_bar_data.push((cpu_stat, cpu.cpu_usage() as u64));
                    }

                    let cpu_bar_data_map: Vec<(&str, u64)> = cpu_bar_data.iter().map(|(s, u)| (s.as_str(), *u)).collect();

                    let barchart = BarChart::default()
                    .block(Block::default().title("CPU Bar Graph").borders(Borders::ALL))
                    .data(&cpu_bar_data_map)
                    .bar_width(3)
                    .group_gap(3)
                    .bar_gap(1)
                    .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    .label_style(Style::default().fg(Color::White))
                    .bar_style(Style::default().fg(Color::LightBlue))
                    .direction(Direction::Vertical);

                    rect.render_widget(cpu_paragraph, chunk_cpu[0]);
                    rect.render_widget(chart, chunk2_cpu[0]);
                    rect.render_widget(barchart, chunk2_cpu[1]);
                }
                MenuItem::Memory => {
                    sys.refresh_all();
                    let mem_total = sys.total_memory();
                    let mem_used = sys.used_memory();
                    let mem_usage = format!("Memory: {:.2} / {:.2} GB", mem_used as f64 / 1_073_741_824.0 , mem_total as f64 / 1_073_741_824.0); // 1 GB = 1_073_741_824.0 bytes
                    let available = format!("Available memory: {:.2} GB", mem_total as f64 / 1_073_741_824.0 - mem_used as f64 / 1_073_741_824.0);
                    let mem_paragraph = Paragraph::new(vec![
                        Line::from(vec![
                            Span::raw(mem_usage),
                        ]),
                        Line::from(vec![
                            Span::raw(available),
                        ]),
                    ])
                    .block(Block::default().borders(Borders::ALL).title("Memory"))
                    .style(Style::default().fg(Color::White))
                    .alignment(ratatui::layout::Alignment::Left);

                    let chunk_mem = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(20),
                                Constraint::Percentage(80),
                            ].as_ref()
                        )
                        .split(chunks[1]);

                    let mem_gauge = Gauge::default()
                        .block(Block::default().title("Memory Gauge").borders(Borders::ALL))
                        .gauge_style(Style::default().fg(Color::Magenta))
                        .use_unicode(true)
                        .ratio(((mem_used as f64 / 1_073_741_824.0) / (mem_total as f64 / 1_073_741_824.0)).into());
                    
                    rect.render_widget(mem_gauge, chunk_mem[1]);
                    rect.render_widget(mem_paragraph, chunk_mem[0]);
                    
                }
                MenuItem::Network => {
                    sys.refresh_all();
                    let mut network_usage = vec![];
                    for (i, network) in sys.networks(){
                        let transmitted_kb = (network.transmitted() as f64 * 8.0) / 1000.0;
                        let received_kb = (network.received() as f64 * 8.0) / 1000.0;
                        let network_stat1 = format!("Network [{}]", i);
                        network_usage.push(Line::from(vec![
                            Span::raw(network_stat1),
                        ]));
                        let network_stat2 = format!("Received: {:.2} KB Transmitted: {:.2} KB", received_kb, transmitted_kb);
                        network_usage.push(Line::from(vec![
                            Span::raw(network_stat2),
                            ]));
                        let space = format!(" ");
                        network_usage.push(Line::from(vec![
                            Span::raw(space),
                        ]));
                    }
                    
                    let network_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(30),
                            Constraint::Percentage(70),
                            ].as_ref()
                        )
                        .split(chunks[1]);
                    
                    let mut network_bar_data: Vec<(String, u64)> = vec![];
                    for (i, network) in sys.networks(){
                        let network_stat = format!("{}", i);
                        network_bar_data.push((network_stat, network.received() as u64));
                    }
                    
                    let network_bar_data_map: Vec<(&str, u64)> = network_bar_data.iter().map(|(s, u)| (s.as_str(), *u)).collect();
                    
                    let network_barchart = BarChart::default()
                    .block(Block::default().title("Network Bar Graph").borders(Borders::ALL))
                    .data(&network_bar_data_map)
                    .bar_width(3)
                    .group_gap(3)
                    .bar_gap(1)
                    .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    .label_style(Style::default().fg(Color::White))
                    .bar_style(Style::default().fg(Color::LightBlue))
                    .direction(Direction::Vertical);
                
                    app.vertical_scroll_state = app.vertical_scroll_state.content_length(network_usage.len() as u16);
                    let network_paragraph = Paragraph::new(network_usage.clone())
                    .block(Block::default().borders(Borders::ALL).title("Network").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .alignment(ratatui::layout::Alignment::Left)
                    .scroll((app.vertical_scroll as u16 , 0));
                    
                    rect.render_widget(network_paragraph, network_chunk[0]);
                    rect.render_stateful_widget(Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
                    network_chunk[0].inner(&Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                     &mut app.vertical_scroll_state
                    );

                    rect.render_widget(network_barchart, network_chunk[1]);
                }
                MenuItem::Process => {
                    sys.refresh_all();
                    let mut process_usage = vec![];
                    for (pid, process) in sys.processes() {
                        let formatted_pid = format!("Process ID: {:7}", pid);
                        let process_stat = format!("[{:17}] {:40} {:.2} MB", formatted_pid, process.name(), process.memory() as f64 / 1_048_576.0);
                        process_usage.push(Line::from(vec![
                            Span::raw(process_stat),
                        ]));
                    }
                    app.vertical_scroll_state = app.vertical_scroll_state.content_length(process_usage.len() as u16);

                    let process_paragraph = Paragraph::new(process_usage.clone())
                        .block(Block::default().borders(Borders::ALL).title("Process"))
                        .style(Style::default().fg(Color::White))
                        .alignment(ratatui::layout::Alignment::Left)
                        .scroll((app.vertical_scroll as u16 , 0));

                    rect.render_widget(process_paragraph, chunks[1]);
                    rect.render_stateful_widget(Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
                    chunks[1].inner(&Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                     &mut app.vertical_scroll_state
                    );
                }
            
                MenuItem::Disk => {
                    sys.refresh_all();
                    let disk = sys.disks();
                    let disk_stat = format!("Name: {:?}", disk[0].name());
                    let disk_type = format!("Type: {:?}", disk[0].kind());
                    let disk_total = format!("Total space: {:.2} GB", disk[0].total_space() as f64 / 1_073_741_824.0);
                    let disk_used = format!("Used space: {:.2} GB", disk[0].total_space()as f64 / 1_073_741_824.0 - disk[0].available_space() as f64 / 1_073_741_824.0);
                    let disk_free = format!("Free space: {:.2} GB", disk[0].available_space() as f64 / 1_073_741_824.0);
                    
                    let disk_paragraph = Paragraph::new(vec![
                        Line::from(vec![
                            Span::raw(disk_stat)
                        ]),
                        Line::from(vec![
                            Span::raw(disk_type)
                        ]),
                        Line::from(vec![
                            Span::raw(disk_total)
                        ]),
                        Line::from(vec![
                            Span::raw(disk_used)
                        ]),
                        Line::from(vec![
                            Span::raw(disk_free)
                        ]),
                    ])
                    .block(Block::default().borders(Borders::ALL).title("Disk"))
                    .style(Style::default().fg(Color::White))
                    .alignment(ratatui::layout::Alignment::Left);
        
                    rect.render_widget(disk_paragraph, chunks[1]);
                }
                MenuItem::Temperature => {
                    sys.refresh_all();
                    let mut temp_usage = vec![];
                    for components in sys.components(){
                        let temp_stat = format!("[{:17}], {:.1}°C", components.label(), components.temperature());
                        temp_usage.push(Line::from(vec![
                            Span::raw(temp_stat)
                        ]))
                    }

                    let temp_chunk = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(30),
                                Constraint::Percentage(70),
                            ].as_ref()
                        )
                        .split(chunks[1]);
                    
                    let mut temp_bar_data: Vec<(String, u64)> = vec![];
                    for components in sys.components(){
                        let temp_stat = format!("{:10}", components.label());
                        temp_bar_data.push((temp_stat, components.temperature() as u64));
                    }

                    let temp_bar_data_map: Vec<(&str, u64)> = temp_bar_data.iter().map(|(s, u)| (s.as_str(), *u)).collect();

                    let temp_barchart = BarChart::default()
                    .block(Block::default().title("Temperature Bar Graph").borders(Borders::ALL))
                    .data(&temp_bar_data_map)
                    .bar_width(3)
                    .group_gap(3)
                    .bar_gap(1)
                    .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    .label_style(Style::default().fg(Color::White))
                    .bar_style(Style::default().fg(Color::LightBlue))
                    .direction(Direction::Vertical);

                    app.vertical_scroll_state = app.vertical_scroll_state.content_length(temp_usage.len() as u16);

                    let temp_paragraph = Paragraph::new(temp_usage)
                    .block(Block::default().borders(Borders::ALL).title("Temperature"))
                    .style(Style::default().fg(Color::White))
                    .alignment(ratatui::layout::Alignment::Left)
                    .scroll((app.vertical_scroll as u16 , 0));
                    
                    rect.render_widget(temp_barchart, temp_chunk[1]);
                    rect.render_widget(temp_paragraph, temp_chunk[0]);
                    rect.render_stateful_widget(Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .symbols(scrollbar::VERTICAL)
                    .begin_symbol(None)
                    .track_symbol(None)
                    .end_symbol(None),
                    temp_chunk[0].inner(&Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                     &mut app.vertical_scroll_state
                    );

                }
                MenuItem::Battery => {
                    sys.refresh_all();
                    let manager = battery::Manager::new().unwrap();
                    let mut battery_usage = vec![];

                    for (idx, battery) in manager.batteries().unwrap().enumerate() {
                        let battery = battery.unwrap();
                        let battery_stat = format!("Battery {}: {:?}, Current Battery {:.2}%", idx + 1, battery.state(), battery.state_of_charge().value * 100.0);
                        battery_usage.push(Line::from(vec![
                            Span::raw(battery_stat)
                        ]));
                    }

                    let chunk_battery = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(30),
                                Constraint::Percentage(70),
                            ].as_ref()
                        )
                        .split(chunks[1]);

                    let battery_paragraph = Paragraph::new(battery_usage)
                        .block(Block::default().borders(Borders::ALL).title("Battery"))
                        .style(Style::default().fg(Color::White))
                        .alignment(ratatui::layout::Alignment::Left);

                    let battery = battery::Manager::new().unwrap();
                    let mut gauge_battery = battery.batteries().unwrap();
                    let used_battery = if let Some(battery) = gauge_battery.next() {
                        battery.expect("Reason").state_of_charge().value * 100.0
                    } else {
                        0.0
                    };

                    let gauge = Gauge::default()
                    .block(Block::default().title("Battery Gauge").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Green))
                    .use_unicode(true)
                    .ratio((used_battery / 100.0).into());

                    rect.render_widget(battery_paragraph, chunk_battery[0]);
                    rect.render_widget(gauge, chunk_battery[1]);
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
                        execute!(io::stdout(), DisableMouseCapture).expect("can disable mouse capture");
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
