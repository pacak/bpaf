//! I have group of options which could be used multiple times. All such groups should be kept without overwriting previous one.
//!
//! $ prometheus_sensors_exporter \
//!     \
//!     `# 2 physical sensors located on physycial different i2c bus or address` \
//!     --sensor \
//!         --sensor-device=tmp102 \
//!         --sensor-name="temperature_tmp102_outdoor" \
//!         --sensor-i2c-bus=0 \
//!         --sensor-i2c-address=0x48 \
//!     --sensor \
//!         --sensor-device=tmp102 \
//!         --sensor-name="temperature_tmp102_indoor" \
//!         --sensor-i2c-bus=1 \
//!         --sensor-i2c-address=0x49 \

use bpaf::*;
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Sensor {
    sensor: (),
    device: String,
    name: String,
    bus_id: usize,
    address: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Opts {
    sensors: Vec<Sensor>,
}

fn opts() -> Opts {
    let sensor = long("sensor").req_flag(());
    let device = long("sensor-device").argument("DEVICE");
    let name = long("sensor-name").argument("NAME");

    // from_str needs to be replaced with `parse` that can deal with hex digits
    let bus_id = long("sensor-i2c-bus").argument::<usize>("BUS");
    let address = long("sensor-i2c-address").argument::<usize>("ADDRESS");
    let sensors = construct!(Sensor {
        sensor,
        device,
        name,
        bus_id,
        address
    })
    .adjacent()
    .many();
    construct!(Opts { sensors }).to_options().run()
}

fn main() {
    println!("{:#?}", opts());
}
