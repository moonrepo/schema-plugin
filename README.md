# Schema plugin

WASM plugin for [proto](https://github.com/moonrepo/proto) that is powered by a schema. Currently supports TOML.

## Contributing

Build the plugin:

```shell
cargo build --target wasm32-wasi
```

Test the plugin by running `proto` commands. Requires proto >= v0.14.

```shell
proto install moon-test
proto list-remote moon-test
```

> Since this plugin requires an external schema file, its testing uses moon: https://moonrepo.dev/docs/install#proto
