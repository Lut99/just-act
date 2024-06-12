# JustAct: Actions Universally Justified by Partial Dynamic Policies
Prototype implementation of the JustAct framework[1].

Specifically, implements the ontology as described in the paper ([`justact-core`](./justact-core/)), some policy languages that can be plugged-in ([`justact-policy`](./justact-policy/)) and then a demo environment that runs scenario's ([`justact-demo`](./justact-demo/)).


## Repository structure
This repository is structured as follows.

The core ontology, and therefore the root crate of the project, is defined in the [`justact-core`](./justact-core/)-crate. It is a little more detailled than the paper, as it aims to provide the ontology as a set of interfaces (Rust traits) that, when implemented, allow one to emulate the relations presented.

As the paper does not specify policy languages, some languages are implemented in the [`policy-langs/`](./policy-langs/)-folder. Specifically, this project hosts a small $Datalog^\neg$ interpreter, which is the example language used in the paper. The code can be found in the [`datalog`](./policy-langs/datalog/)-crate.

Finally, the repository aims to support multiple demo environments. Currently, only the [`justact-prototype`](./justact-proto/)-crate implements a simple, purely in-memory simulator that runs agents step-by-step. Actual implementations of scripts for agents is given in that crate's [examples](./justact-proto/examples/) directory.


## Installation
To install the crate manually, first be sure to install the latest version of Rust. Usually, the easiest is to use <https://rustup.rs>, i.e.,
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Installing with the default settings will work.

When finished, don't forget to reload your PATH by re-starting your shell or running:
```bash
source ~/.cargo/env
```

If you have already installed Rust, make sure you run at least version 1.77.1. If you installed using rustup, then update by running:
```bash
rustup update
```


## Usage
To run the examples in the repository, use the `cargo run --example`-command. First, choose a simulator crate to use as backend and select that as the package; then, run examples for that simulator by giving the name of the example you'd like to run without `.rs`.

For example, to run [`paper1.rs`](./justact-proto/examples/paper1.rs) from the [`justact-prototype`](./justact-proto/)-crate, type:
```bash
cargo run --package justact-prototype --example paper1
```

Some examples require specific features to be enabled, e.g., use a specific language. See the README-file in the examples folder for an overview, or simply check the topmost error if compiling the file fails.


## Contribution
Contributions to this codebase are welcome. Feel free to [raise an issue](https://github.com/Lut99/just-act/issues) or [create a pull request](https://github.com/Lut99/just-act/pulls) if you want to.


## License
This project is licensed under TODO. See [LICENSE](./LICENSE) for more details.


## References
[1] Esterhuyse, C.A., Müller, T., van Binsbergen, L.T. (2024). _JustAct: Actions Universally Justified by Partial Dynamic Policies._ In: Castiglioni, V., Francalanza, A. (eds) Formal Techniques for Distributed Objects, Components, and Systems. FORTE 2024. Lecture Notes in Computer Science, vol 14678. Springer, Cham. <https://doi.org/10.1007/978-3-031-62645-6_4>

[2] A. Van Gelder. 1989. _The alternating fixpoint of logic programs with negation._ In Proceedings of the eighth ACM SIGACT-SIGMOD-SIGART symposium on Principles of database systems (PODS '89). Association for Computing Machinery, New York, NY, USA, 1–10. <https://doi.org/10.1145/73721.73722>
