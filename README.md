# Bevy Local Storage

A simple [Bevy](https://bevyengine.org/) plugin that adds a [`LocalStorage`](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage) asset reader.

---

## Example

The example is a separate crate because `trunk` does not support cargo examples yet.

```shell
cd example_crate
trunk serve
```

* requires [trunk](https://crates.io/crates/trunk): `cargo install --locked trunk`
* requires `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
* this will serve your app on `8080` and automatically rebuild + reload it after code changes

## Usage

Add the plugin *before* `DefaultPlugins`.

```rust
App::new()
    .add_plugins((
        LocalStorageAssetReaderPlugin {
            asset_id: "local",
        },
        DefaultPlugins,
    ));
```

You can now load assets from local storage like so:

```rust
commands.spawn(DynamicSceneBundle {
    scene: asset_server.load(format!("local://example.scn.ron")),
    ..default()
});
```

## Write to LocalStorage

Write to local storage like so.

```rust
use bevy_local_storage::get_local_storage;

// Get a local storage instance
// PANICS: if local storage not present
let local_storage: web_sys::Storage = get_local_storage();
// Write to local storage
local_storage.set_item("example.scn.ron", SCENE_DATA).unwrap();
```

See [web-sys::Storage](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Storage.html) for more methods.

## Compatible Bevy versions

| `bevy_local_storage` | `bevy` |
|:---------------------|:-------|
| `0.1`                | `0.13` |

## License

Dual-licensed under either of

- Apache License, Version 2.0,
  ([LICENSE-APACHE](https://github.com/robertdodd/bevy_local_storage/blob/master/LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](https://github.com/robertdodd/bevy_local_storage/blob/master/LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
