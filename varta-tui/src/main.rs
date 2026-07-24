use clap::Parser;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use ratatui::prelude::*;
use ratatui::text::Text;
use ratatui::widgets::{Block, Clear, Paragraph, Row, Table, Wrap};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::io::stdout;
use std::time::Duration;
use std::time::SystemTime;
use tokio::sync::mpsc;

use strum::EnumCount;
use varta_easyblade::SdoRequest;
use varta_easyblade::SdoResponse;

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
    DeviceOperation,
    ErrorCounters,
    CellVoltageLimits,
    BatteryVoltage,
    BatteryCurrent,
    FetTemperature,
    CellTemperature,
    CellBalance,
    Impedance,
    Capacity,
    CycleCount,
    ChargeParameters,
    MasterTemperature,
}

impl SelectedTab {
    fn title(&self) -> &'static str {
        match self {
            SelectedTab::ModuleInfo => "Module Info",
            SelectedTab::CellVoltages => "Cell Voltages",
            SelectedTab::ErrorHistory => "Error History",
            SelectedTab::DeviceOperation => "Device Operation",
            SelectedTab::ErrorCounters => "Error Counters",
            SelectedTab::CellVoltageLimits => "Cell Voltage Limits",
            SelectedTab::BatteryVoltage => "Battery Voltage",
            SelectedTab::BatteryCurrent => "Battery Current",
            SelectedTab::FetTemperature => "FET Temperature",
            SelectedTab::CellTemperature => "Cell Temperature",
            SelectedTab::CellBalance => "Cell Balance",
            SelectedTab::Impedance => "Impedance",
            SelectedTab::Capacity => "Capacity",
            SelectedTab::CycleCount => "Cycle Count",
            SelectedTab::ChargeParameters => "Charge Parameters",
            SelectedTab::MasterTemperature => "Master Temperature",
        }
    }

    fn cycle(&self, right: bool) -> Self {
        let tabs = [
            SelectedTab::ModuleInfo,
            SelectedTab::CellVoltages,
            SelectedTab::ErrorHistory,
            SelectedTab::DeviceOperation,
            SelectedTab::ErrorCounters,
            SelectedTab::CellVoltageLimits,
            SelectedTab::BatteryVoltage,
            SelectedTab::BatteryCurrent,
            SelectedTab::FetTemperature,
            SelectedTab::CellTemperature,
            SelectedTab::CellBalance,
            SelectedTab::Impedance,
            SelectedTab::Capacity,
            SelectedTab::CycleCount,
            SelectedTab::ChargeParameters,
            SelectedTab::MasterTemperature,
        ];
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

fn format_fet_status(fet: Option<(bool, bool, bool)>) -> String {
    match fet {
        Some((c, d, b)) => format!(
            "{}{}{}",
            if c { "C" } else { "-" },
            if d { "D" } else { "-" },
            if b { "B" } else { "-" },
        ),
        None => "---".to_string(),
    }
}

#[derive(Default, Clone)]
enum SaveState {
    #[default]
    Idle,
    Saving {
        file_path: String,
    },
    Saved(String),
}

impl SaveState {
    fn is_active(&self) -> bool {
        !matches!(self, SaveState::Idle)
    }
}

const TOTAL_SDOS: f64 = SdoRequest::COUNT as f64;

fn sdo_completion(eb: &varta_easyblade::VartaEasyblade) -> f64 {
    let mut count = 0.0;
    if eb.serial_number.is_some() {
        count += 1.0;
    }
    if eb.software_version.is_some() {
        count += 1.0;
    }
    if eb.hardware_version.is_some() {
        count += 1.0;
    }
    if eb.device_config_info.is_some() {
        count += 1.0;
    }
    if eb.device_serial_number_info.is_some() {
        count += 1.0;
    }
    if eb.device_date_info.is_some() {
        count += 1.0;
    }
    if eb.device_variant_info.is_some() {
        count += 1.0;
    }
    if eb.device_control_param.is_some() {
        count += 1.0;
    }
    if eb.device_operation_time.is_some() {
        count += 1.0;
    }
    if eb.device_errors.is_some() {
        count += 1.0;
    }
    if eb.device_error_counter.is_some() {
        count += 1.0;
    }
    if eb.cell_voltages.is_some() {
        count += 1.0;
    }
    if eb.cell_voltage_min_max.is_some() {
        count += 1.0;
    }
    if eb.cell_voltage_limit.is_some() {
        count += 1.0;
    }
    if eb.battery_voltage.is_some() {
        count += 1.0;
    }
    if eb.battery_voltage_limit.is_some() {
        count += 1.0;
    }
    if eb.battery_current.is_some() {
        count += 1.0;
    }
    if eb.battery_current_limit.is_some() {
        count += 1.0;
    }
    if eb.fet_temperature.is_some() {
        count += 1.0;
    }
    if eb.fet_temperature_min_max.is_some() {
        count += 1.0;
    }
    if eb.fet_temperature_limit.is_some() {
        count += 1.0;
    }
    if eb.cell_temperature.is_some() {
        count += 1.0;
    }
    if eb.cell_temperature_min_max.is_some() {
        count += 1.0;
    }
    if eb.cell_temperature_limit.is_some() {
        count += 1.0;
    }
    if eb.cell_balance_status.is_some() {
        count += 1.0;
    }
    if eb.cell_balance_limit.is_some() {
        count += 1.0;
    }
    if eb.cell_impedance.is_some() {
        count += 1.0;
    }
    if eb.battery_capacity.is_some() {
        count += 1.0;
    }
    if eb.battery_capacity_param.is_some() {
        count += 1.0;
    }
    if eb.battery_cycle_count.is_some() {
        count += 1.0;
    }
    if eb.battery_charge_voltage.is_some() {
        count += 1.0;
    }
    if eb.battery_charge_current.is_some() {
        count += 1.0;
    }
    if eb.battery_charge_temperature.is_some() {
        count += 1.0;
    }
    if eb.master_battery_temperature.is_some() {
        count += 1.0;
    }
    count
}

fn easyblade_to_json(eb: &varta_easyblade::VartaEasyblade) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("node_id".into(), json!(eb.node_id));
    map.insert("serial_number".into(), json!(eb.serial_number));

    if let Some(ref v) = eb.software_version {
        map.insert("software_version".into(), json!(v));
    }
    if let Some(ref v) = eb.hardware_version {
        map.insert("hardware_version".into(), json!(v));
    }
    if let Some(ref v) = eb.device_config_info {
        map.insert("device_config_info".into(), json!(v));
    }
    if let Some(ref v) = eb.device_serial_number_info {
        map.insert("device_serial_number_info".into(), json!(v));
    }
    if let Some(ref v) = eb.device_date_info {
        map.insert("device_date_info".into(), json!(v));
    }
    if let Some(ref v) = eb.device_variant_info {
        map.insert("device_variant_info".into(), json!(v));
    }
    if let Some(ref v) = eb.device_control_param {
        map.insert("device_control_param".into(), json!(v));
    }
    if let Some(ref v) = eb.device_operation_time {
        map.insert("device_operation_time".into(), json!(v));
    }
    if let Some(ref v) = eb.device_errors {
        map.insert(
            "device_error_history".into(),
            json!(v.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>()),
        );
    }
    if let Some(ref v) = eb.device_error_counter {
        map.insert("device_error_counter".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_voltages {
        map.insert("cell_voltages".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_voltage_min_max {
        map.insert("cell_voltage_min_max".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_voltage_limit {
        map.insert("cell_voltage_limit".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_voltage {
        map.insert("battery_voltage".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_voltage_limit {
        map.insert("battery_voltage_limit".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_current {
        map.insert("battery_current".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_current_limit {
        map.insert("battery_current_limit".into(), json!(v));
    }
    if let Some(ref v) = eb.fet_temperature {
        map.insert("fet_temperature".into(), json!(v));
    }
    if let Some(ref v) = eb.fet_temperature_min_max {
        map.insert("fet_temperature_min_max".into(), json!(v));
    }
    if let Some(ref v) = eb.fet_temperature_limit {
        map.insert("fet_temperature_limit".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_temperature {
        map.insert("cell_temperature".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_temperature_min_max {
        map.insert("cell_temperature_min_max".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_temperature_limit {
        map.insert("cell_temperature_limit".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_balance_status {
        map.insert("cell_balance_status".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_balance_limit {
        map.insert("cell_balance_limit".into(), json!(v));
    }
    if let Some(ref v) = eb.cell_impedance {
        map.insert("cell_impedance".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_capacity {
        map.insert("battery_capacity".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_capacity_param {
        map.insert("battery_capacity_param".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_cycle_count {
        map.insert("battery_cycle_count".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_charge_voltage {
        map.insert("battery_charge_voltage".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_charge_current {
        map.insert("battery_charge_current".into(), json!(v));
    }
    if let Some(ref v) = eb.battery_charge_temperature {
        map.insert("battery_charge_temperature".into(), json!(v));
    }
    if let Some(ref v) = eb.master_battery_temperature {
        map.insert("master_battery_temperature".into(), json!(v));
    }

    serde_json::Value::Object(map)
}

fn try_complete_save(varta: &varta_easyblade::Varta, selected: usize, save_state: &mut SaveState) {
    if let SaveState::Saving { ref file_path } = *save_state {
        let eb = varta.get_easyblade_by_index(selected);
        if let Some(data) = eb {
            if sdo_completion(data) >= TOTAL_SDOS {
                let json_value = easyblade_to_json(data);
                let json_string = serde_json::to_string_pretty(&json_value).unwrap();
                let path = file_path.clone();
                if let Ok(mut file) = fs::File::create(&path) {
                    if let Err(e) = file.write_all(json_string.as_bytes()) {
                        eprintln!("Error writing file: {}", e);
                    }
                } else {
                    eprintln!("Error creating file: {}", path);
                }
                *save_state = SaveState::Saved(path);
            }
        } else {
            *save_state = SaveState::Idle;
        }
    }
}

fn draw_popup(f: &mut Frame, area: Rect, save_state: &SaveState, completion: f64) {
    f.render_widget(Clear, area);

    let popup_block = Block::bordered()
        .title(" Save Module Data ")
        .style(Style::new().bg(Color::Rgb(50, 50, 50)));
    f.render_widget(&popup_block, area);
    let inner = popup_block.inner(area);

    let lines: Vec<Line> = match save_state {
        SaveState::Saving { .. } => {
            let percentage = (completion / TOTAL_SDOS * 100.0).round() as u16;
            let clamped_percentage = percentage.min(100);
            let bar_width: usize = 40;
            let filled = (clamped_percentage as f64 / 100.0 * bar_width as f64).round() as usize;
            let bar_str = format!(
                "{}{}",
                "█".repeat(filled),
                " ".repeat(bar_width.saturating_sub(filled))
            );
            let bar = format!("[{}] {}%", bar_str, clamped_percentage);
            vec![
                Line::from(""),
                Line::from(bar).style(Style::new().fg(Color::Green)),
                Line::from(format!(
                    "Reading SDOs: {}/{}",
                    completion.min(TOTAL_SDOS) as u32,
                    TOTAL_SDOS as u32
                )),
                Line::from("Press 'q' to cancel"),
                Line::from(""),
            ]
        },
        SaveState::Saved(path) => {
            vec![
                Line::from(""),
                Line::from("Data saved successfully!"),
                Line::from(""),
                Line::from(format!("  {}", path)),
                Line::from(""),
                Line::from("Press Enter to dismiss"),
            ]
        },
        SaveState::Idle => vec![],
    };

    let text_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height,
    };
    let text = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(text, text_area);
}

fn draw_frame(
    f: &mut Frame,
    varta: &varta_easyblade::Varta,
    selected: usize,
    tab: SelectedTab,
    save_state: &SaveState,
) {
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

    let header = Row::new(["Serial", "Voltage", "Current", "SOC", "FET", "Last Seen"])
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
        let fet = format_fet_status(eb.fet_status);
        let last_seen = format_last_seen(eb.last_seen);
        let row = Row::new([
            eb.serial_number
                .map_or("----".to_string(), |v| format!("{}", v)),
            voltage,
            current,
            soc,
            fet,
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
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Percentage(32),
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

    let tab_titles: Vec<SelectedTab> = vec![
        SelectedTab::ModuleInfo,
        SelectedTab::CellVoltages,
        SelectedTab::ErrorHistory,
        SelectedTab::DeviceOperation,
        SelectedTab::ErrorCounters,
        SelectedTab::CellVoltageLimits,
        SelectedTab::BatteryVoltage,
        SelectedTab::BatteryCurrent,
        SelectedTab::FetTemperature,
        SelectedTab::CellTemperature,
        SelectedTab::CellBalance,
        SelectedTab::Impedance,
        SelectedTab::Capacity,
        SelectedTab::CycleCount,
        SelectedTab::ChargeParameters,
        SelectedTab::MasterTemperature,
    ];

    // Build tab labels and calculate widths
    let separator = " ";
    let tab_labels: Vec<String> = tab_titles
        .iter()
        .map(|t| {
            if *t == tab {
                format!("[{}]", t.title())
            } else {
                format!(" {} ", t.title())
            }
        })
        .collect();
    let tab_widths: Vec<usize> = tab_labels.iter().map(|l| l.len()).collect();

    let available_width = tab_layout[0].width as usize;

    // Calculate total width including separators
    let total_width: usize =
        tab_widths.iter().sum::<usize>() + separator.len() * (tab_titles.len() - 1);

    // Calculate scroll offset to keep selected tab visible
    let selected_idx = tab_titles.iter().position(|t| *t == tab).unwrap();
    let (scroll_start, scroll_end) = if total_width > available_width {
        // Find the byte offset of the start and end of each tab in the full string
        let mut tab_positions: Vec<(usize, usize)> = Vec::new();
        let mut offset = 0;
        for w in &tab_widths {
            let start = offset;
            let end = offset + w;
            tab_positions.push((start, end));
            offset += w + separator.len();
        }

        let sel_start = tab_positions[selected_idx].0;
        let sel_end = tab_positions[selected_idx].1;

        // Try to center the selected tab, but ensure it's fully visible
        let mut candidate_start = sel_start.saturating_sub(available_width / 2);

        // Make sure the selected tab fits
        if candidate_start + available_width < sel_end {
            candidate_start = sel_end.saturating_sub(available_width);
        }

        // Clamp to valid range
        let max_start = total_width.saturating_sub(available_width);
        let candidate_start = candidate_start.min(max_start);

        (candidate_start, candidate_start + available_width)
    } else {
        (0, total_width)
    };

    // Build the full indicator string
    let full_indicator = tab_labels.join(separator);
    let indicator = if scroll_start > 0 || scroll_end < full_indicator.len() {
        let chars: Vec<char> = full_indicator.chars().collect();
        let mut byte_pos = 0;
        let mut start_char = None;
        let mut end_char = chars.len();
        for (i, c) in chars.iter().enumerate() {
            if byte_pos >= scroll_start && start_char.is_none() {
                start_char = Some(i);
            }
            let next_pos = byte_pos + c.len_utf8();
            if next_pos > scroll_end {
                end_char = i;
                break;
            }
            byte_pos = next_pos;
        }
        let start = start_char.unwrap_or(0);
        chars[start..end_char].iter().collect()
    } else {
        full_indicator
    };

    let tabs_text = Paragraph::new(indicator).style(Style::new().fg(Color::White));
    f.render_widget(tabs_text, tab_layout[0]);

    let content_area = tab_layout[1];
    let eb = varta.get_easyblade_by_index(selected);

    match tab {
        SelectedTab::ModuleInfo => {
            if let Some(eb) = eb {
                let info = format!(
                    "Node ID:              {}\n\
                     Serial Number:        {}\n\
                     Software Version:     {}\n\
                     Hardware Version:     {}\n\
                     Voltage:              {}\n\
                     Current:              {}\n\
                     SOC:                  {}\n\
                     SOH:                  {}\n\
                     FET Status:           {}\n\
                     Last Seen:            {}\n",
                    eb.node_id,
                    eb.serial_number
                        .map_or("N/A".to_string(), |v| format!("{}", v)),
                    eb.software_version.as_deref().unwrap_or("N/A"),
                    eb.hardware_version.as_deref().unwrap_or("N/A"),
                    eb.voltage
                        .map_or("N/A".to_string(), |v| format!("{:.2} V", v)),
                    eb.current
                        .map_or("N/A".to_string(), |c| format!("{:.2} A", c)),
                    eb.soc.map_or("N/A".to_string(), |v| format!("{:.1}%", v)),
                    eb.soh.map_or("N/A".to_string(), |v| format!("{:.1}%", v)),
                    format_fet_status(eb.fet_status),
                    format_last_seen(eb.last_seen),
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
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
                        .map(|(i, v)| format!("Cell {:>2}: {:.3} V\n", i + 1, v))
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

        SelectedTab::DeviceOperation => {
            if let Some(eb) = eb {
                let info = format!(
                    "Operation Minutes (by temp range):\n\
                     <0°C:       {:>3} min\n\
                     0-40°C:     {:>3} min\n\
                     40-60°C:    {:>3} min\n\
                     60-80°C:    {:>3} min\n\
                     >80°C:      {:>3} min\n\
                     Operation Hours (by temp range):\n\
                     <0°C:       {:>10} h\n\
                     0-40°C:     {:>10} h\n\
                     40-60°C:    {:>10} h\n\
                     60-80°C:    {:>10} h\n\
                     >80°C:      {:>10} h\n",
                    match eb.device_operation_time {
                        Some(ref v) => v.minutes_below_zero,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.minutes_zero_to_40,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.minutes_40_to_60,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.minutes_60_to_80,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.minutes_above_80,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.hours_below_zero,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.hours_zero_to_40,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.hours_40_to_60,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.hours_60_to_80,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.hours_above_80,
                        _ => 0u32,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::ErrorCounters => {
            if let Some(eb) = eb {
                if let Some(ref counters) = eb.device_error_counter {
                    let lines = format!(
                        "over_temp_laden_zellen:                          {:>6}\n\
                         under_temp_laden_zellen:                         {:>6}\n\
                         over_temp_laden_fet:                             {:>6}\n\
                         over_temp_entladen_zellen:                       {:>6}\n\
                         under_temp_entladen_zellen:                      {:>6}\n\
                         over_temp_entladen_fet:                          {:>6}\n\
                         over_temp_clamp:                                 {:>6}\n\
                         over_voltage:                                    {:>6}\n\
                         under_voltage:                                   {:>6}\n\
                         deep_low_voltage:                                {:>6}\n\
                         cell_disbalance:                                 {:>6}\n\
                         akku_pack_spn_min_error:                         {:>6}\n\
                         akku_pack_spn_max_alarm:                         {:>6}\n\
                         akku_pack_fused_spn_diff_error:                  {:>6}\n\
                         akku_pwr_spn_diff_error:                         {:>6}\n\
                         akku_pwr_spn_min_error:                          {:>6}\n\
                         akku_pwr_spn_max_error:                          {:>6}\n\
                         akku_netz_spn_min_error:                         {:>6}\n\
                         akku_netz_spn_max_alarm:                         {:>6}\n\
                         akku_rekuperation_spn_max_alarm:                 {:>6}\n\
                         i_charge_sc:                                     {:>6}\n\
                         i_charge_occ_1:                                  {:>6}\n\
                         i_charge_occ_2:                                  {:>6}\n\
                         i_charge_occ_3:                                  {:>6}\n\
                         i_discharge_sc:                                  {:>6}\n\
                         i_discharge_ocd_1:                               {:>6}\n\
                         i_discharge_ocd_2:                               {:>6}\n\
                         i_discharge_ocd_3:                               {:>6}\n\
                         i_akku_diff_error:                               {:>6}\n\
                         scnd_spn_min_error:                              {:>6}\n\
                         scnd_spn_max_error:                              {:>6}\n\
                         scnd_uc_fet_enable:                              {:>6}\n\
                         scnd_current_sense_ein:                          {:>6}\n\
                         scnd_voltage_sense_ein:                          {:>6}\n\
                         scnd_temp_cell_sense_ein:                        {:>6}\n\
                         scnd_temp_fet_sense_ein:                         {:>6}\n\
                         scnd_pyro_fuse_eject_sense_ein:                  {:>6}\n\
                         scnd_i_discharge_fet_error:                      {:>6}\n\
                         scnd_i_charge_fet_error:                         {:>6}\n\
                         scnd_voltage_error:                              {:>6}\n\
                         adc_spn_min_scale:                               {:>6}\n\
                         adc_spn_max_scale:                               {:>6}\n\
                         adc_temp_zellen_min_scale:                       {:>6}\n\
                         adc_temp_zellen_max_scale:                       {:>6}\n\
                         adc_temp_fet_min_scale:                          {:>6}\n\
                         adc_temp_fet_max_scale:                          {:>6}\n\
                         adc_temp_clamp_min_scale:                        {:>6}\n\
                         adc_temp_clamp_max_scale:                        {:>6}\n\
                         adc_i_charge_min_scale:                          {:>6}\n\
                         adc_i_charge_max_scale:                          {:>6}\n\
                         adc_i_discharge_min_scale:                       {:>6}\n\
                         adc_i_discharge_max_scale:                       {:>6}\n\
                         i_discharge_fet_error:                           {:>6}\n\
                         i_charge_fet_error:                              {:>6}\n\
                         i_discharge_charge_fet_error:                    {:>6}\n\
                         temp_discharge_error_lock:                       {:>6}\n\
                         temp_charge_error_lock:                          {:>6}\n\
                         over_charge_current_alarm_recuperation:          {:>6}\n\
                         over_charge_cell_voltage_alarm_recuperation:     {:>6}\n\
                         v24_spn_min_error:                               {:>6}\n\
                         v24_spn_max_error:                               {:>6}\n\
                         can_network_not_conf_node_id:                    {:>6}\n\
                         can_network_double_node_id:                      {:>6}\n\
                         parameter_configuration_error:                   {:>6}\n",
                        counters.over_temp_laden_zellen,
                        counters.under_temp_laden_zellen,
                        counters.over_temp_laden_fet,
                        counters.over_temp_entladen_zellen,
                        counters.under_temp_entladen_zellen,
                        counters.over_temp_entladen_fet,
                        counters.over_temp_clamp,
                        counters.over_voltage,
                        counters.under_voltage,
                        counters.deep_low_voltage,
                        counters.cell_disbalance,
                        counters.akku_pack_spn_min_error,
                        counters.akku_pack_spn_max_alarm,
                        counters.akku_pack_fused_spn_diff_error,
                        counters.akku_pwr_spn_diff_error,
                        counters.akku_pwr_spn_min_error,
                        counters.akku_pwr_spn_max_error,
                        counters.akku_netz_spn_min_error,
                        counters.akku_netz_spn_max_alarm,
                        counters.akku_rekuperation_spn_max_alarm,
                        counters.i_charge_sc,
                        counters.i_charge_occ_1,
                        counters.i_charge_occ_2,
                        counters.i_charge_occ_3,
                        counters.i_discharge_sc,
                        counters.i_discharge_ocd_1,
                        counters.i_discharge_ocd_2,
                        counters.i_discharge_ocd_3,
                        counters.i_akku_diff_error,
                        counters.scnd_spn_min_error,
                        counters.scnd_spn_max_error,
                        counters.scnd_uc_fet_enable,
                        counters.scnd_current_sense_ein,
                        counters.scnd_voltage_sense_ein,
                        counters.scnd_temp_cell_sense_ein,
                        counters.scnd_temp_fet_sense_ein,
                        counters.scnd_pyro_fuse_eject_sense_ein,
                        counters.scnd_i_discharge_fet_error,
                        counters.scnd_i_charge_fet_error,
                        counters.scnd_voltage_error,
                        counters.adc_spn_min_scale,
                        counters.adc_spn_max_scale,
                        counters.adc_temp_zellen_min_scale,
                        counters.adc_temp_zellen_max_scale,
                        counters.adc_temp_fet_min_scale,
                        counters.adc_temp_fet_max_scale,
                        counters.adc_temp_clamp_min_scale,
                        counters.adc_temp_clamp_max_scale,
                        counters.adc_i_charge_min_scale,
                        counters.adc_i_charge_max_scale,
                        counters.adc_i_discharge_min_scale,
                        counters.adc_i_discharge_max_scale,
                        counters.i_discharge_fet_error,
                        counters.i_charge_fet_error,
                        counters.i_discharge_charge_fet_error,
                        counters.temp_discharge_error_lock,
                        counters.temp_charge_error_lock,
                        counters.over_charge_current_alarm_recuperation,
                        counters.over_charge_cell_voltage_alarm_recuperation,
                        counters.v24_spn_min_error,
                        counters.v24_spn_max_error,
                        counters.can_network_not_conf_node_id,
                        counters.can_network_double_node_id,
                        counters.parameter_configuration_error,
                    );
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

        SelectedTab::CellVoltageLimits => {
            if let Some(eb) = eb {
                let info = format!(
                    "Min Cell Voltage:   {:.3} V\n\
                     Max Cell Voltage:   {:.3} V\n\
                     Over Voltage Error: {:.3} V\n",
                    match eb.cell_voltage_min_max {
                        Some(ref v) => v.min_voltage_v,
                        _ => 0.0,
                    },
                    match eb.cell_voltage_min_max {
                        Some(ref v) => v.max_voltage_v,
                        _ => 0.0,
                    },
                    match eb.cell_voltage_limit {
                        Some(ref v) => v.over_voltage_error_v,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::BatteryVoltage => {
            if let Some(eb) = eb {
                let info = format!(
                    "SumOfCell Voltage:       {:.3} V\n\
                     Internal Connector:      {:.3} V\n\
                     External Connector:      {:.3} V\n\
                     Internal-External MinΔ:  {:.3} V\n",
                    match eb.battery_voltage {
                        Some(ref v) => v.sum_of_cell_voltage_v,
                        _ => 0.0,
                    },
                    match eb.battery_voltage {
                        Some(ref v) => v.internal_connector_voltage_v,
                        _ => 0.0,
                    },
                    match eb.battery_voltage {
                        Some(ref v) => v.external_connector_voltage_v,
                        _ => 0.0,
                    },
                    match eb.battery_voltage_limit {
                        Some(ref v) => v.internal_external_min_delta_v,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::BatteryCurrent => {
            if let Some(eb) = eb {
                let info = format!(
                    "Fast Current:           {:>10.2} A\n\
                     Weighted Avg Current:   {:>10.2} A\n\
                     Integrated Current:     {:>10.2} A\n\
                     Average 1s Current:     {:>10.2} A\n\
                     Average 10s Current:    {:>10.2} A\n\
                     Discharge SC Error:     {:>10.2} A\n",
                    match eb.battery_current {
                        Some(ref v) => v.fast_current_a,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(ref v) => v.weighted_avg_current_a,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(ref v) => v.integrated_current_a,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(ref v) => v.average_1s_current_a,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(ref v) => v.average_10s_current_a,
                        _ => 0.0,
                    },
                    match eb.battery_current_limit {
                        Some(ref v) => v.discharge_sc_error_a,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::FetTemperature => {
            if let Some(eb) = eb {
                let info = format!(
                    "FET Temperature 1:    {:>7.1} °C\n\
                     FET Temperature 2:    {:>7.1} °C\n\
                     Min FET Temperature:  {:>7.1} °C\n\
                     Max FET Temperature:  {:>7.1} °C\n\
                     Discharge Over Temp:  {:>7.1} °C\n",
                    match eb.fet_temperature {
                        Some(ref v) => v.temperature_1_c,
                        _ => 0.0,
                    },
                    match eb.fet_temperature {
                        Some(ref v) => v.temperature_2_c,
                        _ => 0.0,
                    },
                    match eb.fet_temperature_min_max {
                        Some(ref v) => v.min_temperature_c,
                        _ => 0.0,
                    },
                    match eb.fet_temperature_min_max {
                        Some(ref v) => v.max_temperature_c,
                        _ => 0.0,
                    },
                    match eb.fet_temperature_limit {
                        Some(ref v) => v.discharge_over_temp_c,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::CellTemperature => {
            if let Some(eb) = eb {
                let info = format!(
                    "Cell Temperature 1:   {:>7.1} °C\n\
                     Cell Temperature 2:   {:>7.1} °C\n\
                     Cell Temperature 3:   {:>7.1} °C\n\
                     Cell Temperature 4:   {:>7.1} °C\n\
                     Cell Temperature 5:   {:>7.1} °C\n\
                     Cell Temperature 6:   {:>7.1} °C\n\
                     Min Cell Temperature: {:>7.1} °C\n\
                     Max Cell Temperature: {:>7.1} °C\n\
                     Discharge Over Temp:  {:>7.1} °C\n",
                    match eb.cell_temperature {
                        Some(ref v) => v.temperature_1_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(ref v) => v.temperature_2_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(ref v) => v.temperature_3_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(ref v) => v.temperature_4_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(ref v) => v.temperature_5_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(ref v) => v.temperature_6_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature_min_max {
                        Some(ref v) => v.min_temperature_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature_min_max {
                        Some(ref v) => v.max_temperature_c,
                        _ => 0.0,
                    },
                    match eb.cell_temperature_limit {
                        Some(ref v) => v.discharge_over_temp_c,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::CellBalance => {
            if let Some(eb) = eb {
                let info = format!(
                    "Balance Status Register:      {:#05x}\n\
                     Balance FET Active:           {:#05x}\n\
                     Balance FET Active Persistent:{:#05x}\n\
                     Balance Start Diff Voltage:   {:.3} V\n",
                    match eb.cell_balance_status {
                        Some(ref v) => v.balance_status_register,
                        _ => 0u16,
                    },
                    match eb.cell_balance_status {
                        Some(ref v) => v.balance_fet_active,
                        _ => 0u16,
                    },
                    match eb.cell_balance_status {
                        Some(ref v) => v.balance_fet_active_persistent,
                        _ => 0u16,
                    },
                    match eb.cell_balance_limit {
                        Some(ref v) => v.balance_start_diff_voltage_v,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::Impedance => {
            if let Some(eb) = eb {
                if let Some(ref imp) = eb.cell_impedance {
                    let mut lines = String::new();
                    for (i, v) in imp.cell_impedances_mohm.iter().enumerate() {
                        lines.push_str(&format!("Cell {:>2} Impedance: {:>6} mΩ\n", i + 1, v));
                    }
                    lines.push_str(&format!("Low Temp Factor:  {:>6}\n", imp.low_temp_factor));
                    lines.push_str(&format!("High Temp Factor: {:>6}\n", imp.high_temp_factor));
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

        SelectedTab::Capacity => {
            if let Some(eb) = eb {
                let info = format!(
                    "Design Capacity:           {:>10.2} Ah\n\
                     Full Charge Capacity:      {:>10.2} Ah\n\
                     Remaining Capacity:        {:>10.2} Ah\n\
                     SOC:                       {:>10} %\n\
                     SOH:                       {:>10} %\n\
                     Total Discharged Capacity: {:>10.2} Ah\n\
                     Total Charged Capacity:    {:>10.2} Ah\n",
                    match eb.battery_capacity {
                        Some(ref v) => v.design_capacity_ah,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(ref v) => v.full_charge_capacity_ah,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(ref v) => v.remaining_capacity_ah,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(ref v) => format!("{:.1}", v.soc_percent),
                        _ => String::from("----"),
                    },
                    match eb.battery_capacity {
                        Some(ref v) => format!("{:.1}", v.soh_percent),
                        _ => String::from("----"),
                    },
                    match eb.battery_capacity {
                        Some(ref v) => v.total_discharged_capacity_ah,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(ref v) => v.total_charged_capacity_ah,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::CycleCount => {
            if let Some(eb) = eb {
                let info = format!(
                    "Discharge Cycles:                    {:>10}\n\
                     Discharge Learning Cycles:           {:>10}\n\
                     Discharge Cycles After Learning:     {:>10}\n\
                     Charge Cycles Completed:             {:>10}\n\
                     Charge Cycles Started:               {:>10}\n\
                     Discharge Use Detect:                {:>10}\n\
                     Charge Use Low Temperature:          {:>10}\n\
                     Charge Use Normal Temperature:       {:>10}\n\
                     Charge Use High Temperature:         {:>10}\n",
                    match eb.battery_cycle_count {
                        Some(ref v) => v.discharge_cycles,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.discharge_learning_cycles,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.discharge_cycles_after_learning,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.charge_cycles_completed,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.charge_cycles_started,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.discharge_use_detect,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.charge_use_low_temperature,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.charge_use_normal_temperature,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(ref v) => v.charge_use_high_temperature,
                        _ => 0u32,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::ChargeParameters => {
            if let Some(eb) = eb {
                let info = format!(
                    "Charge Voltage Valid:    {:.3} V\n\
                     Charge Max Voltage:      {:.3} V\n\
                     Charge Keep Power Volt:  {:.3} V\n\
                     Charge Current Valid:    {:.3} A\n\
                     Charge Max Current N:    {:.3} A\n\
                     Charge Max Current Low:  {:.3} A\n\
                     Charge Max Current High: {:.3} A\n\
                     Charge Keep Power Curr:  {:.3} A\n\
                     Charge Temp Min Low:     {:>7.1} °C\n\
                     Charge Temp Min Normal:  {:>7.1} °C\n\
                     Charge Temp Max Normal:  {:>7.1} °C\n\
                     Charge Temp Max High:    {:>7.1} °C\n",
                    match eb.battery_charge_voltage {
                        Some(ref v) => v.charge_voltage_valid_v,
                        _ => 0.0,
                    },
                    match eb.battery_charge_voltage {
                        Some(ref v) => v.charge_max_voltage_v,
                        _ => 0.0,
                    },
                    match eb.battery_charge_voltage {
                        Some(ref v) => v.charge_keep_power_voltage_v,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(ref v) => v.charge_current_valid_a,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(ref v) => v.charge_max_current_n_a,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(ref v) => v.charge_max_current_low_a,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(ref v) => v.charge_max_current_high_a,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(ref v) => v.charge_keep_power_current_a,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(ref v) => v.temp_min_low_c,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(ref v) => v.temp_min_normal_c,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(ref v) => v.temp_max_normal_c,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(ref v) => v.temp_max_high_c,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },

        SelectedTab::MasterTemperature => {
            if let Some(eb) = eb {
                let info = format!(
                    "Master Max FET Temperature:  {:>7.1} °C\n\
                     Master Max Cell Temperature: {:>7.1} °C\n",
                    match eb.master_battery_temperature {
                        Some(ref v) => v.max_fet_temperature_c,
                        _ => 0.0,
                    },
                    match eb.master_battery_temperature {
                        Some(ref v) => v.max_cell_temperature_c,
                        _ => 0.0,
                    },
                );
                let text = Paragraph::new(info).wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            } else {
                let text = Paragraph::new("No module selected").wrap(Wrap { trim: true });
                f.render_widget(text, content_area);
            }
        },
    }

    if save_state.is_active() {
        let popup_width = 60;
        let popup_height = 10;
        let popup_area = Rect::new(
            area.width / 2 - popup_width / 2,
            area.height / 2 - popup_height / 2,
            popup_width,
            popup_height,
        );
        let eb_ref = varta.get_easyblade_by_index(selected);
        let completion = eb_ref.map(sdo_completion).unwrap_or(TOTAL_SDOS);
        draw_popup(f, popup_area, save_state, completion);
    }
}

struct ModuleSdoTask {
    sdo_request_tx: tokio::sync::mpsc::UnboundedSender<SdoRequest>,
    cancellation_token: tokio_util::sync::CancellationToken,
}

async fn easyblade_task(
    node_id: u8,
    mut sdo_client: zencan_client::SdoClient<
        zencan_client::common::SocketCanSender,
        zencan_client::common::SocketCanReceiver,
    >,
    mut sdo_request_rx: tokio::sync::mpsc::UnboundedReceiver<SdoRequest>,
    sdo_response_tx: tokio::sync::mpsc::UnboundedSender<SdoResponse>,
    cancellation_token: tokio_util::sync::CancellationToken,
) {
    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                break;
            }
            request = sdo_request_rx.recv() => {
                let response = match request {
                    Some(SdoRequest::SerialNumber) => {
                        SdoResponse::SerialNumber {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_serial_number(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::SoftwareVersion) => {
                        SdoResponse::SoftwareVersion {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_software_version(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::HardwareVersion) => {
                        SdoResponse::HardwareVersion {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_hardware_version(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceErrorHistory) => {
                        SdoResponse::DeviceErrorHistory {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_error_history(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellVoltages) => {
                        SdoResponse::CellVoltages {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_voltages(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceConfigInfo) => {
                        SdoResponse::DeviceConfigInfo {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_config_info(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceSerialNumberInfo) => {
                        SdoResponse::DeviceSerialNumberInfo {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_serial_number_info(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceDateInfo) => {
                        SdoResponse::DeviceDateInfo {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_date_info(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceVariantInfo) => {
                        SdoResponse::DeviceVariantInfo {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_variant_info(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceControlParam) => {
                        SdoResponse::DeviceControlParam {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_control_param(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceOperationTime) => {
                        SdoResponse::DeviceOperationTime {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_operation_time(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::DeviceErrorCounter) => {
                        SdoResponse::DeviceErrorCounter {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_device_error_counter(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellVoltageMinMax) => {
                        SdoResponse::CellVoltageMinMax {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_voltage_min_max(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellVoltageLimit) => {
                        SdoResponse::CellVoltageLimit {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_voltage_limit(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryVoltage) => {
                        SdoResponse::BatteryVoltage {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_voltage(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryVoltageLimit) => {
                        SdoResponse::BatteryVoltageLimit {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_voltage_limit(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryCurrent) => {
                        SdoResponse::BatteryCurrent {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_current(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryCurrentLimit) => {
                        SdoResponse::BatteryCurrentLimit {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_current_limit(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::FetTemperature) => {
                        SdoResponse::FetTemperature {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_fet_temperature(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::FetTemperatureMinMax) => {
                        SdoResponse::FetTemperatureMinMax {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_fet_temperature_min_max(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::FetTemperatureLimit) => {
                        SdoResponse::FetTemperatureLimit {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_fet_temperature_limit(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellTemperature) => {
                        SdoResponse::CellTemperature {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_temperature(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellTemperatureMinMax) => {
                        SdoResponse::CellTemperatureMinMax {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_temperature_min_max(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellTemperatureLimit) => {
                        SdoResponse::CellTemperatureLimit {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_temperature_limit(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellBalanceStatus) => {
                        SdoResponse::CellBalanceStatus {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_balance_status(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellBalanceLimit) => {
                        SdoResponse::CellBalanceLimit {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_balance_limit(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::CellImpedance) => {
                        SdoResponse::CellImpedance {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_cell_impedance(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryCapacity) => {
                        SdoResponse::BatteryCapacity {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_capacity(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryCapacityParam) => {
                        SdoResponse::BatteryCapacityParam {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_capacity_param(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryCycleCount) => {
                        SdoResponse::BatteryCycleCount {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_cycle_count(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryChargeVoltage) => {
                        SdoResponse::BatteryChargeVoltage {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_charge_voltage(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryChargeCurrent) => {
                        SdoResponse::BatteryChargeCurrent {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_charge_current(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::BatteryChargeTemperature) => {
                        SdoResponse::BatteryChargeTemperature {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_battery_charge_temperature(&mut sdo_client).await,
                        }
                    },
                    Some(SdoRequest::MasterBatteryTemperature) => {
                        SdoResponse::MasterBatteryTemperature {
                            node_id,
                            value: varta_easyblade::Varta::sdo_read_master_battery_temperature(&mut sdo_client).await,
                        }
                    },
                    None => break,
                };
                let _ = sdo_response_tx.send(response);
            }
        }
    }
}

fn spawn_module_task(
    can_interface: &str,
    node_id: u8,
    sdo_response_tx: tokio::sync::mpsc::UnboundedSender<SdoResponse>,
) -> ModuleSdoTask {
    let (sdo_request_tx, sdo_request_rx) = tokio::sync::mpsc::unbounded_channel();
    let (socketcan_tx, socketcan_rx) = zencan_client::open_socketcan(can_interface).unwrap();
    let sdo_client = zencan_client::SdoClient::new_std(node_id, socketcan_tx, socketcan_rx);
    let cancellation_token = tokio_util::sync::CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();

    tokio::spawn(async move {
        easyblade_task(
            node_id,
            sdo_client,
            sdo_request_rx,
            sdo_response_tx,
            cancellation_token_clone,
        )
        .await;
    });

    ModuleSdoTask { sdo_request_tx, cancellation_token }
}

fn burst_initial_sdos(tx: &tokio::sync::mpsc::UnboundedSender<SdoRequest>) {
    tx.send(SdoRequest::SerialNumber).ok();
    tx.send(SdoRequest::SoftwareVersion).ok();
    tx.send(SdoRequest::HardwareVersion).ok();
    tx.send(SdoRequest::DeviceErrorHistory).ok();
    tx.send(SdoRequest::CellVoltages).ok();
    tx.send(SdoRequest::DeviceConfigInfo).ok();
    tx.send(SdoRequest::DeviceSerialNumberInfo).ok();
    tx.send(SdoRequest::DeviceDateInfo).ok();
    tx.send(SdoRequest::DeviceVariantInfo).ok();
    tx.send(SdoRequest::DeviceControlParam).ok();
    tx.send(SdoRequest::DeviceOperationTime).ok();
    tx.send(SdoRequest::DeviceErrorCounter).ok();
    tx.send(SdoRequest::CellVoltageMinMax).ok();
    tx.send(SdoRequest::CellVoltageLimit).ok();
    tx.send(SdoRequest::BatteryVoltage).ok();
    tx.send(SdoRequest::BatteryVoltageLimit).ok();
    tx.send(SdoRequest::BatteryCurrent).ok();
    tx.send(SdoRequest::BatteryCurrentLimit).ok();
    tx.send(SdoRequest::FetTemperature).ok();
    tx.send(SdoRequest::FetTemperatureMinMax).ok();
    tx.send(SdoRequest::FetTemperatureLimit).ok();
    tx.send(SdoRequest::CellTemperature).ok();
    tx.send(SdoRequest::CellTemperatureMinMax).ok();
    tx.send(SdoRequest::CellTemperatureLimit).ok();
    tx.send(SdoRequest::CellBalanceStatus).ok();
    tx.send(SdoRequest::CellBalanceLimit).ok();
    tx.send(SdoRequest::CellImpedance).ok();
    tx.send(SdoRequest::BatteryCapacity).ok();
    tx.send(SdoRequest::BatteryCapacityParam).ok();
    tx.send(SdoRequest::BatteryCycleCount).ok();
    tx.send(SdoRequest::BatteryChargeVoltage).ok();
    tx.send(SdoRequest::BatteryChargeCurrent).ok();
    tx.send(SdoRequest::BatteryChargeTemperature).ok();
    tx.send(SdoRequest::MasterBatteryTemperature).ok();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut varta = varta_easyblade::Varta::new(&args.can_interface).await?;
    let (sdo_response_tx, mut sdo_response_rx) = tokio::sync::mpsc::unbounded_channel();

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
    let mut module_tasks: HashMap<u8, ModuleSdoTask> = HashMap::new();
    let mut save_state = SaveState::Idle;

    loop {
        let count = varta.easyblade_count();
        if selected >= count {
            selected = count.saturating_sub(1);
        }

        let current_tab = selected_tab;
        let current_save_state = save_state.clone();
        terminal.draw(|f| draw_frame(f, &varta, selected, current_tab, &current_save_state))?;

        tokio::select! {
            result = varta.process_socketcan_msg() => {
                match result {
                    Ok(Some(node_id)) => {
                        let task = spawn_module_task(&varta.canbus_interface, node_id, sdo_response_tx.clone());
                        burst_initial_sdos(&task.sdo_request_tx);
                        module_tasks.insert(node_id, task);
                    },
                    Ok(None) => {},
                    Err(e) => {
                        eprintln!("Error processing CAN message: {e}");
                    },
                }
                expire_timer = Box::pin(tokio::time::sleep(varta.next_expiry_delay()));
            }
            _ = expire_timer.as_mut() => {
                varta.expire_missing_modules();
                module_tasks.retain(|node_id, task| {
                    if varta.easyblades[*node_id as usize].is_some() {
                        true
                    } else {
                        task.cancellation_token.cancel();
                        false
                    }
                });
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
                        varta_easyblade::SdoResponse::DeviceConfigInfo { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_config_info = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::DeviceSerialNumberInfo { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_serial_number_info = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::DeviceDateInfo { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_date_info = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::DeviceVariantInfo { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_variant_info = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::DeviceControlParam { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_control_param = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::DeviceOperationTime { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_operation_time = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::DeviceErrorCounter { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.device_error_counter = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellVoltageMinMax { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_voltage_min_max = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellVoltageLimit { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_voltage_limit = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryVoltage { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_voltage = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryVoltageLimit { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_voltage_limit = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryCurrent { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_current = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryCurrentLimit { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_current_limit = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::FetTemperature { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.fet_temperature = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::FetTemperatureMinMax { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.fet_temperature_min_max = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::FetTemperatureLimit { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.fet_temperature_limit = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellTemperature { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_temperature = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellTemperatureMinMax { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_temperature_min_max = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellTemperatureLimit { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_temperature_limit = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellBalanceStatus { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_balance_status = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellBalanceLimit { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_balance_limit = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::CellImpedance { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.cell_impedance = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryCapacity { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_capacity = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryCapacityParam { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_capacity_param = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryCycleCount { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_cycle_count = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryChargeVoltage { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_charge_voltage = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryChargeCurrent { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_charge_current = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::BatteryChargeTemperature { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.battery_charge_temperature = Some(v);
                            }
                        },
                        varta_easyblade::SdoResponse::MasterBatteryTemperature { node_id, value: Ok(v) } => {
                            if let Some(Some(eb)) = varta.easyblades.get_mut(node_id as usize) {
                                eb.master_battery_temperature = Some(v);
                            }
                        },
                        _ => {
                            // Errors are silently dropped for now
                        },
                    }
                }
                try_complete_save(&varta, selected, &mut save_state);
            }
            event = rx.recv() => {
                if let Some(Event::Key(key)) = event
                    && key.kind == KeyEventKind::Press
                {
                    match key.code {
                        KeyCode::Char('q') => {
                            if save_state.is_active() {
                                save_state = SaveState::Idle;
                            } else {
                                break;
                            }
                        },
                        KeyCode::Enter if save_state.is_active() => {
                            save_state = SaveState::Idle;
                        },
                        KeyCode::Char('s') if !save_state.is_active() && count > 0 => {
                            let eb = varta.get_easyblade_by_index(selected);
                            if let Some(eb) = eb {
                                let serial = eb.serial_number.unwrap_or(0);
                                let file_path = format!("/tmp/varta-easyblade-{}.json", serial);
                                save_state = SaveState::Saving { file_path };
                                try_complete_save(&varta, selected, &mut save_state);
                            }
                        },
                        KeyCode::Up if count > 0 && !save_state.is_active() => {
                            selected = selected.saturating_sub(1);
                        }
                        KeyCode::Down if selected + 1 < count && !save_state.is_active() => {
                            selected += 1;
                        }
                        KeyCode::Left if !save_state.is_active() => {
                            selected_tab = selected_tab.cycle(false);
                        }
                        KeyCode::Right if !save_state.is_active() => {
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
