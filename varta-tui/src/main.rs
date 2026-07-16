use clap::Parser;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use ratatui::prelude::*;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Row, Table, Wrap};
use std::io::stdout;
use std::time::Duration;
use std::time::SystemTime;
use tokio::sync::mpsc;

#[derive(Debug, Parser)]
struct Args {
    /// The CAN interface to connect to (e.g. 'can0' or 'vcan0').
    #[arg(short, long, default_value_t = String::from("can0"))]
    can_interface: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectedTab {
    ModuleInfo,
    CellVoltages,
    ErrorHistory,
}

impl SelectedTab {
    fn title(&self) -> &'static str {
        match self {
            SelectedTab::ModuleInfo => "Module Info",
            SelectedTab::CellVoltages => "Cell Voltages",
            SelectedTab::ErrorHistory => "Error History",
        }
    }

    fn cycle(&self, right: bool) -> Self {
        let tabs = [SelectedTab::ModuleInfo, SelectedTab::CellVoltages, SelectedTab::ErrorHistory];
        let idx = tabs.iter().position(|t| *t == *self).unwrap();
        let next = if right {
            (idx + 1) % tabs.len()
        } else {
            (idx as i32 - 1 + tabs.len() as i32) as usize % tabs.len()
        };
        tabs[next]
    }
}

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn cleanup_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen);
}

fn format_last_seen(last_seen: SystemTime) -> String {
    chrono::DateTime::<chrono::Local>::from(last_seen)
        .format("%Y-%m-%d %H:%M:%S.%3f")
        .to_string()
}

