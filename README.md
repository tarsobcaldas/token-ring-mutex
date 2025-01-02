# Token Ring Mutex Peer Calculator

This project implements the Token Ring algorithm described in Chapter 6 of van Steen & Tanenbaum (2020) in Rust using the UDP Sockets library.
The peers share a token between themselves, and the one that has it sends requests for calculations to the server, which have been generated
using a Poisson distribution.

The server can be started with
```bash
    cargo run server [PORT]
```

And the peers should be added with
```bash
    cargo run peer [PORT] [NEXT PEER ADDRESS] [SERVER ADRESS]
```

For the program to work, the addresses should be in the form \[IP]:\[PORT], and the peers need to form a circle, that is, 
each peer should refer to the next, while the last refers to the first.

After all the peers have been added, start the listening process with `listen` on all of the peers, except one, which
is the one that will send the first token, and should be intialized with `start`.
