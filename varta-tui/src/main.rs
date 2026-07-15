use clap::Parser;
use crossterm::event::{Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use ratatui::prelude::*;
use ratatui::widgets::{Block, Row, Table};
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

fn draw_frame(f: &mut Frame, varta: &varta_easyblade::Varta) {
    let area = f.area();

    let block = Block::bordered().title(" VARTA EasyBlade Monitor ");
    f.render_widget(&block, area);

    let inner = block.inner(area);

    let header = Row::new(["Serial", "Voltage", "Current", "Last Seen"])
        .style(Style::new().add_modifier(Modifier::BOLD));

    let mut rows = Vec::new();
    for eb in varta.easyblades.iter().filter_map(|e| e.as_ref()) {
        let voltage = eb
            .voltage
            .map_or("----".to_string(), |v| format!("{v:.2} V"));
        let current = eb
            .current
            .map_or("----".to_string(), |c| format!("{c:.2} A"));
        let last_seen = format_last_seen(eb.last_seen);
        rows.push(Row::new([
            format!("{}", eb.serial_number),
            voltage,
            current,
            last_seen,
        ]));
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

    f.render_widget(table, inner);
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

    loop {
        terminal.draw(|f| draw_frame(f, &varta))?;

        tokio::select! {
            result = varta.process_socketcan_msg() => {
                if let Err(e) = result {
                    eprintln!("Error processing CAN message: {e}");
                }
            }
            event = rx.recv() => {
                if let Some(Event::Key(key)) = event
                    && key.kind == KeyEventKind::Press
                    && key.code == crossterm::event::KeyCode::Char('q')
                {
                    break;
                }
            }
        }
    }

    handle.abort();
    cleanup_terminal();
    Ok(())
}
