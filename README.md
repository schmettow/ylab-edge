# YLab Edge

... is a framework for high performance hetereogeneous sensor arrays on common microcontrollers. The goal is to provide a high-level API making it easy to create for example a recording system for movement science that simultaneously takes measures from  arrays of EMG and acceleration sensors. The high performace is achieved by consequently using a concurrency programming (async/await).

The architecture is primarily task based. All sensors, as well as all input and output devices can be added to the play by simply adding a task main(). All communication between tasks is transferred via thread-safe channels.

# Supported MCUs

+ RP2040
+ STM32 FF46Zet (soon)

# Supported sensors

+ analog sensors (ECG, EEG etc)
+ several acceleration/gyroscope sensors
+ several air quality sensors (CO2, VOC, fine dust etc.)
+ moment-of-interest

# User interface

While YLab Edge is very capable as a plug-and-play device, it also supports interaction programming. Currently supported are Led, RGB and Ssd1306 displays for output and debounced buttons for input. A control flow is easy to develop as triggered state transitions using `enum` and `match`.

# Credits
+ Embassy developers
+ Embedded_hal developers
+ The consequent use of concurrent IO made it necessary to fork and adjust several existing sensor driver crates.

# TODO
+ port STM32-F466Zet
+ port more I2C drivers
+ port Ads1299 (HackEEG)
