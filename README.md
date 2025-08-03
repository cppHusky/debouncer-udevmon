# Debouncer-Udevmon

This program is a middle layer that serves [interception tools](https://gitlab.com/interception/linux/tools) for debouncing.

When you use it, you can delay the keyboard "release" event for some time, which is similar to the ["spuious" mode of libinput](https://wayland.freedesktop.org/libinput/doc/latest/button-debouncing.html).

## Building and Using

```bash
$ make # for release, or
$ make debug # for debug
```
When you use the debug version, `./debounce` will create a log file named `keyboard-debouncer.log` in current directory.

```bash
$ sudo udevmon -c udevmon.yaml
```

the `DEVICE/LINK` in `udevmon.yaml` maybe edited by yourself to meet your requirements.

 `udevmon` needs root previleges.
