use std::env;
use std::time::Duration;
use time::{Date, Month, PrimitiveDateTime, Time};
use ureq::Agent;

pub struct Client {
    agent: Agent,
    url: String,
    token: String,
    pub location: String,
}

impl Client {
    pub fn from_env() -> Result<Self, String> {
        let base_url = env::var("INFLUXDB_URL").map_err(|_| "INFLUXDB_URL not set")?;
        let token = env::var("INFLUXDB_TOKEN").map_err(|_| "INFLUXDB_TOKEN not set")?;
        let org = env::var("INFLUXDB_ORG").map_err(|_| "INFLUXDB_ORG not set")?;
        let bucket = env::var("INFLUXDB_BUCKET").map_err(|_| "INFLUXDB_BUCKET not set")?;
        let location = env::var("INFLUXDB_LOCATION").unwrap_or_else(|_| "default".to_string());

        let url = format!("{}/api/v2/write?org={}&bucket={}", base_url, org, bucket);

        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(30)))
            .build();
        let agent: Agent = config.into();

        Ok(Client {
            agent,
            url,
            token,
            location,
        })
    }

    pub fn write(&self, line_protocol: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.agent
            .post(&self.url)
            .header("Authorization", &format!("Token {}", self.token))
            .header("Content-Type", "text/plain; charset=utf-8")
            .send(line_protocol)?;

        Ok(())
    }
}

fn tst_to_nanos(tst: &dsmr5::types::TST) -> Option<i64> {
    let year = 2000 + tst.year as i32;
    let month = Month::try_from(tst.month).ok()?;
    let date = Date::from_calendar_date(year, month, tst.day).ok()?;
    let time = Time::from_hms(tst.hour, tst.minute, tst.second).ok()?;
    let datetime = PrimitiveDateTime::new(date, time);

    // TST contains local time (CET/CEST for Dutch meters)
    // dst=false: CET (UTC+1), dst=true: CEST (UTC+2)
    let offset_hours = if tst.dst { 2 } else { 1 };
    let utc_datetime = datetime.assume_utc() - time::Duration::hours(offset_hours);

    Some(utc_datetime.unix_timestamp_nanos() as i64)
}

pub fn format_power_reading(state: &dsmr5::state::State, location: &str) -> Option<String> {
    let timestamp = state.datetime.as_ref()?;
    let timestamp_nanos = tst_to_nanos(timestamp)?;

    let electricity_tariff_1 = state.meterreadings[0].to?;
    let electricity_tariff_2 = state.meterreadings[1].to?;

    let voltage_l1 = state.lines[0].voltage?;
    let voltage_l2 = state.lines[1].voltage?;
    let voltage_l3 = state.lines[2].voltage?;

    let active_power_l1 = state.lines[0].active_power_plus?;
    let active_power_l2 = state.lines[1].active_power_plus?;
    let active_power_l3 = state.lines[2].active_power_plus?;

    let current_usage = state.power_delivered?;

    Some(format!(
        "power_reading,location={} electricity_used_tariff_1={},electricity_used_tariff_2={},voltage_l1={:.2},voltage_l2={:.2},voltage_l3={:.2},active_power_l1={:.3},active_power_l2={:.3},active_power_l3={:.3},current_electricity_usage={} {}",
        location,
        electricity_tariff_1,
        electricity_tariff_2,
        voltage_l1,
        voltage_l2,
        voltage_l3,
        active_power_l1,
        active_power_l2,
        active_power_l3,
        current_usage,
        timestamp_nanos
    ))
}

pub fn format_gas_reading(state: &dsmr5::state::State, location: &str) -> Option<String> {
    let gas_slave = &state.slaves[0];

    if gas_slave.device_type != Some(3) {
        return None;
    }

    let (timestamp, reading) = gas_slave.meter_reading.as_ref()?;
    let timestamp_nanos = tst_to_nanos(timestamp)?;

    Some(format!(
        "gas_reading,location={} gas_meter_reading={} {}",
        location, reading, timestamp_nanos
    ))
}

