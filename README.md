# dsmr_save

A Rust application that reads DSMR 5.0 smart meter telegrams from a serial port and stores the data in InfluxDB.

## Configuration

All configuration is done through environment variables.

### Required Environment Variables

| Variable | Description |
|----------|-------------|
| `INFLUXDB_URL` | Base URL of your InfluxDB instance (e.g., `http://localhost:8086`) |
| `INFLUXDB_TOKEN` | InfluxDB authentication token |
| `INFLUXDB_ORG` | InfluxDB organization name |
| `INFLUXDB_BUCKET` | InfluxDB bucket name |

### Optional Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERIAL_DEVICE` | `/dev/ttyUSB0` | Path to the serial device connected to the smart meter |
| `INFLUXDB_LOCATION` | `default` | Location tag added to measurements |
| `RUST_LOG` | `info` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |

## Data Points

The application writes two measurements to InfluxDB:

**power_reading** (updated every telegram):
- `electricity_used_tariff_1` - Total electricity used on tariff 1 (kWh)
- `electricity_used_tariff_2` - Total electricity used on tariff 2 (kWh)
- `voltage_l1`, `voltage_l2`, `voltage_l3` - Voltage per phase (V)
- `active_power_l1`, `active_power_l2`, `active_power_l3` - Active power per phase (kW)
- `current_electricity_usage` - Current power consumption (kW)

**gas_reading** (updated when gas meter reports):
- `gas_meter_reading` - Total gas consumption (m³)

## Running

```bash
export INFLUXDB_URL="http://localhost:8086"
export INFLUXDB_TOKEN="your-token"
export INFLUXDB_ORG="your-org"
export INFLUXDB_BUCKET="energy"
export SERIAL_DEVICE="/dev/ttyUSB0"
export INFLUXDB_LOCATION="home"

./dsmr_save
```

## Systemd Service

Create `/etc/systemd/system/dsmr_save.service`:

```ini
[Unit]
Description=DSMR Smart Meter to InfluxDB
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/dsmr_save
Restart=always
RestartSec=10

Environment="INFLUXDB_URL=http://localhost:8086"
Environment="INFLUXDB_TOKEN=your-token"
Environment="INFLUXDB_ORG=your-org"
Environment="INFLUXDB_BUCKET=energy"
Environment="SERIAL_DEVICE=/dev/ttyUSB0"
Environment="INFLUXDB_LOCATION=home"
Environment="RUST_LOG=info"

# Run as non-root user (optional)
User=dsmr
Group=dsmr

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
```

Enable and start the service:
```bash
sudo systemctl daemon-reload
sudo systemctl enable dsmr_save
sudo systemctl start dsmr_save
```

Check status and logs:

```bash
sudo systemctl status dsmr_save
sudo journalctl -u dsmr_save -f
```

**Note:** The user running the service needs read access to the serial device. Add the user to the `dialout` group:

```bash
sudo usermod -a -G dialout dsmr
```
