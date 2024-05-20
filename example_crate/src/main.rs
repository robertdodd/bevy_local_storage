use bevy::prelude::*;

use bevy_local_storage::*;

/// Asset source ID for the local storage asset source.
///
/// Can be used to load local assets like so:
///
/// ```ignore
/// asset_server.load(format!("{LOCAL_STORAGE_ASSET_ID}://path/to/asset"));
/// ```
const LOCAL_STORAGE_ASSET_ID: &str = "local";

/// An example scene that we will save and load from local storage.
const SCENE_FILENAME: &str = "example.scn.ron";
const SCENE_DATA: &str = r#"(
  resources: {},
  entities: {
    4294967297: (
      components: {
        "example_crate::MyComponent": (),
      },
    ),
  },
)"#;

fn main() {
    App::new()
        .add_plugins((
            LocalStorageAssetReaderPlugin {
                asset_id: LOCAL_STORAGE_ASSET_ID,
            },
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Bevy Local Storage Example".to_string(),
                        resolution: (1280., 720.).into(),
                        canvas: Some("#bevy".to_owned()),
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        ))
        .register_type::<MyComponent>()
        .add_systems(Update, handle_keys)
        .run();
}

/// Component that we will be loaded from the scene.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct MyComponent;

fn handle_keys(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    query: Query<(), With<MyComponent>>,
) {
    if keys.just_pressed(KeyCode::KeyS) {
        info!("SAVE PRESSED");

        let local_storage = get_local_storage();
        local_storage.set_item(SCENE_FILENAME, SCENE_DATA).unwrap();
    } else if keys.just_pressed(KeyCode::KeyL) {
        info!("LOAD PRESSED");

        commands.spawn(DynamicSceneBundle {
            scene: asset_server.load(format!("{LOCAL_STORAGE_ASSET_ID}://{SCENE_FILENAME}")),
            ..default()
        });
    } else if keys.just_pressed(KeyCode::KeyR) {
        info!("RESET PRESSED");

        let local_storage = get_local_storage();
        local_storage.remove_item(SCENE_FILENAME).unwrap();
        asset_server.reload(format!("{LOCAL_STORAGE_ASSET_ID}://{SCENE_FILENAME}"));
    } else if keys.just_pressed(KeyCode::F1) {
        info!("There are {} MyComponents", query.iter().count());
    }
}
