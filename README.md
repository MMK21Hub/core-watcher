# Core watcher

> A Rust tool that watches your CPU cores (and other system metrics!) and exports them to Prometheus

## Screenshots

![An overview of the metrics, rendered in Grafana](assets/overview_metrics.png)

## Development instructions

Requires Rust and Cargo. Also requires Prometheus if you want to actually do anything with the metrics.

1. Run the program: `cargo run`
2. Run Prometheus: `prometheus --config.file=development/prometheus.yaml`
