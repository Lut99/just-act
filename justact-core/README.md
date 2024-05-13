# JustAct: Core Ontology
This crate defines the core ontology for the JustAct framework, as presented in the paper [1] (see the [root README](/README.md) for references).


## Ontology
The currently implemented ontology is a little simpler, yet more detailled, than the one in the paper for implementation purposes.

TODO

Most notably, the implemented specification differs from the diagram in the paper (see below) in the following ways:
- _truths_ and _facts_ have been omitted, as these are not used at the policy language-agnostic level of the framework;
- authorship has a layer of indirection through identifiers, for performance reasons; and
- _agreements_ and _actions_ do not contain statements, but rather messages. As such, an explicit additional check is necessary at audit time to ensure these are stated.

Specifically, the following traits directly correlate to sets mentioned in the framework figure:
- `Message`s represents the policy-on-the-wire, i.e., information exchanged between agents.
- `MessageSet`s represents combinations of `Message`s that one forms a composed `Policy`. Not necessarily a _valid_ policy, just a policy.
- `Policy` is the thing carried by `MessageSet`s.
    - `ExtractablePolicy` extends `Policy`s with the ability to be extracted from a `MessageSet`.
- `Agreement`s are special (stated) `MessageSet`s that are confirmed to be synchronized between agents.
- `Action`s are a collection of `Message`(`Set`)s (basis, justification, enactment) that gives justification to some statements.
    - `AuditableAction`s extend `Action`s with the ability to audit them.
- `Statements` is a local interface to all the `Message`s that are stated and known by the agent in question.
- `Actions` is a local interface to all the `Action`s that are stated and known by the agent in question.
- `Times` is a global interface to the known timesteps that are synchronized between all agents.
- `Agreements` is a global interface to the known `Agreement`s that are synchronized between all agents.
- `Agent`s represent the agents in the system.
- `RationalAgent`s represent an agent that reasons over `Statements`, `Actions`, `Times` and `Agreements` and publishes new ones as necessary.

There are also some more implementation-oriented traits:
- `Set`s abstract over different types of sets (`MessageSet`, `Statements`, `Actions`, `Times` and `Agreements` are all sets).
- `Identifiable` represents some object that has unique identifiers (`Message`, `Agent`).
- `Authored` represents some object that has an author (`Message`).
- `LocalView` is a shorthand trait for something that implements both `Statements` and `Actions`.
- `GlobalView` is a shorthand trait for something that implements both `Times` and `Agreements`.


## Features
This crate has no features.
