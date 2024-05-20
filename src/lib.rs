use std::{path::Path, pin::Pin, task::Poll};

use bevy::{
    asset::io::{
        memory::Value, AssetReader, AssetReaderError, AssetSource, AssetSourceId, PathStream,
        Reader,
    },
    prelude::*,
    tasks::futures_lite::{ready, AsyncRead},
    utils::BoxedFuture,
};

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

/// Struct that implements [`AsyncRead`] for a string.
///
/// Copied from [`bevy_asset::io::memory::DataReader`].
struct DataReader {
    data: Value,
    bytes_read: usize,
}

impl DataReader {
    fn value(&self) -> &[u8] {
        match &self.data {
            Value::Vec(vec) => vec,
            Value::Static(value) => value,
        }
    }
}

/// Implement [`AsyncRead`] for [`DataReader`].
///
/// Copied from [`bevy_asset::io::memory::DataReader`].
impl AsyncRead for DataReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<futures_io::Result<usize>> {
        if self.bytes_read >= self.value().len() {
            Poll::Ready(Ok(0))
        } else {
            let n = ready!(Pin::new(&mut &self.value()[self.bytes_read..]).poll_read(cx, buf))?;
            self.bytes_read += n;
            Poll::Ready(Ok(n))
        }
    }
}

/// A custom asset reader implementation that wraps a given asset reader implementation
struct LocalStorageAssetReader;

impl AssetReader for LocalStorageAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move {
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

            // Return the value wrapped in [`DataReader`] so that it implements [`AsyncRead`].
            let reader: Box<Reader> = Box::new(DataReader {
                data: Value::from(value.as_bytes().to_vec()),
                bytes_read: 0,
            });
            Ok(reader)
        })
    }

    /// Not implemented for local storage.
    ///
    /// Always raises [`AssetReaderError::NotFound`].
    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async move { Err(AssetReaderError::NotFound(path.to_path_buf())) })
    }

    /// Not implemented, there are no directories in local storage.
    ///
    /// Always raises [`AssetReaderError::NotFound`].
    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(async move { Err(AssetReaderError::NotFound(path.to_path_buf())) })
    }

    /// Not implemented, there are no directories in local storage.
    ///
    /// Always returns [`false`].
    fn is_directory<'a>(
        &'a self,
        _path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        Box::pin(async move { Ok(false) })
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
