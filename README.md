A simple tool suitable for adjusting external monitor's brightness.

Much faster than [ddcutil](https://github.com/rockowitz/ddcutil), still faster than [ddcset](https://github.com/arcnmx/ddcset-rs) which enumerates all monitors. And it matches monitor by the output name you see from xrandr / wayland-info output.

```
monitor-control 0.1.0
lilydjwg <lilydjwg@gmail.com>
The fastest way to get / set DDC values for a monitor

USAGE:
    monitor-control <OUTPUT_NAME> <FEATURE_CODE> [FEATURE_VALUE]

ARGS:
    <OUTPUT_NAME>      
    <FEATURE_CODE>     
    <FEATURE_VALUE>    

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
```

E.g.

```sh
# get current and max brightness value
monitor-control DP-2 16
# set brightness to 50
monitor-control DP-2 16 50
```

<small>Scripts to determine which monitor to adjust and show indicators like [wob](https://github.com/francma/wob) are left to other projects.</small>
