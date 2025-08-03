# Core watcher

> A Rust tool that watches your CPU cores (and other system metrics!) and exports them to Prometheus

## Screenshots

![An overview of the metrics, rendered in Grafana](assets/overview_metrics.png)

## Running locally

### Installing dependencies

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Install Prometheus, e.g. by using your system's package manager. This isn't strictly required to run the program, but is needed if you want to use the metrics in a Grafana dashboard or something.

### Running the project

1. Clone the repository and `cd` into it as per usual
2. Run the program: `cargo run`
3. Check that it's working by visiting <http://localhost:9000/metrics>
4. Optionally, run Prometheus: `prometheus --config.file=development/prometheus.yaml`

### Using Prometheus in Docker

As an alternative to installing Prometheus on your system, you can run it in a Docker container, like this:

```bash
docker run \
    -p 9090:9090 \
    -v ./development/prometheus.yaml:/etc/prometheus/prometheus.yml \
    prom/prometheus
```

### Using an existing Prometheus instance

In production, you can of course use an existing Prometheus instance, or something compatible like VictoriaMetrics (which is what I use). Simply add a scrape config like the following:

<!-- prettier-ignore -->
```yaml
  - job_name: core_watcher
    scrape_interval: "1s"
    static_configs:
      - targets: ["arch-pc:9000"]
```
