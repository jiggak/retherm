+++
title = "Install"
template = "docgen.html"

[extra]
toc = true
+++

> At this time, the installation process is not exactly user friendly.
> You will need a rooted Nest, Rust toolchain, Arm toolchain, and some comfort
> with the Linux command line.
> I have plans to at least provide builds for download from
> [Releases](https://github.com/jiggak/retherm/releases) on Github.

## Get root

You'll need a rooted Nest with SSH access. Any of the following methods should work.

* [NestDFUAttack](https://github.com/exploiteers/NestDFUAttack)
* [Cuckoo Loader](https://github.com/cuckoo-nest/cuckoo_loader)
* [NoLongerEvil](https://nolongerevil.com/)

## Build & Install

See the project [README](https://github.com/jiggak/retherm) for details about
building for the Nest.

I'll assume retherm will be placed under `/retherm/`, but choose whatever
directory you like.

1. Copy retherm to `/retherm/retherm`
2. Create `/etc/init.d/retherm` with contents of [init.sh](https://github.com/jiggak/retherm/blob/main/init.sh)
3. Stop Nest app `/etc/init.d/nestlabs stop`
4. Start ReTherm `/etc/init.d/retherm start`

ReTherm will run until the device reboots; the default Nest app will start the
next time the device reboots.

> Currently ReTherm doesn't have an interface for configuring Wifi.
> It's crutial the Nest app starts at boot so that you maintain some means
> to set network settings (i.e. SSH access).

## Logging to syslogd

Optionally, ReTherm can output logs to `syslogd`. Log messages will be written
to `/var/log/messages`.

```console
# INFO can be one of: TRACE, DEBUG, INFO, ERROR
retherm --syslog INFO
```

This opens up the option to forwarding log messages to a log collector by appending
`-R 192.168.1.42:514 -L` to `/etc/syslogd.options` where "192.168.1.42" is the
address of your log server.

```
-O /var/log/messages -s 384 -b 15 -u -R 192.168.1.42:514 -L
```
