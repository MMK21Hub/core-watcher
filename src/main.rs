use metrics_exporter_prometheus::PrometheusBuilder;

fn main() {
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install recorder/exporter");
}
