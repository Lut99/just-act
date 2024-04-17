# CHANGELOG for the JustAct Prototype
This file tracks notable changes to the JustAct framework. This is tracked per crate, unless changes are cross-crate, which are then logged as `General` changes.

This project uses [semantic versioning](https://semver.org). As such, breaking changes are indicated as **\[breaking\]**.


## v0.1.0 - 2024-04-17
Initial release!

### General - Added
- The `justact-core`-crate, implementing a variant of the core ontology as discussed in the paper.
- The `justact-policy`-crate, implementing the core ontology traits for various policy languages.
- The `justact-demo`-crate, implementing a simple simulation environment for testing JustAct in various scenarios.


### Core - Added
- `Message`-, `MessageSet`- and `Action`-traits for encoding policy-on-the-wire.
- `Policy`-trait for encoding policy.
- `MessagePool`- and `Interface`-traits for representing agent communication channels.
    - Also added `MessagePoolAsync` and `InterfaceAsync` counterparts.
- `Agent`- and `RationalAgent`-traits for representing agents.
    - Also added `RationalAgentAsync` counterpart.


### Policy - Added
- Added $Datalog^\neg$-support through the `datalog`-crate.
    - This implement `Policy` from the core ontology for $Datalog^\neg$.


### Datalog - Added
- Initial support for the $Datalog^\neg$ AST.
- Deriving truth values of atoms in programs through the alternating fixpoint semantings.
- Writing $Datalog^\neg$ programs as an embedded DSL through the `datalog!()`-macro.


### Demo - Added
- $Datalog^\neg$-specific implementations for `Message`, `MessageSet` and `Action`.
- Simple, synchronous implementations for `MessagePool` and `Interface`.
- Examples:
    - The first example from the paper, Step1 ([`paper1.rs`](/justact-demo/examples/paper1.rs)).
        - This adds the `AbstractAgent`, `Consortium`, `Administrator`, `Amy` and `Anton` implementations for `RationalAgent`.
