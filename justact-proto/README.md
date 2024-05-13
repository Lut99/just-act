# JustAct: Demo Simulation
Implements a simplified environment for the JustAct framework.

Specfically, provides implementations of the [core ontology](../justact-core/README.md) such that simple scenarios can be executed.


## Policy languages
Currently, the demo environment supports $Datalog^\neg$ as only policy language. As a consequence, `Message`, `MessageSet` and `Action` are all available only as `datalog`-variants. Enable the `datalog`-feature to make use of this.


## Execution
This environment performs simple step-wise execution of all agents.

Specifically, it implements an executor that:
1. Executes the `poll()` method on all its `Agent`s once;
2. Runs an audit on all published `Action`s, reporting their validity to the user;
3. Removes any `Agent`s that have reported `AgentPoll::Dead`.
4. Goes to 1 as long as there is at least one agent left.

Note that this is implemented without `async`. The appropriate core ontology traits have been used for that.


## Features
This crate supports the following features:
- `datalog`: Enables the use of $Datalog^\neg$-messages in the simulation.
- `datalog-log`: Enables `log`-traces in the `datalog` crate for debugging.
