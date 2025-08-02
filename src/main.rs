use lm_sensors::{SubFeatureRef, Value, value::Kind};
use metrics::{Gauge, counter, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
};
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

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

    loop {
        sys.refresh_all();
        if cpu_count != sys.cpus().len() {
            panic!(
                "CPU count changed from {} to {}",
                cpu_count,
                sys.cpus().len()
            );
        }

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

        // thread::sleep(cmp::max(
        //     Duration::from_secs(1),
        //     MINIMUM_CPU_UPDATE_INTERVAL,
        // ));
        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    }
}
