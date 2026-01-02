mod influxdb;

use log::{debug, error, info};
use serial2::SerialPort;
use std::env;
use std::io::Read;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let influx_client = influxdb::Client::from_env()
        .expect("Failed to load InfluxDB configuration from environment variables");

    let serial_device = env::var("SERIAL_DEVICE")
        .unwrap_or_else(|_| "/dev/ttyUSB0".to_string());

    info!("Opening serial device: {}", serial_device);

    let mut port = SerialPort::open(&serial_device, |mut settings: serial2::Settings| {
        settings.set_baud_rate(115200)?;
        settings.set_char_size(serial2::CharSize::Bits8);
        settings.set_stop_bits(serial2::StopBits::One);
        settings.set_parity(serial2::Parity::None);
        settings.set_flow_control(serial2::FlowControl::None);
        Ok(settings)
    })?;
    port.set_read_timeout(Duration::from_secs(20))?;

    info!("Connected, reading DSMR telegrams...");

    let reader = dsmr5::Reader::new(port.bytes());

    for readout in reader {
        let readout = readout.unwrap();
        let telegram = readout.to_telegram().unwrap();
        let state = dsmr5::Result::<dsmr5::state::State>::from(&telegram).unwrap();

        debug!("Received state: {:#?}", state);

        if let Some(power_line) = influxdb::format_power_reading(&state, &influx_client.location) {
            debug!("Power line protocol: {}", power_line);
            if let Err(e) = influx_client.write(&power_line) {
                error!("Failed to write power reading: {}", e);
            }
        }

        if let Some(gas_line) = influxdb::format_gas_reading(&state, &influx_client.location) {
            debug!("Gas line protocol: {}", gas_line);
            if let Err(e) = influx_client.write(&gas_line) {
                error!("Failed to write gas reading: {}", e);
            }
        }
    }

    Ok(())
}

