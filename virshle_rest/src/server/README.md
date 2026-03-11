# Server methods

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
