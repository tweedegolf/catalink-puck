CATALINK Blueprint
==================

The code in this repository represents an early version of an implementation of CATALINK on Precursor.
The goal of this version is to show that CATALINK (with its current design based on puckfs communication)
can be implemented on Precursor, and to invite interest and feedback on the technical design choices
in the internal protocol, cryptographic design, user interface, supply chain, and effects on
implementation of these choices.

Organisation
------------

* `precursor/` Main application that should be installed on precursor. This is the actual catalink-app.

* `puckfs/` Low-level Puckfs implementation to talk to the Go code published in https://github.com/n2vi/hotline. This focusses on a reliable connection between Puck and Broker.

* `puckmsg/` Higher-level interface to talk to the Go code published in https://github.com/n2vi/hotline. This focusses on communicating with other parties on Puck via the Broker.

License
-------
This code is made available under an OpenBSD-style license, see the file `LICENSE`.
