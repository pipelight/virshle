# Network crate

## tldr;

This crate is a set of function for network manipulation.

Everything done programmatically here can be done with `ip` and `ovs-vsctl`.
_I recommend using those tools for troubleshooting._

## Host network autoconfiguration

The aim is to **split the host main network interface**
to add network connectivity to virtual machines.

We basically want to achieve this:

<!---
  ┌───────┐
  │ eno1  │
  ┴───┬───┴
      │
  ┌───┴──┐
  │      │
  │      │
  │ host │
  │      │
  └──────┘
--->

<!---
         ┌───────┐
         │ eno1  │
    ┌────┴───┬───┴────┐
    │        │        │
    │        │        │
┌───┴──┐ ┌───┴──┐ ┌───┴──┐
│      │ │      │ │      │
│      │ │      │ │      │
│ host │ │ vm1  │ │ vm2  │
│      │ │      │ │      │
└──────┘ └──────┘ └──────┘

--->

But Network is **stupid**, and there multiple but no simple way to achieve this, so we have to get a bit creative.

We thus use (virtual) network switches through the `openvswitch`
network management tool.

The main interface `eno1` is plugged into a main switch `vs0` which will

- redirect traffic to the host via a random port (here `vs0p1`)
- redirect traffic to another switch dedicated to VMs (`br0`)

<!---
                                 ┌─────┐
                                 │ eno1│
                                 └──┬──┘
                                    │
                                    │
   ┌───────────────────┐        ┌───┴──┐
   │         br0       ├────────┤  vs0 │
   └───────────────────┘        └──────┘
   │      │     │                   │
   │      │     │                ┌────┐
┌────┐ ┌────┐ ┌────┐             ┌────┐
│    │ │    │ │    │             │    │
│vm1 │ │vm2 │ │vm3 │             │host│
│    │ │    │ │    │             │    │
└────┘ └────┘ └────┘             └────┘

--->
