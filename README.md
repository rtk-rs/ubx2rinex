UBX2RINEX
=========

[![Rust](https://github.com/rtk-rs/ubx2rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/rtk-rs/ubx2rinex/actions/workflows/rust.yml)
[![Rust](https://github.com/rtk-rs/ubx2rinex/actions/workflows/daily.yml/badge.svg)](https://github.com/rtk-rs/ubx2rinex/actions/workflows/daily.yml)
[![crates.io](https://img.shields.io/crates/v/ubx2rinex.svg)](https://crates.io/crates/ubx2rinex)

[![License](https://img.shields.io/badge/license-MPL_2.0-orange?style=for-the-badge&logo=mozilla)](https://github.com/rtk-rs/ubx2rinex/blob/main/LICENSE)

`ubx2rinex` is a small program serves a few purposes:

- collect GNSS measurements and wrap them as Observation RINEX files (active by default,
turned off with `--no-obs`)
- fulfill the complex task of NAV RINEX collection (disabled by default, requested with `--nav`)

Licensing
=========

This application is part of the [RTK-rs framework](https://github.com/rtk-rs) which
is delivered under the [Mozilla V2 Public](https://www.mozilla.org/en-US/MPL/2.0) license.

Ecosystem
=========

This application is the combination of:

- the great [ublox-rs](https://github.com/ublox-rs/ublox) library
- our [RINEX library](https://github.com/rtk-rs/rinex)
- the great [Hifitime library](https://github.com/nyx-space/hifitime) by nyx-space

Install from Cargo
==================

You can install the tool directly from Cargo with internet access:

```bash
cargo install ubx2rinex
```

Build from sources
==================

Download the version you are interested in:

```bash
git clone https://github.com/rtk-rs/ubx2rinex
```

And build it using cargo:

```bash
cargo build -r
```

The application uses the latest `UBX` protocol supported. This may unlock full potential
of modern devices, and does not cause issues with older firmwares, simply restricted applications.

Application logs
================

`ubx2rinex` uses the Rust logger for tracing events in real-time and not disturb the collection process.  
To activate the application logs, simply define the `$RUST_LOG` variable:

```bash
export RUST_LOG=info
```

Several sensitivity options exist:

- info
- error
- debug
- trace

RINEX Files collection
======================

`ubx2rinex` is a collecter, in the sense that it collects real-time data and is forms
a snapshot. It is dedicated to U-Blox receivers, thanks to the [great ublox-rs library](https://github.com/ublox-rs/ublox). It is currently dedicated to either Observation and NAV RINEX formats, both may be collected
at the same time. It is not clear as of today, whether other RINEX format may apply here. If so,
we may unlock more in the future.

Since we're collecting a snapshot, the snapshot (or release period) needs to be defined. 
In RINEX terminology, it is the PPU which is indicated in file names that follow the V3 standard.  

Note that, by default, this tool prefers V2 (shorter) file names, for the simple reason
that V3 file names require more information to be complete. You can design a valid V3 file name
using our command line.

Serial port settings
====================

Serial port settings are always required, and defined with

- `-p, --port`: to define the serial port
- `-b, --baud`: to define the baud rate, which can be omitted because `115_200` is the default value.

To determine your U-Blox port on linux, for example:

```bash
dmesg | tail -n 20
```

<img src="docs/ports-listing.png" alt="Serial Port listing" width="500" />

Observation RINEX collection
============================

Observation RINEX collection is the default mode. It can only be disabled whenever `--no-obs` is asserted.  
In this mode, the U-Blox serves as a signal source that we can snapshot to RINEX format.

This tool supports V2, V3 and V4 RINEX revisions. Selecting a revision applies to all RINEX formats,
so if you select V4 NAV collection, it will also apply to your signal collection, in case both are active.

To deploy you must activate at least one constellation, one carrier signal and select one observable.

- We propose one flag per constellation (for example `--gps`)
- We propose one flag per signal (for example `--l1`) 
- And we have one flag per observable, and `--all-meas` that activates all of them)

This example would collect C1, L1 and D1 observations, for GPS:

```C
RUST_LOG=debug ubx2rinex -p /dev/ttyUSB1 --gps --l1 --allmeas

[2025-02-23T10:48:22Z INFO  ubx2rinex] Connected to U-Blox
[2025-02-25T20:30:01Z DEBUG ubx2rinex::device] U-Blox Software version: EXT CORE 3.01 (111141)
[2025-02-25T20:30:01Z DEBUG ubx2rinex::device] U-Blox Firmware version: 00080000
[2025-02-25T20:30:01Z INFO  ubx2rinex::device] Enabled constellations: GPS, Glonass, 
[2025-02-25T20:30:01Z INFO  ubx2rinex::device] Supported constellations: GPS, Galileo, BeiDou, Glonass, 
[2025-02-25T20:30:01Z DEBUG ubx2rinex] UBX-NAV-EOE enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] UBX-NAV-PVT enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] UBX-NAV-CLK enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] UBX-NAV-PVT enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] Measurement rate is 30 s (Gps)
[2025-02-25T20:30:03Z INFO  ubx2rinex] Observation RINEX mode deployed
[2025-02-25T20:30:03Z INFO  ubx2rinex] 2025-02-25T20:30:18.313672551 GPST - program deployed
[2025-02-25T20:30:12Z TRACE ubx2rinex::collecter] 2025-02-25T20:30:30.006000000 GPST - (G02 RAWX) - pr=2.2654484E7 cp=1.1905012E8 dop=-1.1215144E3 cno=25
[2025-02-25T20:30:12Z TRACE ubx2rinex::collecter] 2025-02-25T20:30:30.006000000 GPST - (G03 RAWX) - pr=2.1846652E7 cp=1.1480491E8 dop=6.1150366E2 cno=37
[2025-02-25T20:30:12Z TRACE ubx2rinex::collecter] 2025-02-25T20:30:30.006000000 GPST - (R19 RAWX) - pr=2.4613278E7 cp=1.3166441E8 dop=-3.8444370E3 cno=23
[2025-02-25T20:30:12Z DEBUG ubx2rinex] NAV PVT: NavPvt { itow: 246630000, year: 2025, month: 2, day: 25, hour: 20, min: 30, sec: 12, valid: 55, time_accuracy: 50, nanosecond: 371019, fix_type: Fix3D, flags: NavPvtFlags(GPS_FIX_OK), flags2: NavPvtFlags2(RESERVED1 | RESERVED3 | CONFIRMED_AVAI | CONFIRMED_DATE | CONFIRMED_TIME), num_satellites: 8, lon: 4.635316899999999, lat: 43.6876077, height_meters: 65.992, height_msl: 18.155, horiz_accuracy: 21079, vert_accuracy: 24241, vel_north: 0.081, vel_east: 0.092, vel_down: -0.129, ground_speed: 0.123, heading: 0.0, speed_accuracy_estimate: 0.20400000000000001, heading_accuracy_estimate: 162.7699, pdop: 204, reserved1: [0, 0, 74, 100, 35, 0], heading_of_vehicle: 0.0, magnetic_declination: 0.0, magnetic_declination_accuracy: 0.0 }
[2025-02-25T20:30:12Z DEBUG ubx2rinex] nav-sat: NavSat { itow: 246630000, version: 1, num_svs: 21, reserved: [0, 0], [...] }
[2025-02-25T20:30:12Z DEBUG ubx2rinex] NAV CLK: NavClock { itow: 246630000, clk_b: 628984, clk_d: 187, t_acc: 50, f_acc: 736 }

[..]
```

In parallel, right after this:

```C
cat UBXR132.25O

───────┬──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
       │ File: UBXR132.25O
───────┼──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   1   │      3.00           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE
   2   │ rs-rinex v0.17.1                                            PGM / RUN BY / DATE
   3   │ G    3 D1C L1C C1C                                          SYS / # / OBS TYPES
   4   │ R    3 D1C L1C C1C                                          SYS / # / OBS TYPES
   5   │                                                             END OF HEADER
───────┴────────────────────────────────────────────────────────────────────────────────────────────────────────
```

Note that, conveniently, `ubx2rinex` does not wait for the snapshot period to dump the data.  
This means you can have an external listener / file watcher doing its job at the same time.
But it can only work with read-only access. Because `ubx2rinex` owns write access, until
the file is fully published (end of snapshot period).

The file header is released when that is feasible. 

The first observations start to appear according to your sampling settings.
In parallel, after at last 30s in this example:


```C
cat UBXR132.25O
───────┬──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
       │ File: UBXR132.25O
───────┼──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
   1   │      3.00           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE
   2   │ rs-rinex v0.17.1                                            PGM / RUN BY / DATE
   3   │ G    3 D1C L1C C1C                                          SYS / # / OBS TYPES
   4   │ R    3 D1C L1C C1C                                          SYS / # / OBS TYPES
   5   │                                                             END OF HEADER
   6   │ > 2025 05 12 18 20 30.0050000  0 17
   7   │ G03     -2915.745    24088368.360   126585229.731
   8   │ G04     -1809.767    22078563.674   116023565.200
   9   │ G06      1145.509    22585834.349   118689349.627
  10   │ G07      2889.160    24996256.806   131356267.067
  11   │ G09       446.475    21511342.968   113042908.415
  12   │ G11      2890.540    24962430.711   131178458.310
  13   │ G17     -3228.178    25694724.299   135026691.165
  14   │ G19     -2854.094    24223747.524   127296646.638
  15   │ G31     -3328.472    26354976.610   138496336.727
```

At the end of the snapshot period, `ubx2rinex` moves on to the new file descriptor:

```C
TODO EXAMPLE
```

Clock state collection
======================

Observation RINEX allows the description of the sampling clock state.  
Note that it is not active by default. You can activate it with `--rx-clock`. 
In this case, each Epoch starts with the local cloc state, as per the RINEX standards.

```C
RUST_LOG=debug ubx2rinex \
        -p /dev/ttyACM0 \
        -b 115200 \
        --gps \
        --l1 \
        --all-meas \
        --rx-clock \
        -m "M8T u-Blox"
 
[..] in parallel, after 1 min with those settings (at least 2x 30s captures)
```

Observations Timescale
======================

The clock state and all observations are expressed in a single Timescale, as per the
RINEX standards. Yet you can select it with `--timescale`. The default being GPST,
all previous examples expressed measurements in GPST and produced RINEX files referenced to GPST.

:wanring: The RINEX standards specifies that the timescale should be GNSS, but this toolbox
allows you to select any of the supported timescales. So it is technically possible
to generate a UTC RINEX, which does not follow the standards. Other tools of our toolbox can work
around it and still accept it, for the same reasons. But it is unlikely that other toolboxes will,
so you have to be careful.

:warning: The timescale settings does not follow the constellation settings! 
Let's say you're only interested in BDS measurements, our default timescale is still GPST
(which is the most common case!). You can then specify to express your BDS measurements in BDT with the following:

```C
RUST_LOG=debug ubx2rinex \
    -p /dev/ttyACM0 \
    -b 115200 \
    --bds \
    --l1 \
    --all-meas \
    --rx-clock \
    --timescale BDT \
    -m "M8T u-Blox"
```

Note that the rest of our framework understands timescale correctly, in particular, it is possible to
post-process any measurements correctly, whether it is for QC analysis or post processed navigation.

Sampling period
===============

The default sampling period is `30s` for standardized "low-rate" observation RINEX files.  
You can modify that with `-s, --sampling`. In this example, we "upgrade" to higher rate (01S) RINEX:

```C
ubx2rinex -p /dev/ttyACM0 \
          --gps \
          --l1 \
          --all-meas \
          -s "1 s"
```

Note that U-Blox is limited to 50ms.

Snapshot period
===============

The snapshot period defines how often we release a RINEX of each kind.
When the snapshot is being released, the file handled is released and the file is ready to be distributed or post processed.

By default, the snapshot period is set to Daily, which is compatible with standard RINEX publications.  

Several options exist (you can only select one at once):

- `--hourly` for Hourly RINEX publication
- `--quarterly` for one publication every 6 hours
- `--noon` for one publication every 12 hours
- `--custom $dt` for custom publication period. Every valid `Duration` description may apply. For example, these are all valid durations: `--period  

NB: 

- the first signal observation is released everyday at midnight 00:00:00 in the main Timescale
- the last signal observation is released everyday at 23:59:30 in the main Timescale

File rotation and descriptors
=============================

`ubx2rinex` owns the File descriptor with write access (only) until the end of this period.  
That means you can only read the file until the end of the period.
Deleting the file descriptor while the program is actively collecting will the program to panic.

At the end of each period, the file descriptor is released and you can fully process it.   
The file pointer is then incremented, using standard naming conventions.

`ubx2rinex` will provide content "as soon as" it exists (+/- some file descriptor access, that we
try to keep efficient). This means that exploitation of this program is compatible with real-time
watching of the file being produced and each new symbol is published fairly quickly.


## :warning: M8 Series usage

:warning: Until further notice :warning:

This application is compatible with M8 series device but does not offer
any option (as of today) to reprogram the Constellation / Signal settings. You will have
to use a third party tool to actually reconfigure your device. 

For example, M8T factory settings are GPS (L1) and GLO (L1).
We would deploy like this:

```bash
ubx2rinex -p /dev/ttyUSB1 --gps --glonass --l1
```

Any other constellation flags has no effect. Selecting other signals has no effect.
Removing L1 signal would create invalid RINEX.

U-Blox configuration
====================

U-Blox receivers are very user friendly yet still require a little knowledge to operate.  
That is especially true to advanced use cases. 

The most basic configuration your need to understand, is how to parametrize the streaming options
of your U-Blox device. `ubx2rinex` allows partial reconfiguration of the U-Blox receiver:

(1) Define the streaming interface(s) and options
(2) Customize the receiver for this application's need

(1): means you can actually use `ubx2rinex` to parametrize how your U-Blox streams.
It is also necessary to activate streaming on at least the USB/UART port that you intend to use.

(2): configuring the receiver, in particular what frames it will transmit, will modify the RINEX content
we are able to collect obviously.

## USB/UART port setup

TODO

## UBX streaming setup

TODO

Observation RINEX collection is the default mode and deploys at all-times, unless you
use the `--no-obs` flag, which will disable this mode: 

```bash
./target/release/ubx2rinex \
        -p /dev/ttyACM0 \
        -b 115200 \
        --gps \
        --l1 \
        --rx-clock \
        -m "M8T u-Blox"

[2025-02-25T20:30:01Z DEBUG ubx2rinex::device] U-Blox Software version: EXT CORE 3.01 (111141)
[2025-02-25T20:30:01Z DEBUG ubx2rinex::device] U-Blox Firmware version: 00080000
[2025-02-25T20:30:01Z INFO  ubx2rinex::device] Enabled constellations: GPS, Glonass, 
[2025-02-25T20:30:01Z INFO  ubx2rinex::device] Supported constellations: GPS, Galileo, BeiDou, Glonass, 
[2025-02-25T20:30:01Z DEBUG ubx2rinex] UBX-NAV-EOE enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] UBX-NAV-PVT enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] UBX-NAV-CLK enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] UBX-NAV-PVT enabled
[2025-02-25T20:30:02Z DEBUG ubx2rinex] Measurement rate is 30 s (Gps)
[2025-02-25T20:30:03Z INFO  ubx2rinex] Observation RINEX mode deployed
[2025-02-25T20:30:03Z INFO  ubx2rinex] 2025-02-25T20:30:18.313672551 GPST - program deployed
[2025-02-25T20:30:12Z TRACE ubx2rinex::collecter] 2025-02-25T20:30:30.006000000 GPST - (G02 RAWX) - pr=2.2654484E7 cp=1.1905012E8 dop=-1.1215144E3 cno=25
[2025-02-25T20:30:12Z TRACE ubx2rinex::collecter] 2025-02-25T20:30:30.006000000 GPST - (G03 RAWX) - pr=2.1846652E7 cp=1.1480491E8 dop=6.1150366E2 cno=37
[2025-02-25T20:30:12Z TRACE ubx2rinex::collecter] 2025-02-25T20:30:30.006000000 GPST - (R19 RAWX) - pr=2.4613278E7 cp=1.3166441E8 dop=-3.8444370E3 cno=23
[2025-02-25T20:30:12Z DEBUG ubx2rinex] NAV PVT: NavPvt { itow: 246630000, year: 2025, month: 2, day: 25, hour: 20, min: 30, sec: 12, valid: 55, time_accuracy: 50, nanosecond: 371019, fix_type: Fix3D, flags: NavPvtFlags(GPS_FIX_OK), flags2: NavPvtFlags2(RESERVED1 | RESERVED3 | CONFIRMED_AVAI | CONFIRMED_DATE | CONFIRMED_TIME), num_satellites: 8, lon: 4.635316899999999, lat: 43.6876077, height_meters: 65.992, height_msl: 18.155, horiz_accuracy: 21079, vert_accuracy: 24241, vel_north: 0.081, vel_east: 0.092, vel_down: -0.129, ground_speed: 0.123, heading: 0.0, speed_accuracy_estimate: 0.20400000000000001, heading_accuracy_estimate: 162.7699, pdop: 204, reserved1: [0, 0, 74, 100, 35, 0], heading_of_vehicle: 0.0, magnetic_declination: 0.0, magnetic_declination_accuracy: 0.0 }
[2025-02-25T20:30:12Z DEBUG ubx2rinex] nav-sat: NavSat { itow: 246630000, version: 1, num_svs: 21, reserved: [0, 0], [...] }
[2025-02-25T20:30:12Z DEBUG ubx2rinex] NAV CLK: NavClock { itow: 246630000, clk_b: 628984, clk_d: 187, t_acc: 50, f_acc: 736 }
[...]
```

Program interruption and release
================================

`ubx2rinex` does not support Ctrl+C interruption cleanly as of today.

Other customizations
====================

- Define your name as `Operator`` in RINEX terminology,
with `--operator myself`
- Define your name as `Observer`` in RINEX terminology,
with `--observer myself`
- Define your agency (publisher) with `--agency myagency`
- Define the country code (3 letter) of your agency with `--country ABC`

no-std
======

This program relies on both the `ubx` parser and the `rinex` library.  
The first one supports `no-std`, but it is unfortunately not true for the latter.  
We will see if we can provide some very reduced, `no-std` compatible portions of the `rinex` library
in the future, especially the file production side. 
This is not scheduled work as of today. Feel free to join in if you want to see this happen sooner.
