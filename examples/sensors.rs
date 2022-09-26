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

// generates completion suggestions
fn sensor_device_comp(input: &String) -> Vec<(String, Option<String>)> {
    [
        ("temp100", "Main temperature sensor"),
        ("temp101", "Output temperature sensor"),
        ("tank01", "Temperature in a storage tank 1"),
        ("tank02", "Temperature in a storage tank 2"),
        ("tank03", "Temperature in a storage tank 3"),
        ("outdoor", "Outdoor temperature sensor"),
    ]
    .iter()
    .filter_map(|(name, descr)| {
        if name.starts_with(input) {
            Some((name.to_string(), Some(descr.to_string())))
        } else {
            None
        }
    })
    .collect::<Vec<_>>()
}

fn opts() -> Opts {
    let sensor = long("sensor").req_flag(());
    let device = long("sensor-device")
        .argument::<String>("DEVICE")
        .complete(sensor_device_comp);
    let name = long("sensor-name").argument::<String>("NAME");

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
