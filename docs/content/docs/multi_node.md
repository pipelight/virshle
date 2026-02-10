+++
date = 2025-09-11
updated = 2026-02-10

weight = 10

title = "Virshle documentation 📖"

description = """

"""

draft=false
+++

## Multiple node usage.

When you need to manage vms on many machines.

```txt
┌──────┬──────┐            ┌──────┬──────┐
│      │      │            │      │      │
│      │      │            │      │      │
│ vm_1 │ vm_2 │            │ vm_1 │ vm_2 │
│      │      │            │      │      │
│      │      │            │      │      │
├──────┴──────┴──────┐     ├──────┴──────┴──────┐
│   node_1           │     │   node_2           │
└─────▲──────────────┘     └─────▲──────────────┘
      │                          │
      │                          │
      │                          │
┌─────┴───┬──────────────────────┘
│         │
│   cli   │
│         │
└─────────┘
```

### Connect to remote nodes.

Connection between the client and servers are done through
**unix-sockets** or **ssh**.

You can create a list of manageable nodes in the configuration file at
`/etc/virshle/config.toml`

```toml
# /etc/virshle/config.toml

# local host
[[node]]
name = "local"
url = "unix:///var/lib/virshle/virshle.sock"

# local host through ssh
[[node]]
name = "local-ssh"
url = "ssh://anon@deku:22/var/lib/virshle/virshle.sock"
```

_When specifying nodes url,
you have to explicitly write your local node address if you want to use it._

For virshle to access a node through ssh, it needs the **authorized_key**
into a running **ssh-agent**.
Make sure you have an ssh-agent running with your key loaded inside.

```sh
virshle node ls -vvv
```

![node_list_multi](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_vvv_multi.png)

### Node load balancing.

When you work with multiple nodes, and create a machine with
`v vm create -t xs`
without giving a node to work on
`--node <node_name>`,

The **load balancer** chooses a random (and not saturated) node,
You can add a `weight` to the node if you want it to be chosen
more often.

```toml
# /etc/virshle/config.toml

# local host
[[node]]
name = "remote_1"
url = "ssh://anon@remote_1:22/var/lib/virshle/virshle.sock"
weight = 10

# local host through ssh
[[node]]
name = "remote_2"
url = "ssh://anon@remote_2:22/var/lib/virshle/virshle.sock"
weight = 2
```

### Node health check.

Instead of troubleshooting the node by hand with your favourite tools(df, free, htop...),
you may have a quick glance at your node global state.

```sh
virshle node ls -all -vvv
```

![node_list_all](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_all_vvv.png)

Here can you see **used resources**,
plus **reserved resources** for your VMs.

For example, you can, of course, reserve more CPUs than what you physically have on a host
and the linux kernel will share the power between guests.

### Disks

Copy your local disks to the new node cache.

```sh
rsync --progress -azv /var/lib/virshle/cache/* remote:/var/lib/virshle/cache
```

### Create Vms

You can either choose the node which to create the vm on.

```sh
v vm create -t xs --node <node_name>
```

Or let the node balancer choose the best node for your vm.

```sh
v vm create -t xs
```
