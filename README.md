# Schema plugin

WASM plugin for [proto](https://github.com/moonrepo/proto) that is powered by a schema. Currently supports TOML.

## Contributing

Build the plugin:

```shell
cargo build --target wasm32-wasi
```

Test the plugin by running `proto` commands. Requires proto >= v0.12.

```shell
proto install schema-test
proto list-remote schema-test
```
