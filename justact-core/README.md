# JustAct: Core Ontology
This crate defines the core ontology for the JustAct framework, as presented in the paper [1] (see the [root README](/README.md) for references).


## Ontology
The currently implemented ontology is a little simpler, yet more detailled, than the one in the paper for implementation purposes.

Specifically, the following is implemented:
- `Message`s represents the policy-on-the-wire, i.e., information exchanged between agents.
- `Policy` is the thing carried by `Message`s.
- `MessageSet`s represents meaningful combinations of `Message`s to an extend that one forms a composed `Policy`. Not necessarily a _valid_ policy, just a policy.
- `Action`s are a collection of three `MessageSet`s (basis, justification, enactment) that gives special meaning to some `Message`s.
- `MessagePool` is an interface to all the `Message`s that an agent knows of. Depending on the implementation, this can be a fully-public pool or a private pool shared between agents.
- `Interface` is an interface for agents to communicate with the simulation input/output (e.g., interact with users, publish logs).
- `Agent`s represent the agents in the system.
- `RationalAgent`s represent an agent that reasons over available `Message`s in a `MessagePool`, pushes new `Message`s to it and communicates results through an `Interface`.

As an implementation detail, some objects (`MessagePool`, `Interface`, `RationalAgent`) have an `async` counterpart available for more complex environments (`MessagePoolAsync`, `InterfaceAsync` and `RationalAgentAsync`, respectively).


## Features
This crate has no features.