fn draw_frame(f: &mut Frame, varta: &varta_easyblade::Varta, selected: usize, tab: SelectedTab) {
    let area = f.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(5), Constraint::Min(5)])
        .split(area);

    // Top: master info
    let top_block = Block::bordered().title(" Master ");
    let top_inner = top_block.inner(layout[0]);
    let master = &varta.master;
    if master
        .last_seen
        .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
        .map(|d| d.as_secs() <= 10)
        .unwrap_or(false)
    {
        let info = format!(
            "{:>7.2} V  {:>7.2} A  SOC: {:>6}\n\
             Charge Request: {:>7.2} V, {:>7.2} A  Status: {:>6}\n\
             FET Temp: {:>6.1}  Cell Temp: {:>6.1}\n\
             Capacity: {:>7.2} Ah remaining / {:>7.2} Ah full ({:>7.2} Ah design)",
            master.voltage.unwrap_or(0.0),
            master.current.unwrap_or(0.0),
            master
                .soc
                .map_or("-----".to_string(), |v| format!("{:.1}%", v)),
            master.charge_voltage_request.unwrap_or(0.0),
            master.charge_current_request.unwrap_or(0.0),
            master
                .battery_status
                .map_or("-----".to_string(), |v| format!("{v:#x}")),
            master.max_battery_fet_temp.unwrap_or(0.0),
            master.max_battery_cell_temp.unwrap_or(0.0),
            master.master_remaining_capacity.unwrap_or(0.0),
            master.master_full_charge_capacity.unwrap_or(0.0),
            master.master_design_capacity.unwrap_or(0.0),
        );
        let text = Paragraph::new(info).wrap(Wrap { trim: true });
        f.render_widget(text, top_inner);
    } else {
        let text = Paragraph::new("No master data");
        f.render_widget(text, top_inner);
    }

    // Middle: module table
    let middle_block = Block::bordered().title(" EasyBlade Modules ");
    f.render_widget(&middle_block, layout[1]);
    let middle_inner = middle_block.inner(layout[1]);

    let header = Row::new(["Serial", "Voltage", "Current", "SOC", "Last Seen"])
        .style(Style::new().add_modifier(Modifier::BOLD));

    let mut rows = Vec::new();
    for (idx, eb) in varta
        .easyblades
        .iter()
        .filter_map(|e| e.as_ref())
        .enumerate()
    {
        let voltage = eb
            .voltage
            .map_or("----".to_string(), |v| format!("{v:.2} V"));
        let current = eb
            .current
            .map_or("----".to_string(), |c| format!("{c:.2} A"));
        let soc = eb.soc.map_or("----".to_string(), |v| format!("{:.1}%", v));
        let last_seen = format_last_seen(eb.last_seen);
        let row = Row::new([
            eb.serial_number
                .map_or("----".to_string(), |v| format!("{}", v)),
            voltage,
            current,
            soc,
            last_seen,
        ]);
        if idx == selected {
            rows.push(row.style(Style::new().add_modifier(Modifier::REVERSED)));
        } else {
            rows.push(row);
        }
    }

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(12),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(14),
            Constraint::Percentage(38),
        ],
    )
    .header(header)
    .column_spacing(1);

    f.render_widget(table, middle_inner);

    // Bottom: tabbed pane
    let bottom_block = Block::bordered().title(" Detail ");
    f.render_widget(&bottom_block, layout[2]);
    let bottom_inner = bottom_block.inner(layout[2]);

    let tab_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(bottom_inner);

    let tab_titles: Vec<SelectedTab> =
        vec![SelectedTab::ModuleInfo, SelectedTab::CellVoltages, SelectedTab::ErrorHistory];
    let indicator: String = tab_titles
        .iter()
        .map(|t| {
            if *t == tab {
                format!("[{}]", t.title())
            } else {
                t.title().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" -- ");
    let tabs_text = Paragraph::new(indicator).style(Style::new().fg(Color::White));
    f.render_widget(tabs_text, tab_layout[0]);

    let content_area = tab_layout[1];
    let eb = varta.get_easyblade_by_index(selected);

    match tab {
        SelectedTab::ModuleInfo => {
            if let Some(eb) = eb {
                let node_id = eb.node_id.to_string();
                let serial = eb
                    .serial_number
                    .map_or("N/A".to_string(), |v| format!("{}", v));
                let sw_ver = eb.software_version.as_deref().unwrap_or("N/A");
                let hw_ver = eb.hardware_version.as_deref().unwrap_or("N/A");
                let voltage = eb
                    .voltage
                    .map_or("N/A".to_string(), |v| format!("{:.2} V", v));
                let current = eb
                    .current
                    .map_or("N/A".to_string(), |c| format!("{:.2} A", c));
                let soc = eb.soc.map_or("N/A".to_string(), |v| format!("{:.1}%", v));
                let soh = eb.soh.map_or("N/A".to_string(), |v| format!("{:.1}%", v));
                let last_seen = format_last_seen(eb.last_seen);
                let rows = vec![
                    Row::new(["Node ID", &node_id]),
                    Row::new(["Serial Number", &serial]),
                    Row::new(["Software Version", sw_ver]),
                    Row::new(["Hardware Version", hw_ver]),
                    Row::new(["Voltage", &voltage]),
                    Row::new(["Current", &current]),
                    Row::new(["SOC", &soc]),
                    Row::new(["SOH", &soh]),
                    Row::new(["Last Seen", &last_seen]),
                ];
                let table = Table::new(
                    rows,
                    [Constraint::Percentage(50), Constraint::Percentage(50)],
                );
                f.render_widget(table, content_area);
            } else {
                let text = Paragraph::new("No module selected");
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::CellVoltages => {
            if let Some(eb) = eb {
                if let Some(ref voltages) = eb.cell_voltages {
                    let lines: String = voltages
                        .iter()
                        .enumerate()
                        .map(|(i, v)| format!("Cell {}: {:.3} V\n", i + 1, v))
                        .collect();
                    let text = Paragraph::new(lines).wrap(Wrap { trim: true });
                    f.render_widget(text, content_area);
                } else {
                    let text = Paragraph::new("N/A (not yet read)").wrap(Wrap { trim: true });
                    f.render_widget(text, content_area);
                }
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },
        SelectedTab::ErrorHistory => {
            if let Some(eb) = eb {
                if let Some(ref errors) = eb.device_errors {
                    let error_text: Text = errors
                        .iter()
                        .enumerate()
                        .map(|(i, e)| format!("[{:#03}] {:?}\n", i + 1, e))
                        .collect();
                    let text = Paragraph::new(error_text).wrap(Wrap { trim: true });
                    f.render_widget(text, content_area);
                } else {
                    let text = Paragraph::new("No error history").wrap(Wrap { trim: true });
                    f.render_widget(text, content_area);
                }
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (mut varta, mut sdo_response_rx) = varta_easyblade::Varta::new(&args.can_interface).await?;

    let mut terminal = setup_terminal()?;

    let (tx, mut rx) = mpsc::channel::<Event>(16);
    let handle = tokio::task::spawn_blocking(move || {
        while crossterm::event::poll(Duration::from_secs(10)).is_ok() {
            if let Ok(event) = crossterm::event::read()
                && tx.blocking_send(event).is_err()
            {
                break;
            }
        }
    });

    let mut selected = 0;
    let mut selected_tab = SelectedTab::ModuleInfo;
    let mut expire_timer = Box::pin(tokio::time::sleep(varta.next_expiry_delay()));

    loop {
        let count = varta.easyblade_count();
        if selected >= count {
            selected = count.saturating_sub(1);
        }

        let current_tab = selected_tab;
        terminal.draw(|f| draw_frame(f, &varta, selected, current_tab))?;

        tokio::select! {
            result = varta.process_socketcan_msg() => {
                if let Err(e) = result {
                    eprintln!("Error processing CAN message: {e}");
                }
                expire_timer = Box::pin(tokio::time::sleep(varta.next_expiry_delay()));
            }
            _ = expire_timer.as_mut() => {
                varta.expire_missing_modules();
                expire_timer = Box::pin(tokio::time::sleep(varta.next_expiry_delay()));
            }
            response = sdo_response_rx.recv() => {
                if let Some(resp) = response {
                    match resp {
                        varta_easyblade::SdoResponse::SerialNumber { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.serial_number = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::SoftwareVersion { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.software_version = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::HardwareVersion { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.hardware_version = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::DeviceErrorHistory { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_errors = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellVoltages { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_voltages = Some(v);
                            }
                        },
                        _ => {
                            // Errors are silently dropped for now
                        },
                    }
                }
            }
            event = rx.recv() => {
                if let Some(Event::Key(key)) = event
                    && key.kind == KeyEventKind::Press
                {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up if count > 0 => {
                            selected = selected.saturating_sub(1);
                        }
                        KeyCode::Down if selected + 1 < count => {
                            selected += 1;
                        }
                        KeyCode::Left => {
                            selected_tab = selected_tab.cycle(false);
                        }
                        KeyCode::Right => {
                            selected_tab = selected_tab.cycle(true);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    handle.abort();
    cleanup_terminal();
    Ok(())
}
