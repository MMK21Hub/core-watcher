# Core Watcher

A Rust tool that watches your Linux system's CPU cores (and other system metrics!) and exports them to Prometheus.

## Screenshots

![An overview of the metrics, rendered in Grafana](assets/overview_metrics.png)

## Running locally

Only Linux is supported. Currently, only pre-compiled binaries for x86_64 are available, but you can compile it yourself for other architectures.

### Step 1: Quick start

1. Download the Linux x86_64 binary from the [latest release](https://github.com/MMK21Hub/core-watcher/releases/latest)
2. Make it executable and run it:

   ```bash
   chmod +x core-watcher
   ./core-watcher
   ```

3. Core Watcher is now running! 🎉
4. Head to <http://localhost:9000/metrics> to verify that it's producing metrics as expected.

### Step 2: Add Prometheus

Prometheus isn't strictly required to run the program, but you'll need it if you want to actually track the metrics, so that you can visualise them in a Grafana dashboard or something like that.

#### Option A: Installing Prometheus locally

The easiest way to run Prometheus is to install it to your system.

1. Install Prometheus, e.g. by using your system's package manager: `sudo apt install prometheus` or `sudo pacman -S prometheus`

   ```bash
   sudo apt install prometheus # For Debian/Ubuntu
   sudo pacman -S prometheus # For Arch Linux
   ```

2. Run Prometheus:

   ```bash
   prometheus --config.file=development/prometheus.yaml
   ```

#### Option B: Using Prometheus in Docker

As an alternative to installing Prometheus on your system, you can run it in a Docker container, like this:

```bash
docker run \
    -p 9090:9090 \
    -v ./development/prometheus.yaml:/etc/prometheus/prometheus.yml \
    prom/prometheus
```

#### Option C: Using an existing Prometheus instance

In production, you can of course use an existing Prometheus instance, or something compatible like VictoriaMetrics (which is what I use). Simply add a scrape config like the following:

<!-- prettier-ignore -->
```yaml
  - job_name: core_watcher
    scrape_interval: "1s"
    static_configs:
      - targets: ["arch-pc:9000"]
```

### Installing as a systemd service

You may want to install Core Watcher as a systemd service, so that it starts automatically on boot.

1. Install the binary globally: `sudo install -m755 target/release/core-watcher /usr/local/bin/core-watcher`
2. Install the systemd service file: `sudo install -m644 production/core-watcher.service /etc/systemd/system/core-watcher.service`
3. Enable and start the service: `sudo systemctl enable --now core-watcher`
   - This will set Core Watcher to start automatically on boot
4. Check that it's started properly: `sudo systemctl status core-watcher`

### Compiling from source

You only need to do this if a pre-compiled binary isn't available for your architecture, or if you want to be sure that you can trust the compiled binary.

Ensure that you have [installed Rust](https://www.rust-lang.org/tools/install) first.

1. Clone the repository and `cd` into it as per usual
2. Run the program: `cargo run`
3. Optionally, run Prometheus: `prometheus --config.file=development/prometheus.yaml`

## Contributing

See above for instructions for compiling the program from source and running it locally.

### Publishing a release

1. Bump version in `Cargo.toml`
2. Make sure `Cargo.lock` has updated with the new version number (done automatically if you use rust-analyzer)
3. Commit the version bump to the `master` branch
4. Wait for the GitHub Action to build all the binaries for the various platforms
5. Download all the zip file artifacts from the GitHub Action
6. Create a GitHub release, tell it to create a new tag (e.g. `v0.4.1`), upload the zip files to the assets/binaries section, write a little changelog, and publish it as the latest release
