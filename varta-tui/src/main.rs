use clap::Parser;
use crossterm::event::{Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use ratatui::prelude::*;
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

fn draw_frame(f: &mut Frame, varta: &varta_easyblade::Varta, selected: usize) {
    let area = f.area();

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Min(5), Constraint::Min(3)])
        .split(area);

    let top_block = Block::bordered().title(" Master Info ");
    let top_inner = top_block.inner(layout[0]);
    let master = &varta.master;
    if master.last_seen.is_some() {
        let info = format!(
            "V: {:.2}  I: {:.2}  SOC: {}%\n\
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

    let header = Row::new(["Serial", "Voltage", "Current", "Last Seen"])
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
        let last_seen = format_last_seen(eb.last_seen);
        let row = Row::new([format!("{}", eb.serial_number), voltage, current, last_seen]);
        if idx == selected {
            rows.push(row.style(Style::new().add_modifier(Modifier::REVERSED)));
        } else {
            rows.push(row);
        }
    }

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(15),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(49),
        ],
    )
    .header(header)
    .column_spacing(1);

    f.render_widget(table, middle_inner);

    let bottom_block = Block::bordered().title(" SDO Info ");
    let bottom_inner = bottom_block.inner(layout[2]);
    if let Some(eb) = varta.get_easyblade_by_index(selected) {
        let info = format!(
            "Node ID:          {}\n\
             Serial Number:    {}\n\
             Software Version: {}\n\
             Hardware Version: {}\n\
             Voltage:          {}\n\
             Current:          {}\n\
             Last Seen:        {}\n",
            eb.node_id,
            eb.serial_number,
            eb.software_version.as_deref().unwrap_or("N/A"),
            eb.hardware_version.as_deref().unwrap_or("N/A"),
            eb.voltage
                .map_or("N/A".to_string(), |v| format!("{:.2} V", v)),
            eb.current
                .map_or("N/A".to_string(), |c| format!("{:.2} A", c)),
            format_last_seen(eb.last_seen),
        );
        let text = Paragraph::new(info).wrap(Wrap { trim: true });
        f.render_widget(text, bottom_inner);
    } else {
        let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
        f.render_widget(text, bottom_inner);
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

    loop {
        let count = varta.easyblade_count();
        if selected >= count {
            selected = count.saturating_sub(1);
        }

        terminal.draw(|f| draw_frame(f, &varta, selected))?;

        tokio::select! {
            result = varta.process_socketcan_msg() => {
                if let Err(e) = result {
                    eprintln!("Error processing CAN message: {e}");
                }
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
            }
        }
    }

    handle.abort();
    cleanup_terminal();
    Ok(())
}
