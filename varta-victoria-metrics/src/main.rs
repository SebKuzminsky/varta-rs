use clap::Parser;
use reqwest::Client;
use std::time::Duration;
use varta_easyblade::Varta;

#[derive(Debug, Parser)]
#[command(name = "varta-victoria-metrics")]
struct Args {
    /// Victoria Metrics HTTP URL (e.g. http://localhost:8428)
    #[arg(long)]
    vm_url: String,

    /// The CAN interface to connect to (e.g. 'can0' or 'vcan0').
    #[arg(long, default_value_t = String::from("can0"))]
    canbus: String,
}

#[derive(Debug)]
struct MetricPoint {
    metric: String,
    value: f64,
    labels: Vec<(&'static str, String)>,
    timestamp: i64,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

async fn send_metrics(client: &Client, vm_url: &str, points: &[MetricPoint]) -> Result<(), String> {
    for p in points {
        let mut obj = serde_json::Map::new();

        let mut metric_obj = serde_json::Map::new();
        metric_obj.insert(
            "__name__".into(),
            serde_json::Value::String(p.metric.clone()),
        );
        obj.insert("metric".into(), serde_json::Value::Object(metric_obj));

        let mut values = Vec::new();
        values.push(serde_json::Value::Number(
            serde_json::Number::from_f64(p.value).unwrap(),
        ));
        obj.insert("values".into(), values.into());

        let mut timestamps = Vec::new();
        timestamps.push(serde_json::Number::from(p.timestamp));
        obj.insert("timestamps".into(), timestamps.into());

        obj.insert(
            "timestamp".into(),
            serde_json::Value::Number(serde_json::Number::from(p.timestamp)),
        );

        // if !p.labels.is_empty() {
        //     let mut labels_map = serde_json::Map::new();
        //     for (k, v) in &p.labels {
        //         labels_map.insert(k.to_string(), serde_json::Value::String(v.clone()));
        //     }
        //     obj.insert("__labels__".into(), serde_json::Value::Object(labels_map));
        // }

        let import_body = serde_json::to_vec(&obj).map_err(|e| e.to_string())?;

        client
            .post(format!("{}/api/v1/import", vm_url))
            .header("Content-Type", "application/json")
            .body(import_body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn collect_metrics(varta: &Varta) -> Vec<MetricPoint> {
    let mut points = Vec::new();
    let ts = now_ms();

    if let Some(voltage) = varta.master.voltage {
        points.push(MetricPoint {
            metric: "varta_master_voltage".to_string(),
            value: voltage as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(current) = varta.master.current {
        points.push(MetricPoint {
            metric: "varta_master_current".to_string(),
            value: current as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(soc) = varta.master.soc {
        points.push(MetricPoint {
            metric: "varta_master_soc".to_string(),
            value: soc as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(charge_voltage_request) = varta.master.charge_voltage_request {
        points.push(MetricPoint {
            metric: "varta_master_charge_voltage_request".to_string(),
            value: charge_voltage_request as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(charge_current_request) = varta.master.charge_current_request {
        points.push(MetricPoint {
            metric: "varta_master_charge_current_request".to_string(),
            value: charge_current_request as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(max_fet_temp) = varta.master.max_battery_fet_temp {
        points.push(MetricPoint {
            metric: "varta_master_max_fet_temp".to_string(),
            value: max_fet_temp as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(max_cell_temp) = varta.master.max_battery_cell_temp {
        points.push(MetricPoint {
            metric: "varta_master_max_cell_temp".to_string(),
            value: max_cell_temp as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(full_cap) = varta.master.master_full_charge_capacity {
        points.push(MetricPoint {
            metric: "varta_master_full_charge_capacity".to_string(),
            value: full_cap as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    if let Some(remaining_cap) = varta.master.master_remaining_capacity {
        points.push(MetricPoint {
            metric: "varta_master_remaining_capacity".to_string(),
            value: remaining_cap as f64,
            labels: Vec::new(),
            timestamp: ts,
        });
    }

    for (idx, entry) in varta.easyblades.iter().enumerate() {
        let node_id = idx as u8;
        let node_label = vec![("node_id", node_id.to_string())];

        if let Some(eb) = entry {
            if let Some(voltage) = eb.voltage {
                points.push(MetricPoint {
                    metric: "varta_module_voltage".to_string(),
                    value: voltage as f64,
                    labels: node_label.clone(),
                    timestamp: ts,
                });
            }

            if let Some(current) = eb.current {
                points.push(MetricPoint {
                    metric: "varta_module_current".to_string(),
                    value: current as f64,
                    labels: node_label.clone(),
                    timestamp: ts,
                });
            }

            if let Some(soc) = eb.soc {
                points.push(MetricPoint {
                    metric: "varta_module_soc".to_string(),
                    value: soc as f64,
                    labels: node_label.clone(),
                    timestamp: ts,
                });
            }

            if let Some(soh) = eb.soh {
                points.push(MetricPoint {
                    metric: "varta_module_soh".to_string(),
                    value: soh as f64,
                    labels: node_label.clone(),
                    timestamp: ts,
                });
            }

            if let Some((charge_fet, discharge_fet, bypass_fet)) = eb.fet_status {
                points.push(MetricPoint {
                    metric: "varta_module_charge_fet".to_string(),
                    value: if charge_fet { 1.0 } else { 0.0 },
                    labels: node_label.clone(),
                    timestamp: ts,
                });
                points.push(MetricPoint {
                    metric: "varta_module_discharge_fet".to_string(),
                    value: if discharge_fet { 1.0 } else { 0.0 },
                    labels: node_label.clone(),
                    timestamp: ts,
                });
                points.push(MetricPoint {
                    metric: "varta_module_bypass_fet".to_string(),
                    value: if bypass_fet { 1.0 } else { 0.0 },
                    labels: node_label.clone(),
                    timestamp: ts,
                });
            }

            if let Some(serial) = eb.serial_number {
                points.push(MetricPoint {
                    metric: "varta_module_serial".to_string(),
                    value: serial as f64,
                    labels: node_label.clone(),
                    timestamp: ts,
                });
            }
        }
    }

    points
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let client = Client::new();

    let mut varta = match Varta::new(&args.canbus).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error opening CAN interface {}: {}", args.canbus, e);
            std::process::exit(1);
        },
    };

    eprintln!(
        "Connected to CAN interface {}, streaming to {}",
        args.canbus, args.vm_url
    );

    let mut batch_interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            result = varta.process_socketcan_msg() => {
                match result {
                    Ok(_) => { },
                    Err(e) => {
                        eprintln!("Error reading CAN: {}", e);
                        std::process::exit(1);
                    },
                }
            },
            _ = batch_interval.tick() => {
                let points = collect_metrics(&varta);
                if !points.is_empty()
                    && let Err(e) = send_metrics(&client, &args.vm_url, &points).await
                {
                    eprintln!("Error sending metrics: {}", e);
                }
            },
        }
    }
}
