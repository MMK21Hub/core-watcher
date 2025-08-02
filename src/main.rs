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

    // Initialize metrics
    let count = counter!("test_counter", "service" => "amazing service");
    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_usages: Vec<Gauge> = sys
        .cpus()
        .iter()
        .enumerate()
        .map(|(i, cpu)| gauge!(format!("cpu_{}_usage", i), "cpu_name" => cpu.name().to_string()))
        .collect();
    let cpu_count = sys.cpus().len();

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
        }

        // thread::sleep(cmp::max(
        //     Duration::from_secs(1),
        //     MINIMUM_CPU_UPDATE_INTERVAL,
        // ));
        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    }
}
