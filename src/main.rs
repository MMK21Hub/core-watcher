use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread,
    time::Duration,
};

fn main() {
    let builder = PrometheusBuilder::new();
    let listen_on = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 9000);
    builder
        .with_http_listener(listen_on)
        .install()
        .expect("failed to install recorder/exporter");
    println!("Prometheus exporter listening on {}", listen_on);
    println!("Try accessing http://localhost:9000/metrics");

    loop {
        let count = counter!("test_counter", "service" => "amazing service");
        thread::sleep(Duration::from_secs(2));
        count.increment(1);
    }
}
