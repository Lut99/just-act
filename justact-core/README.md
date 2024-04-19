# JustAct: Core Ontology
This crate defines the core ontology for the JustAct framework, as presented in the paper [1] (see the [root README](/README.md) for references).


## Ontology
The currently implemented ontology is a little simpler, yet more detailled, than the one in the paper for implementation purposes.

Specifically, the following is implemented:
- `Message`s represents the policy-on-the-wire, i.e., information exchanged between agents.
- `Policy` is the thing carried by `Message`s.
- `MessageSet`s represents combinations of `Message`s that one forms a composed `Policy`. Not necessarily a _valid_ policy, just a policy.
- `Action`s are a collection of `Message`(`Set`)s (basis, justification, enactment) that gives justification to some statements.
- `Statements` is an interface to all the `Message`s that are stated and/or `Actions` that are enacted. Depending on the implementation, this can be a fully-public pool or offer agents asymmetric views.
- `Agent`s represent the agents in the system.
- `RationalAgent`s represent an agent that reasons over stated `Messages`/enacted `Actions` in a `Statements` and might push either of those.

As an implementation detail, some objects (`Statements`, `RationalAgent`) have an `async` counterpart available for more complex environments (`StatementsAsync` and `RationalAgentAsync`, respectively).


## Features
This crate has no features.