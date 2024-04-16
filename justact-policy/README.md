# JustAct: Policy Languages
This crate implements various policy languages for use with the JustAct framework.

Currently, only $Datalog^\neg$, a simple Datalog-dialect, is supported. See the [`datalog`](./lang/datalog/)-crate for more information.

See the [repository root](/README.md) for more information, including references.


## $Datalog^\neg$
The paper [1] uses Datalog with negation as example policy language for showcasing framework operation.

As such, a small interpreter is included that can reason about this language.

As it stands, though, does the interpreter not yet have any parser. Instead, use the `datalog!{}`-macro to write Datalog as an embedded DSL in Rust.

For more information on the language, see its own [README](./lang/datalog/README.md).
