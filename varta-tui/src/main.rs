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
                        Some(ref v) => v.0,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.1,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.2,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.3,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.4,
                        _ => 0,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.5,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.6,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.7,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.8,
                        _ => 0u32,
                    },
                    match eb.device_operation_time {
                        Some(ref v) => v.9,
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
                    let lines: String = counters
                        .iter()
                        .enumerate()
                        .map(|(i, v)| format!("Error {:>2}: {:>6}\n", i + 1, v))
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

        SelectedTab::CellVoltageLimits => {
            if let Some(eb) = eb {
                let info = format!(
                    "Min Cell Voltage:   {:.3} V\n\
                     Max Cell Voltage:   {:.3} V\n\
                     Over Voltage Error: {:.3} V\n",
                    match eb.cell_voltage_min_max {
                        Some(v) => v.0 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.cell_voltage_min_max {
                        Some(v) => v.1 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.cell_voltage_limit {
                        Some(v) => v as f32 / 1000.0,
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
                        Some(v) => v.0 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_voltage {
                        Some(v) => v.1 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_voltage {
                        Some(v) => v.2 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_voltage_limit {
                        Some(v) => v as f32 / 1000.0,
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
                        Some(v) => v.0 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(v) => v.1 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(v) => v.2 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(v) => v.3 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_current {
                        Some(v) => v.4 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_current_limit {
                        Some(v) => v as f32 / 1000.0,
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
                        Some(v) => v.0 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.fet_temperature {
                        Some(v) => v.1 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.fet_temperature_min_max {
                        Some(v) => v.0 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.fet_temperature_min_max {
                        Some(v) => v.1 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.fet_temperature_limit {
                        Some(v) => v as f32 / 10.0,
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
                        Some(v) => v.0 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(v) => v.1 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(v) => v.2 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(v) => v.3 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(v) => v.4 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature {
                        Some(v) => v.5 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature_min_max {
                        Some(v) => v.0 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature_min_max {
                        Some(v) => v.1 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.cell_temperature_limit {
                        Some(v) => v as f32 / 10.0,
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
                        Some(v) => v.0,
                        _ => 0u16,
                    },
                    match eb.cell_balance_status {
                        Some(v) => v.1,
                        _ => 0u16,
                    },
                    match eb.cell_balance_status {
                        Some(v) => v.2,
                        _ => 0u16,
                    },
                    match eb.cell_balance_limit {
                        Some(v) => v as f32 / 1000.0,
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
                    let lines: String = imp
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            if i < 16 {
                                format!("Cell {:>2} Impedance: {:>6} mΩ\n", i + 1, v)
                            } else if i == 16 {
                                format!("Low Temp Factor:  {:>6}\n", v)
                            } else {
                                format!("High Temp Factor: {:>6}\n", v)
                            }
                        })
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
                        Some(v) => v.0 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(v) => v.1 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(v) => v.2 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(v) => format!("{:.1}", v.3 as f32),
                        _ => String::from("----"),
                    },
                    match eb.battery_capacity {
                        Some(v) => format!("{:.1}", v.4 as f32),
                        _ => String::from("----"),
                    },
                    match eb.battery_capacity {
                        Some(v) => v.5 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_capacity {
                        Some(v) => v.6 as f32 / 1000.0,
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
                        Some(v) => v.0,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.1,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.2,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.3,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.4,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.5,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.6,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.7,
                        _ => 0u32,
                    },
                    match eb.battery_cycle_count {
                        Some(v) => v.8,
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
                        Some(v) => v.0 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_voltage {
                        Some(v) => v.1 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_voltage {
                        Some(v) => v.2 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(v) => v.0 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(v) => v.1 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(v) => v.2 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(v) => v.3 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_current {
                        Some(v) => v.4 as f32 / 1000.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(v) => v.0 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(v) => v.1 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(v) => v.2 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.battery_charge_temperature {
                        Some(v) => v.3 as f32 / 10.0,
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
                        Some(v) => v.0 as f32 / 10.0,
                        _ => 0.0,
                    },
                    match eb.master_battery_temperature {
                        Some(v) => v.1 as f32 / 10.0,
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
