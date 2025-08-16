# Network configuration.

Virshle tries to lower the amount of black magic needed for network configuration.
Every command needed to create a network interface can be seen in the logs of
`v node serve -vvvv`

Nixos users can find an example of host network configuration at
[`modules/networking.nix`](https://github.com/pipelight/virshle/modules/config.nix).

## MacVTap

The fastest way to add networking connectivity to a VM is with a `macvtap`.
Add the following network configuration to a vm template.

```toml
[[template.vm.net]]
name = "main"
[template.vm.net.type.macvtap]
```

It uses the linux network [ip](https://www.man7.org/linux/man-pages/man8/ip.8.html) command
to create a macvtap interface that will be bound to the VM.

No need for a bridge here, the upstream router gives ips and network access to the VM.

## Bridge and Tap

For a fine-grained control of your network, prefer the tap device.

```toml
[[template.vm.net]]
name = "main"
[template.vm.net.type.tap]
```

It uses the [ip](https://www.man7.org/linux/man-pages/man8/ip.8.html) command and
[openvswitch](https://github.com/openvswitch/ovs)
commands (`ovs-vsctl`) to create a VM dedicated brigde/switch (br0).

Then freshly created VMs are attached to this bridge(br0) via a tap device.

To add outside network connectivity, you need to add your main
interface to the bridge.

Checkout your network configuration with.

`ip l`
`ovs-vsctl show`

More on network: [https://github.com/pipelight/virshle/virshle_core/src/network/README.md]

## DHCP

Virshle relies on external software to manage vm ips.

[KeaDHCP](https://kea.readthedocs.io/en/latest/) is supported.
You need a configured KeaDHCP(v4 or v6 or both) instance running somewhere.

Then add the kea remote control url to your virshle configuration.

```toml
[dhcp]
[dhcp.kea]
url = "tcp://localhost:5547"
```

Dhcp leases managed by KeaDHCP show up when increasing verbosity.

```sh
v vm ls -v
```
