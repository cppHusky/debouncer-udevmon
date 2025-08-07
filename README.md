# Debouncer (with udevmon)

This program is a middleware layer that integrated into [interception tools](https://gitlab.com/interception/linux/tools) for debouncing.

I've been troubled by keyboard chattering for months. It's hard for me to replace the keyboard or fix hardware problems, so I tried software debouncing.

I implemented this middleware layer. It can create a virtual device that receives information from previous hardware layer and provide input.

This middleware is flexible, loosely coupled and easy-to-use. After bringing it into use, my problem is resolved successfully.

## How it works

> See also: [Interception Tools#how it works](https://gitlab.com/interception/linux/tools#how-it-works)

### Overview

The `intercept`, `debouncer` and `uinput` form a whole virtual device. The data flow is in this middleware is shown as follows:

```
(keyboard device) -> intercept -> debouncer -> uinput -> (next layer)
```

+ `intercept` captures your input from previous layer, and write raw input to `stdout`;
+ `debouncer` get the raw input data from `stdin`, process them and write back to `stdout`;
+ `uinput` convert the raw input from last step, and write it to this virtual device (can be found as `/dev/input/eventX`);

The "next layer" can be evtest, X server or Wayland compositor.

### Key event and debouncing

See [input.h](https://github.com/torvalds/linux/blob/master/include/uapi/linux/input.h#L28-L47) in Linux kernel. Each input behaviour is represented by a bunch of `input_event` object. For example, if you press a key on your machine, there would be 3 [input events](https://docs.kernel.org/input/event-codes.html): a miscellaneous input data, *a key event*, and a sync event.

A key event has 3 possible values: 1 (pressed), 0 (released) or 2 (autorepeat).

The role of `debouncer` is that, it can delay the keyboard "release" event for some time (+12ms is ideal for my machine), which is similar to the ["spuious" mode of libinput](https://wayland.freedesktop.org/libinput/doc/latest/button-debouncing.html).

Once `debouncer` received a "release" event, it will wait for some time. During this time, if no "press" event of the same key comes, it will write the "release" event to `stdout`; otherwise, it will neglect this event.

## How to use

### Build on your machine

**You need the following build dependencies**:

- [Rust development tools](https://www.rust-lang.org/learn/get-started)

```bash
$ cargo b # for debug version
$ cargo b -r # for release version
```

The debug version will write more detailed info to `stderr`.

### Directly use on your machine

**You need the following runtime dependencies**:

- [interception tools](https://gitlab.com/interception/linux/tools)

Before using it, you would better create a yaml file as config. For example, here is my `udevmon.yaml`:

```yaml
- JOB: intercept -g $DEVNODE | /usr/local/bin/debouncer | uinput -d $DEVNODE
  DEVICE:
    LINK: "/dev/input/by-path/platform-i8042-serio-0-event-kbd"
```

Then run `udevmon` as root loading your config.

```bash
$ sudo udevmon -c udevmon.yaml
```

If you look into `/dev/input` you will found a new virtual device.

### Use it with systemd

I don't want to run `udevmon` every time I start my machine. I want to use it as an autostart daemon.

Just place the yaml file in `/etc/interception/`, and enable the `udevmon.service`.

```bash
$ sudo systemctl enable udevmon.service --now
```

### Integrated with keyd

[Keyd](https://github.com/rvaiya/keyd) is a key remapping daemon for linux. I use it to remap my keyboard.

This debouncer is easy to integrated with keyd, because they are both modularized middleware layer in libevdev.

Just run `debouncer` first and `keyd` then, you will get a debounced and remapped keyboard! the data flow from keyboard to applications is:

```
(keyboard device) -> debouncer -> keyd -> (next layer)
```

You can also integrate `udevmon.service` and `keyd.service`. But notice that **the order and time of services matter**. `keyd` needs to start after `udevmon` starts, and `udevmon` needs a few microseconds to be ready. So you should [create a drop-in configuration file](https://wiki.archlinux.org/title/Systemd#Drop-in_configuration_files) that wants `udevmon` as optional dependency and sleep for some time before executing the command.

Here is my drop-in config:

```conf
[Unit]
Wants=udevmon.service
After=udevmon.service
[Service]
ExecStartPre=/usr/bin/sleep .7 # 700ms is fine on my machine
```
