# myu

*myu* is a model-checker for Labeled Transition Systems using a subset of the modal μ-calculus. It works with LTS specified in the [aldebaran format](https://www.mcrl2.org/web/user_manual/language_reference/lts.html#aldebaran-format). The sub-set of modal μ-calculus used is specified by the following grammar:
```
f, g ::= false | true | X | (f && g) | (f || g) | <a>f | [a]f | mu X. f | nu X. f
```
Here, a is an arbitrary lower-case string (i.e. `a ∈ [a-z][a-z,0-9,_]∗` consists of alphanumeric characters and/or the underscore character) matching an action name and `X ∈ [A-Z]` is a recursion variable.

## Installation
### From source
Assuming [rust is installed](https://www.rust-lang.org/tools/install), *myu* can be built with the command
```
cargo build --release
```
after which the resulting binary will be placed in the `target/release` folder.

## Usage
The usage of *myu* can be found with `myu --help`:
```
myu 0.1.0
A model-checker for Labeled Transition Systems using a subset of the modal μ-calculus

USAGE:
    myu [FLAGS] <lts> <mcf>

FLAGS:
    -h, --help       Prints help information
        --naive      Use naive algorithm instead of the Emerson-Lei algorithm
    -V, --version    Prints version information

ARGS:
    <lts>    File specifying the LTS to be verified in aldebaran format
    <mcf>    File specifying the formula to check in modal μ-calculus
```

## Known quirks
* *myu* does not implement variable shadowing; given a formula with variables declared more than once, the expected behaviour is undefined.
* *myu* does not know how to deal with top-level open variables; if the top-level formula contains open-variables *myu* will panic.
