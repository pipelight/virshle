# Virshle ROADMAP

Virshle is compiled as a unique binary that is:

- a cli that remotely control nodes,
- a server(node) with a http rest API.

## Improvements for next versions

- [ ]: Virshle node daemon:

  - [x]: add socket that takes http and http over ssh req.
  - [x]: replace logs by tracing.
  - [ ]: improve event tracing.

- [ ]: Configuration files:

  - [x]: Toml config files.
  - [ ]: KDL config files.
  - [ ]: Add section for node maximum resources saturation.

## Features

- [ ]: Networking

  - [x]: Get ip leases with kea dhcp.
  - [ ]: Get ip leases with dora dhcp.

  - [x]: Vm with bridge and tab device.
  - [ ]: Vm with macvtap (deprecated because of cloud-hypervisor deprecation).
