+++
title = "Install"
template = "docgen.html"

[extra]
toc = true
+++

> At this time, the installation process expects you to have a rooted thermostat
> and familiarity with the Linux command line.

## Get root

You'll need a rooted Nest with SSH access. Any of the following methods should work.

* [NestDFUAttack](https://github.com/exploiteers/NestDFUAttack)
* [Cuckoo Loader](https://github.com/cuckoo-nest/cuckoo_loader)
* [NoLongerEvil](https://nolongerevil.com/)

## Download & Install

I'll assume retherm will be placed under `/retherm/`, but choose whatever
directory you like.

Go to the [Releases](https://github.com/jiggak/retherm/releases) page on Github
and copy the latest download link.

If you prefer to build ReTherm yourself, details are in the project
[README](https://github.com/jiggak/retherm).

1. Make directory for ReTherm if it doesn't already exist
   ```bash
   mkdir /retherm
   ```
2. Download latest build
   ```bash
   # Replace download link with one copied from releases page.
   # Make sure retherm is stopped before replacing (when updating, not installing first time).
   curl -o /retherm/retherm https://github.com/jiggak/retherm/releases/download/v1.0.0/retherm
   chmod +x /retherm/retherm
   ```
3. Create `/etc/init.d/retherm` with contents of [init.sh](https://github.com/jiggak/retherm/blob/main/init.sh)
   ```bash
   curl -o /etc/init.d/retherm https://raw.githubusercontent.com/jiggak/retherm/refs/heads/main/init.sh
   chmod +x /etc/init.d/retherm
   ```
4. Stop Nest app `/etc/init.d/nestlabs stop`
5. Start ReTherm `/etc/init.d/retherm start`

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
