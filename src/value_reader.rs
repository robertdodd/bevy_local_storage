use std::{pin::Pin, sync::Arc, task::Poll};

use bevy::asset::io::{AsyncSeekForward, Reader};
use bevy::tasks::futures_lite::ready;
use futures_io::AsyncRead;

/// Stores either an allocated vec of bytes or a static array of bytes.
///
/// Copied from [`bevy_asset::io::memory::DataReader`].
#[derive(Clone, Debug)]
pub enum Value {
    Vec(Arc<Vec<u8>>),
    Static(&'static [u8]),
}

impl Value {
    fn value(&self) -> &[u8] {
        match &self {
            Self::Vec(vec) => vec,
            Self::Static(value) => value,
        }
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Self::Vec(Arc::new(value))
    }
}

impl From<&'static [u8]> for Value {
    fn from(value: &'static [u8]) -> Self {
        Self::Static(value)
    }
}

impl<const N: usize> From<&'static [u8; N]> for Value {
    fn from(value: &'static [u8; N]) -> Self {
        Self::Static(value)
    }
}

/// Reader for `Value` objects.
///
/// Copied from [`bevy_asset::io::memory::DataReader`].
pub(crate) struct ValueReader {
    pub(crate) value: Value,
    pub(crate) bytes_read: usize,
}

impl AsyncRead for ValueReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<futures_io::Result<usize>> {
        if self.bytes_read >= self.value.value().len() {
            Poll::Ready(Ok(0))
        } else {
            let n =
                ready!(Pin::new(&mut &self.value.value()[self.bytes_read..]).poll_read(cx, buf))?;
            self.bytes_read += n;
            Poll::Ready(Ok(n))
        }
    }
}

impl AsyncSeekForward for ValueReader {
    fn poll_seek_forward(
        mut self: Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
        offset: u64,
    ) -> Poll<std::io::Result<u64>> {
        let result = self
            .bytes_read
            .try_into()
            .map(|bytes_read: u64| bytes_read + offset);

        if let Ok(new_pos) = result {
            self.bytes_read = new_pos as _;
            Poll::Ready(Ok(new_pos as _))
        } else {
            Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek position is out of range",
            )))
        }
    }
}

impl Reader for ValueReader {
    fn read_to_end<'a>(
        &'a mut self,
        buf: &'a mut Vec<u8>,
    ) -> stackfuture::StackFuture<'a, std::io::Result<usize>, { super::STACK_FUTURE_SIZE }> {
        stackfuture::StackFuture::from(async {
            if self.bytes_read >= self.value.value().len() {
                Ok(0)
            } else {
                buf.extend_from_slice(&self.value.value()[self.bytes_read..]);
                let n = self.value.value().len() - self.bytes_read;
                self.bytes_read = self.value.value().len();
                Ok(n)
            }
        })
    }
}
