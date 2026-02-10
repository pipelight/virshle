+++
date = 2025-09-11
updated = 2026-02-10

weight = 10

title = "Installation (Arch, Debian...)"

description = """

"""

draft=false
+++

# Installation (Arch, Debian...)

Install on a distribution that is Filesystem hierarchy standard (FHS) compliant
like **Arch** and **Debian**.

{% container(type="danger") %}

**Work in progress...**

However, some parts of the installation remain cumbersome on these distributions
because of the rudimentariness of the existing tooling.

Any help on the creation automated installation scripts is welcome.

{% end %}

{% container(type="info") %}

**Unprivileged user**

If you intend to run Virshle as an unprivileged/non-root user,
you must add this user to the `sudoers` file,
to give it permission to mount and unmount storage devices.

```sh
# /etc/sudores
<username> ALL=(ALL:ALL) NOPASSWD: /usr/bin/mount, /usr/bin/umount
```

{% end %}

## Dependencies overview.

Install dependencies from your favourite package manager or from source.

Mandatory dependencies:

- [cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
  (alias ch), the level 1 hypervisor.
- [openvswitch](https://github.com/openvswitch/ovs), the packet switching software.

Optional dependencies:

- [KeaDHCP](https://kea.readthedocs.io/en/latest/), the DHCP server.

### Virtualization: cloud-hypervisor configuration

Copy or symlink the firmware files for direct kernel boot.
Must be available at
`/run/cloud-hypervisor/hypervisor-fw`
or
`/run/cloud-hypervisor/CLOUDVH.fd`

```sh
wget https://github.com/cloud-hypervisor/cloud-hypervisor/releases/latest/download/hypervisor-fw
sudo mv hypervisor-fw /var/run/openvswitch/hypervisor-fw
```

## Network dependencies

### Openvswitch (mandatory)

{% container(type="info") %}

**Unprivileged user**

When Virshle is being run as non-root,
Openvswitch files needs broader permissions in order for Virshle to
interact with it and create network interfaces.

```sh
# Open database permission
# so that a non root user can manipulate the host network.
chown root:users /var/run/openvswitch
chmod -R 774 /var/run/openvswitch`
```

{% end %}

### KeaDHCP (optional)

### RA (optional)

For Ipv6 support.
RA send router announcements for VM address automatic configuration.

## Virshle configuration

Install the binary from source with cargo.

```sh
cargo install --git https://github.com/pipelight/virshle
```

{% container(type="info") %}

**Unprivileged user**

When being run as non-root,
Virshle needs explicit **network capabilities** in order to create network interfaces.

```sh
setcap cap_net_admin
```

{% end %}

The following command bootstraps the node,

```sh
virshle node init --net --db --config
```

It mainly creates required files under `/var/lib/virshle`, creates a default configuration at `/etc/virshle/config.toml`,
and makes the best effort to modify the host network configuration.

You can try and run the virshle node by hand.

```sh
virshle node serve -vvv
```

## Create the daemon.

Create a default systemd unit like the following,
to start a Virshle node in the background on server boot.

```sh
# /etc/systemd/system/virshle.service

[Unit]
Description=Virshle node daemon (type 2 hypervisor)
Documentation=https://github.com/pipelight/virshle

After=network.target socket.target ovs-vswitchd.service ovsdb.service

[Service]
Type=simple
ExecStart=/bin/env virshle node serve -vvvv
ExecStartPre=/bin/env -virshle node init --all -vvvv

Group=wheel
User=root
WorkingDirectory=/var/lib/virshle

Environment=PATH=/usr/bin/env
# If you want "~" to expend as another home.
# Environment=HOME=/home/anon

StandardInput=null
StandardError=journal+console
StandardOutput=journal+console

AmbientCapabilities=CAP_SYS_ADMIN
AmbientCapabilities=CAP_NET_ADMIN

[Install]
WantedBy=multi-user.target
```
