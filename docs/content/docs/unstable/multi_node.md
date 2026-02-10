+++
date = 2025-09-11
updated = 2026-02-10

weight = 20

title = "Running multiple nodes."

draft=true
+++

# Running multiple nodes.

{% container(type="danger") %}

**Work in progress...**

Multi node only works partially!

Some implementation choices still need to be settled regarding the community node trust system.

{% end %}

Virshle is a cli that can control multiple nodes
that manage multiple virtual machines themselves.

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

## Connect to remote nodes.

Connection between the client and servers are done through
**unix-sockets** or **ssh**.

You can create a list of manageable nodes in the configuration file at
`/etc/virshle/config.toml`

{% container(type="info") %}

When specifying nodes url,
you have to explicitly **whitelist** your local node address if you want to use it.

{% end %}

```toml
# /etc/virshle/config.toml

# local host
[[node]]
name = "local"
url = "unix:///var/lib/virshle/virshle.sock"

# local host through ssh
[[node]]
name = "remote-ssh"
url = "ssh://anon@crocuda:22/var/lib/virshle/virshle.sock"
```

However, you must **whitelist** the node in your configuration
to be able to interact with it.

{% container(type="warning") %}

**Need an ssh-agent**

For Virshle to access a node through ssh, it needs the **authorized_key**
loaded into a running **ssh-agent**.
Make sure you have an ssh-agent running with your key loaded inside.

{% end %}

```sh
virshle node ls -vvv
```

![node_list_multi](/images/v_node_ls_vvv_multi.png)

## Node load balancing.

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

To create a virtual machine over a multi_node network,
you can either choose the node which to create the VM on,

```sh
v vm create -t xs --node <node_name>
```

or let the node balancer choose the **best** node for your VM.

```sh
v vm create -t xs
```

## Node health check.

Instead of troubleshooting the node by hand with your favourite tools(df, free, htop...),
you may have a quick glance at your node global state.

```sh
virshle node ls -all -vvv
```

![node_list_all](/images/v_node_ls_all_vvv.png)

Here can you see **used resources**,
plus **reserved resources** for your VMs.

For example, you can, of course, reserve more CPUs than what you physically have on a host
and the linux kernel will share the power between guests.

### Sync template disks between nodes.

Copy your local disks to the new node cache.

```sh
rsync --progress -azv /var/lib/virshle/cache/* remote:/var/lib/virshle/cache
```
