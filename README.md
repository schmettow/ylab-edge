# YLab Edge

... is a framework for high performance hetereogeneous sensor arrays on common microcontrollers. Many researchers need just a reliable recording device, sending a data stream to a computer. At the same time, there is much desire for particular combinations of sensors. 

The goal of YLab Edge is to provide a high-level API making it easy to create firmwares for many combinations of MCUs and sensor types. For example, a researcher in Movement Sciences may want to collect data from an array of EMG probes and a second array with motion sensors. YLab Edge alrady supports several sensor types and by using a shared bus architecture, allows to use use several sensors on one wire string.

The high performace is achieved by consequently using a concurrency programming approach (async/await). For this purpose, several sensor driver crates have been forked and given async traits. While this is complex, it will be hidden from users of the framework. On the surface level, programming a YLab Edge system just means to call one task per sensor (array). The ultimate goals is to have a declarative (almost) way to configure the system. Curently, all sensors, input and output devices can be added to the play by simply adding a task to main(), with some boilerplate code. 

```spawner.spawn(adc::task(adc0, p.PIN_26, p.PIN_27, p.PIN_28, 0, 1
```

All communication between tasks is transferred via thread-safe channels. This also makes it easy to use multiple cores, such as on the RP2040.

# Supported MCUs

+ RP2040
+ STM32 F446Zet
+ STM32 H743Zit

# Supported busses

+ 2 x I2C (shared bus)
+ 1 x SPI (shared bus)

## Planned support

+ ESP C6
+ RP3050
+ other common STM32 discovery boards
+ Arduino UNO

# Supported sensors

1. Digital pins for moments-of-interest coding
2. Analog converters
3. 8-channel signal amplifier (ADS1299, HackEEG)
4. 6 axis motion sensor Lsm6ds
5. arrays of up to eight motion sensors using the XCA9548 multiplexer
6. SCD41 CO2 sensor

## Untested

1. ADS1115 6-channel analog converter
2. BMI160 6 axis motion sensor
3. Sensirion SEN5x air quality sensor
4. TLV493d magnetic field sensor

# Output

Currently data is transmitted as CSV stream with time stamp, device ID, sensor ID and up to eight measures.

## Planned output

One bottleneck is the serial output, which currently is 1Mb for RP2040 boards and 2Mb for STM32 boards. In the future an optional binary format will be introduced, which will probably give throughput a factor of 2x to 3x.

Since the data output is an encapsulated task, it is possible to implement alternative data channels, such as Bluetooth or Meshtastic.

# User interface

Every YLab Edge is plug-and-play. At the aame time, YLab Edge is capable of interaction programming. Currently supported are Led, RGB and Ssd1306 displays for output and debounced buttons for input. A control flow is easy to develop as triggered state transitions using `enum` and `match`.

Currently, the RP2040 firmware provides a voice recorder type of interaction, whereas the STM32 firmwares just start running when plugged in.

# Credits

+ Embassy developers
+ Embedded_hal developers
+ The consequent use of concurrent IO made it necessary to fork and adjust several existing sensor driver crates. The made changes were close to trivial and the credits go to the original authors.
+ Frodo Muijser for contributing a galvanically isolated remix of the HackEEG board.
