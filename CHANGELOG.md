# CHANGELOG for the JustAct Prototype
This file tracks notable changes to the JustAct framework. This is tracked per crate, unless changes are cross-crate, which are then logged as `General` changes.

This project uses [semantic versioning](https://semver.org). As such, breaking changes are indicated as **\[breaking\]**.


## v1.0.0 - TODO-TO-DO
Update to include the up-to-date framework version.

### Core - Added
- Added `ExtractablePolicy` to be able to truly abstract over policy implementation in the final framework layer (`prototype`).
- Added the `Times`-trait to represent Agent's knowledge of current time.
- Added the `Agreements`-trait, to represent the part of Agent's knowledge that is explicitly synchronized.
- Added `GlobalView` to represent an agent's globally synchronized system state.
- Added `LocalView` to represent an agent's local, non-synchronized system state.

### Core - Changed
- Separated `Action`s into `Action`s and `AuditableAction`s, to better represent the optional extension. **\[breaking\]**
- Merged `Statements` and `Stating` back into one `Statements`-trait. This trait is now mainly a plain `Set`. **\[breaking\]**
- Separated `Action`s out of `Statements` in into a separate set, `Actions`. **\[breaking\]**

### Core - Removed
- Removed `async` versions of traits, as these are not yet used. **\[breaking\]**


### Policy - Removed
- The `justact-policy`-crate no longer exists. Instead, the policies are grouped as separate crates in the [`policy-langs/`](./policy-langs/)-directory.

### Datalog - Added
- Added a parser for $Datalog^\neg$ using the [`snack`](https://github.com/Lut99/ast-toolkit-rs)-crate.
- Implemented `Policy` for `Spec`s directly.
- Implemented `ExtractablePolicy` for `Spec`s.


### Prototype - Changed
- Renamed `demo`-environment to `prototype`-environment, as that better reflects its intended usage from now on. **\[breaking\]**



## v0.1.0 - 2024-04-26
Initial release!

### General - Added
- The `justact-core`-crate, implementing a variant of the core ontology as discussed in the paper.
- The `justact-policy`-crate, implementing the core ontology traits for various policy languages.
- The `justact-demo`-crate, implementing a simple simulation environment for testing JustAct in various scenarios.


### Core - Added
- `Message`-, `MessageSet`- and `Action`-traits for encoding policy-on-the-wire.
    - Added `Set` as a non-`Message`-specific set abstraction trait.
- `Policy`-trait for encoding policy.
- `Statements`-trait for encoding stated messages and enacted actions.
    - Also added `StatementsAsync` counterpart.
- `Agent`- and `RationalAgent`-traits for representing agents.
    - Also added `RationalAgentAsync` counterpart for the latter.


### Policy - Added
- Added $Datalog^\neg$-support through the `datalog`-crate.
    - This implement `Policy` from the core ontology for $Datalog^\neg$.


### Datalog - Added
- Initial support for the $Datalog^\neg$ AST.
- Deriving truth values of atoms in programs through the alternating fixpoint semantings.
- Writing $Datalog^\neg$ programs as an embedded DSL through the `datalog!()`-macro.


### Demo - Added
- $Datalog^\neg$-specific implementations for `Message`, `MessageSet` and `Action`.
- Fully in-memory implementation for `Statements`.
- Some `Interface` that agents use to log their actions.
- Examples:
    - The first example from the paper, Step1 ([`paper1.rs`](/justact-demo/examples/paper1.rs)).
        - This adds the `AbstractAgent`, `Consortium`, `Administrator`, `Amy` and `Anton` implementations for `RationalAgent`.
