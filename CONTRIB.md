# How virshle works.

It is based on
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
wich is a virtual machine manager (type-2 hypervisor).

Virshle manages the resources like vms, disks, and networks.

## Internal.

The aim is to keep stuffs simple, so virshle only manages files.

- The disk (containing the OS) is a simple `qcow2` files with efi support generated with
  [nixos-generators](https://github.com/nix-community/nixos-generators).

- Each vm is a running instance of cloud-hypervisor linked to its
  own resources (disks, networks).

For the sake of speed resources are registered into a database,
which one is not crucial for virshle functionning.
The database acts like an index and can be generated from virshle managed files in
`/var/lib/virshle/`

## Rest API

No need to sort Vms by id in rust code,
because they are already sorted by id in the database.

Structs returned by the API are sorted by id.
