use lm_sensors::{SubFeatureRef, Value, value::Kind};
use metrics::{Gauge, counter, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
};
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, Networks, System};

struct NetGauges {
    received_bytes: Gauge,
    transmitted_bytes: Gauge,
    received_packets: Gauge,
    transmitted_packets: Gauge,
    receive_errors: Gauge,
    transmit_errors: Gauge,
}

fn main() {
    // Set up Prometheus exporter
    let builder = PrometheusBuilder::new();
    let listen_on = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 9000);
    builder
        .with_http_listener(listen_on)
        .install()
        .expect("failed to install recorder/exporter");
    println!("Prometheus exporter listening on {}", listen_on);
    println!("Try accessing http://localhost:9000/metrics");

    // Initialize data collection
    let mut sys = System::new_all();
    sys.refresh_all();
    let sensors = lm_sensors::Initializer::default().initialize().unwrap();
    // Initialize metrics
    let count = counter!("test_counter", "service" => "amazing service");
    let cpu_count = sys.cpus().len();
    let cpu_usages: Vec<Gauge> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| gauge!("cpu_usage_percent", "cpu_name" => cpu.name().to_string(), "cpu_number" => (i + 1).to_string()) )
        .collect();
    let cpu_frequencies: Vec<Gauge> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| gauge!("cpu_frequency_megahertz", "cpu_name" => cpu.name().to_string(), "cpu_number" => (i + 1).to_string()) )
        .collect();

    let mut temperature_sensors: Vec<SubFeatureRef<'_>> = Vec::new();
    for chip in sensors.chip_iter(None) {
        for feature in chip.feature_iter() {
            for sub_feature in feature.sub_feature_iter() {
                match sub_feature.kind() {
                    Some(Kind::TemperatureInput) => {
                        temperature_sensors.push(sub_feature);
                    }
                    _ => {}
                }
            }
        }
    }
    let temperature_gauges: Vec<Gauge> = temperature_sensors
        .iter()
        .map(|sensor| {
            let chip = sensor.feature().chip();
            let chip_name = chip.name().unwrap_or_else(|_| match chip.path() {
                Some(path) => format!("unknown_{}", path.to_string_lossy().replace("/", "_")),
                None => format!("unknown_{}_{}", chip.bus().number(), chip.raw_address()),
            });
            let feature_name = sensor.feature().to_string();
            gauge!("hwmon_temperature_celsius",
                    "sensor_chip" => chip_name,
                    "sensor_name" => feature_name,
            )
        })
        .collect();

    let mut networks = Networks::new_with_refreshed_list();
    let mut net_gauges: HashMap<String, NetGauges> = HashMap::new();
    for (interface_name, _) in networks.iter() {
        net_gauges.insert(
            interface_name.to_string(),
            NetGauges {
                received_bytes: gauge!("network_received_bytes_total", "interface" => interface_name.to_string()),
                transmitted_bytes: gauge!("network_transmitted_bytes_total", "interface" => interface_name.to_string()),
                received_packets: gauge!("network_received_packets_total", "interface" => interface_name.to_string()),
                transmitted_packets: gauge!("network_transmitted_packets_total", "interface" => interface_name.to_string()),
                receive_errors: gauge!("network_receive_errors_total", "interface" => interface_name.to_string()),
                transmit_errors: gauge!("network_transmit_errors_total", "interface" => interface_name.to_string()),
            },
        );
    }

    loop {
        sys.refresh_all();
        if cpu_count != sys.cpus().len() {
            panic!(
                "CPU count changed from {} to {}",
                cpu_count,
                sys.cpus().len()
            );
        }
        networks.refresh(false);

        // Update metrics
        count.increment(1);
        for (i, cpu) in sys.cpus().iter().enumerate() {
            cpu_usages[i].set(cpu.cpu_usage());
            cpu_frequencies[i].set(cpu.frequency() as f64);
        }
        for (i, sensor) in temperature_sensors.iter().enumerate() {
            let value = match sensor.value() {
                Ok(Value::TemperatureInput(temperature)) => temperature,
                Ok(_) => {
                    eprintln!("Unexpected value type for sensor: {:?}", sensor.value());
                    continue;
                }
                Err(_) => continue,
            };
            temperature_gauges[i].set(value);
        }
        for (interface_name, network) in networks.iter() {
            if let Some(gauges) = net_gauges.get(interface_name) {
                gauges.received_bytes.set(network.total_received() as f64);
                gauges
                    .transmitted_bytes
                    .set(network.total_transmitted() as f64);
                gauges
                    .received_packets
                    .set(network.total_packets_received() as f64);
                gauges
                    .transmitted_packets
                    .set(network.total_packets_transmitted() as f64);
                gauges
                    .receive_errors
                    .set(network.total_errors_on_received() as f64);
                gauges
                    .transmit_errors
                    .set(network.total_errors_on_transmitted() as f64);
            }
        }

        // thread::sleep(cmp::max(
        //     Duration::from_secs(1),
        //     MINIMUM_CPU_UPDATE_INTERVAL,
        // ));
        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    }
}
