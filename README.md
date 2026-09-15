# README

This is a fork of the n2vi hotline.

To start the broker:

    $ export SF=path/to/server.json
    $ cd puckfs/server && go run .

You can then "send" messages from the root dir of this repo:

    $ go run . send
    to uk

    hello
    ^D

To fetch these messages, you'll need to simulate the ROCSS by moving the message
from `puckfs/server/` to `puckfs/in/`. To actually decode the message succesfully you would also
need to swap the "My" and "Their" fields in `PrincipalsDb` (since you're simulating the other end
of the conversation)
