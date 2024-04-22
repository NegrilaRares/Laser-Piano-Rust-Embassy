# Laser Piano

## Description

<!-- Describe in a few words your project idea. -->
A piano made using laser diodes and laser receivers.


It uses a potentiometer with leds to adjust and show volume, 3 separete switches with leds to displat change in octave.


The plan is to take the sound data from a microsd card and give it to a dac that will (after going thorough a potentiometer) give the analog output to a speaker.
But there may be a need to amplify the signal in which case i will use a LM386 module from which i will swap the variable resistor with a potentiometer.
## Hardware

<!-- Fill out this table with all the hardware components that you mght need.

The format is 
```
| [Device](link://to/device) | This is used ... | [price](link://to/store) |

```

-->

| Device | Usage | Price |
|--------|--------|-------|
| [Rapspberry Pi Pico W](https://www.raspberrypi.com/documentation/microcontrollers/raspberry-pi-pico.html) | The microcontroller | [35 RON](https://www.optimusdigital.ro/en/raspberry-pi-boards/12394-raspberry-pi-pico-w.html) |
| 13 * Led | This is used for displaying current settings | [-] |
| 13 * 220 Ohm R | This is used to make sure the leds don't get fried | [-] |
| 10 * 10 kOhm R | This is used to crate pull up and pull down resistors | [-] |
| 7 * 0.1 uF Capacitor | This is used to debounce the signal from the laser receivers | [-] |
| lots * cables | This is used to connect diffrent things on the breadboard | [-] |
| 10k Potentiometer | This is used for volume input | [2 RON] |
| 2 * [Shift Register]([link://to/device](https://www.diodes.com/assets/Datasheets/74HC595.pdf)) | This is used for the output of the volume setting | [2 RON](https://www.optimusdigital.ro/en/others/2448-registru-de-deplasare-74hc595-dip-16.html) |
| Laser Diode | This is used to generate a laser | [2.5 RON]([link://to/store](https://ardushop.ro/ro/electronica/262-modul-dioda-laser-rou-5mw.html)) |
| [ISO203]([link://to/device](https://forum.arduino.cc/t/documents-about-laser-sensor-ds18b20/1090450/5)) | This is used to monitor for interference in the path of the laser | [6.5 RON](https://www.optimusdigital.ro/en/others/3289-laser-diode-receiver.html) |
| Catalex MicroSd card slot | This is used to read the data from a microsd | [4.4 RON]([link://to/store](https://www.optimusdigital.ro/en/memories/1516-microsd-card-slot-module.html)) |
| [DAC MCP4725]([link://to/device](https://ww1.microchip.com/downloads/en/devicedoc/22039d.pdf)) | This is used to transfrom the binary from the audio file data into an analog singal for the speaker | [25 RON]([link://to/store](https://www.optimusdigital.ro/en/others/1327-dac-mcp4725-module-with-i2c-interface.html?search_query=dac&results=62)) |
| Speaker | This is used to create a sound based on the analog input given by the system | [-] |
| [AMP LM386]([link://to/device](https://www.ti.com/lit/ds/symlink/lm386.pdf)) | This is used amplify the analog singal | [6.5 RON]([link://to/store](https://ardushop.ro/ro/electronica/241-modul-amplificator-audio-lm386.html)) |

## Hardware Design (work in progress)
[kicad] In the Project-KiCad Folder


## Links

<!-- Add a few links that got you the idea and that you think you will use for your project -->

1. [MicroSd Crate](https://github.com/rust-embedded-community/embedded-sdmmc-rs)
2. [link](https://example3.com)
...
