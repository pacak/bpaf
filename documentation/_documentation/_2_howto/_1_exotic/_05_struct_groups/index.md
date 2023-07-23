#### Structure groups: `--foo --foo-1 ARG1 --foo-2 ARG2 --foo-3 ARG3`

Groups of options that can be specified multiple times. All such groups should be kept without
overwriting previous one.

```console
 $ prometheus_sensors_exporter \
     \
     `# 2 physical sensors located on physycial different i2c bus or address` \
     --sensor \
         --sensor-device=tmp102 \
         --sensor-name="temperature_tmp102_outdoor" \
         --sensor-i2c-bus=0 \
         --sensor-i2c-address=0x48 \
     --sensor \
         --sensor-device=tmp102 \
         --sensor-name="temperature_tmp102_indoor" \
         --sensor-i2c-bus=1 \
         --sensor-i2c-address=0x49 \
```

#![cfg_attr(not(doctest), doc = include_str!("docs/adjacent_1.md"))]
