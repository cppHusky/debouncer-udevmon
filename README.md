# Debouncer

This program is a middle layer that serves [interception tools](https://gitlab.com/interception/linux/tools) for debouncing.

When you use it, you can delay the keyboard "release" event for some time, which is similar to the ["spuious" mode of libinput](https://wayland.freedesktop.org/libinput/doc/latest/button-debouncing.html).

## Building

```bash
$ make # for release, or
$ make debug # for debug
$ sudo udevmon -c udevmon.yaml
```

- When you use the debug version, `./debounce` will create a log file named `keyboard-debouncer.log` in current directory.
- the `DEVICE/LINK` in `udevmon.yaml` maybe editted by yourself to meet your requirements.
- `udevmon` needs root previleges.
