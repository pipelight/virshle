+++
date = 2025-09-11
updated = 2026-02-11

weight = 50

title = "Cloud hypervisor"

draft=true
+++

#

## Direct socket communication.

As of today,
Virshle doesn't expose a lot of methods to modify an already created VM.
The only actions you can do through the cli are:
**Create, start, stop, pause, delete,**
and **refresh network interfaces**.

For now,
if you want more flexibility, you can use the cloud-hypervisor Rest API,
through curl on the vm socket.

Get the VM uuid.

```sh
v vm ls -vvv
# or
v vm ls --json
```

And send direct http requests to the VM cloud-hypervisor socket path at
`/var/lib/virshle/vm/<vm_uuid>/ch.sock`.
