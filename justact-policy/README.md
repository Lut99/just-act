# JustAct: Policy Languages
This crate implements various policy languages for use with the JustAct framework.

Currently, only $Datalog^\neg$, a simple Datalog-dialect, is supported. See the [`datalog`](./lang/datalog/)-crate for more information.

See the [repository root](/README.md) for more information, including references.


## $Datalog^\neg$
The paper [1] uses Datalog with negation as example policy language for showcasing framework operation.

As such, a small interpreter is included that can reason about this language.

For more information on the language, see its own [README](./lang/datalog/README.md).


## Features
This crate supports the following features:
- `derive`: Enables the `datalog-derive`-feature.
- `log`: Enables the `datalog-log`-feature.
- `datalog`: Enables a `Policy`-implementation for $Datalog^\neg$-programs.
- `datalog-derive`: Enables the `datalog!()`-macro for writing $Datalog^\neg$-programs as a Rust embedded DSL.
- `datalog-log`: Enables `log`-traces during the derivation process for $Datalog^\neg$-programs for debugging purposes.
