use clap::Parser;
use crossterm::event::{Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use ratatui::prelude::*;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Row, Table, Wrap};
use std::collections::HashMap;
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

fn draw_frame(
    f: &mut Frame,
    varta: &varta_easyblade::Varta,
    selected: usize,
    error_history: &HashMap<usize, Vec<varta_easyblade::DeviceError>>,
) {
    let area = f.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Min(5), Constraint::Min(3)])
        .split(area);

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(layout[2]);

    let top_block = Block::bordered().title(" Master Info ");
    let top_inner = top_block.inner(layout[0]);
    let master = &varta.master;
    if master.last_seen.is_some() {
        let info = format!(
            "V: {:.2}  I: {:.2}  SOC: {:.4}%\n\
             Tfet: {:.1}°C  Tcell: {:.1}°C\n\
             Req: {:.2}V / {:.2}A  Status: {}",
            master.voltage.unwrap_or(0.0),
            master.current.unwrap_or(0.0),
            master.soc.map_or(String::from("-"), |v| v.to_string()),
            master.max_battery_fet_temp.unwrap_or(0.0),
            master.max_battery_cell_temp.unwrap_or(0.0),
            master.charge_voltage_request.unwrap_or(0.0),
            master.charge_current_request.unwrap_or(0.0),
            master
                .battery_status
                .map_or(String::from("?"), |v| v.to_string()),
        );
        let text = Paragraph::new(info).wrap(Wrap { trim: true });
        f.render_widget(text, top_inner);
    } else {
        let text = Paragraph::new("No master data").wrap(Wrap { trim: true });
        f.render_widget(text, top_inner);
    }

    let middle_block = Block::bordered().title(" EasyBlade Modules ");
    f.render_widget(&middle_block, layout[1]);
    let middle_inner = middle_block.inner(layout[1]);

    let header = Row::new(["Serial", "Voltage", "Current", "SOC", "SOH", "Last Seen"])
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
        let soh = eb.soh.map_or("----".to_string(), |v| format!("{:.1}%", v));
        let last_seen = format_last_seen(eb.last_seen);
        let row =
            Row::new([format!("{}", eb.serial_number), voltage, current, soc, soh, last_seen]);
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
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(34),
        ],
    )
    .header(header)
    .column_spacing(1);

    f.render_widget(table, middle_inner);

    let sdo_block = Block::bordered().title(" SDO Info ");
    let sdo_inner = sdo_block.inner(bottom_layout[0]);
    f.render_widget(&sdo_block, bottom_layout[0]);
    if let Some(eb) = varta.get_easyblade_by_index(selected) {
        let info = format!(
            "Node ID:          {}\n\
             Serial Number:    {}\n\
             Software Version: {}\n\
             Hardware Version: {}\n\
             Voltage:          {}\n\
             Current:          {}\n\
             SOC:              {}\n\
             SOH:              {}\n\
             Last Seen:        {}",
            eb.node_id,
            eb.serial_number,
            eb.software_version.as_deref().unwrap_or("N/A"),
            eb.hardware_version.as_deref().unwrap_or("N/A"),
            eb.voltage
                .map_or("N/A".to_string(), |v| format!("{:.2} V", v)),
            eb.current
                .map_or("N/A".to_string(), |c| format!("{:.2} A", c)),
            eb.soc.map_or("N/A".to_string(), |v| format!("{:.1}%", v)),
            eb.soh.map_or("N/A".to_string(), |v| format!("{:.1}%", v)),
            format_last_seen(eb.last_seen),
        );
        let text = Paragraph::new(info).wrap(Wrap { trim: true });
        f.render_widget(text, sdo_inner);
    } else {
        let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
        f.render_widget(text, sdo_inner);
    }

    let error_block = Block::bordered().title(" Error History ");
    let error_inner = error_block.inner(bottom_layout[1]);
    f.render_widget(&error_block, bottom_layout[1]);
    if let Some(errors) = error_history.get(&selected) {
        let error_text: Text = errors
            .iter()
            .enumerate()
            .map(|(i, e)| format!("[{:#03}] {:?}\n", i + 1, e))
            .collect();
        let text = Paragraph::new(error_text).wrap(Wrap { trim: true });
        f.render_widget(text, error_inner);
    } else if varta.get_easyblade_by_index(selected).is_some() {
        let text = Paragraph::new("No error history").wrap(Wrap { trim: true });
        f.render_widget(text, error_inner);
    } else {
        let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
        f.render_widget(text, error_inner);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut varta = varta_easyblade::Varta::new(&args.can_interface).await?;

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
    let mut prev_selected = usize::MAX;
    let mut error_history = HashMap::<usize, Vec<varta_easyblade::DeviceError>>::new();
    let mut expire_timer = Box::pin(tokio::time::sleep(varta.next_expiry_delay()));

    loop {
        let count = varta.easyblade_count();
        if selected >= count {
            selected = count.saturating_sub(1);
        }

        terminal.draw(|f| draw_frame(f, &varta, selected, &error_history))?;

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
            event = rx.recv() => {
                if let Some(Event::Key(key)) = event
                    && key.kind == KeyEventKind::Press
                {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') => break,
                        crossterm::event::KeyCode::Up if count > 0 => {
                            selected = selected.saturating_sub(1);
                        }
                        crossterm::event::KeyCode::Down if selected + 1 < count => {
                            selected += 1;
                        }
                        _ => {}
                    }
                }

                if selected != prev_selected {
                    prev_selected = selected;
                    if let Some(eb) = varta.get_easyblade_by_index(selected)
                        && let Ok(errors) = varta.read_device_error_history(eb).await
                    {
                        error_history.insert(selected, errors);
                    }
                }
            }
        }
    }

    handle.abort();
    cleanup_terminal();
    Ok(())
}
