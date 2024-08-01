use std::path::Path;

use bevy::{asset::io::*, prelude::*};

mod value_reader;
use value_reader::*;

/// A plugins that registers a LocalStorage asset reader.
///
/// **IMPORTANT:** This plugin must be added before [`DefaultPlugins`].
///
/// ## Example:
///
/// ```ignore
/// App::new()
///     .add_plugins((
///         LocalStorageAssetReaderPlugin {
///             asset_id: "local",
///         },
///         DefaultPlugins,
///     ));
/// ```
pub struct LocalStorageAssetReaderPlugin {
    pub asset_id: &'static str,
}

impl Plugin for LocalStorageAssetReaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            AssetSourceId::from(self.asset_id),
            AssetSource::build()
                .with_reader(|| Box::new(LocalStorageAssetReader))
                .with_processed_reader(move || Box::new(LocalStorageAssetReader))
                // Note that we only add a processed watch warning because we don't want to warn
                // noisily about embedded watching (which is niche) when users enable file watching.
                .with_processed_watch_warning("Watching local storage not supported"),
        );
    }
}

/// A custom asset reader implementation that wraps a given asset reader implementation
struct LocalStorageAssetReader;

impl AssetReader for LocalStorageAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        // Read the value from local storage.
        // May any errors, or [`None`] values, to [`AssetReaderError::NotFound`]
        let storage = get_local_storage();
        let key = path.to_str().unwrap().to_string();
        let entry = storage
            .get_item(&key)
            .map_err(|_| AssetReaderError::NotFound(path.to_path_buf()))?;
        let value = entry
            .as_ref()
            .ok_or(AssetReaderError::NotFound(path.to_path_buf()))?;

        // Return the value wrapped in [`ValueReader`] so that it implements [`AsyncRead`].
        let reader = Box::new(ValueReader {
            value: Value::from(value.as_bytes().to_vec()),
            bytes_read: 0,
        });
        Ok(reader)
    }

    /// Not implemented for local storage.
    ///
    /// Always raises [`AssetReaderError::NotFound`].
    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<Box<Reader<'a>>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    /// Not implemented, there are no directories in local storage.
    ///
    /// Always raises [`AssetReaderError::NotFound`].
    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    /// Not implemented, there are no directories in local storage.
    ///
    /// Always returns [`false`].
    async fn is_directory<'a>(
        &'a self,
        _: &'a Path,
    ) -> std::result::Result<bool, AssetReaderError> {
        Ok(false)
    }
}

/// Utility that returns a local storage instance.
///
/// ## Panics
///
/// If local storage not present.
pub fn get_local_storage() -> web_sys::Storage {
    web_sys::window()
        .expect("No window")
        .local_storage()
        .expect("Failed to get local storage")
        .expect("No local storage")
}
