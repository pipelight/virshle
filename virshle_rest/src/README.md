# Internal of rest API

## Client methods

The client can reach one or multiple nodes/peer.
By default it will knock to the local node.

If a peer is requested by its name, the client will connect to the peer.
There is only a few methods to knock at all peers at once like:

- ping
- vm_list: /vm/many

For other methods a peer name is mandatory.

## Server methods

The server methods communicate with the LOCAL NODE ONLY.

# Research

Need server specific methods
->
Get vm return Vms on running node only.

Peer specific methods
->
Get vm return Vms on running on every peer.

## Calling peers.

How to break the peer calling chain?

Expose methods without recursion level
to work only on local peer.
methods under node()

Expose methods with recursion level
methods under peer()

- Solution 1:

  Only one level of recursion.
  The node calls its whitelisted peers only.

  Node -> Peer

- Solution 2:

  Multiple levels of recursion
  The peer calls its peers too.

  Node -> Peer -> Peer...

- Solution 3:

  Choose a custom recursion level
