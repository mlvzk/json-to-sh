# json-to-sh

Fast JSON to Shell parser in Rust

# Installation

Download the executable from this repo's [Releases page](https://github.com/mlvzk/json-to-sh/releases) and put it in your $PATH

OR

```sh
cargo build --release
```

# Usage

```sh
eval $( json-to-sh <some.json )
echo $root_v
```

The default namespace for variables is `root`.

`{ "v": 5 }` outputs `root_v="5"`.

In key names, all characters except 0-9, a-z, A-Z and _ are allowed, illegal characters are skipped.

`{ "0vA!@#z": 5 }` outputs `root_0vAz="5"`.

Keys and indexes are separated by `_` (underscore)

`[0, 1, { "x": true }]` outputs:

```sh
root_0="0"
root_1="1"
root_2_x="true"
```