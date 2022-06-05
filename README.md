# Nonconstant LCL classifier

This tool can be used to find nonconstant lower bounds for LCL problems in the LOCAL model.

It tries to find a counterexample multigraph, in which the problem is unsolvable in the PN model.
This implies that the problem is not solvable in constant time in the LOCAL model.

This work was supported in part by the Academy of Finland, Grant 333837.

## Building the project
```
cargo build --release
```

## Usage

```
cargo run --release -- --help
```

## Running tests

```
cargo test
```

## Documentation

Open the documentation on web browser:
```
cargo doc --open
```
