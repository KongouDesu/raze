///! Different `Read` wrappers, useful for file uploading.
///! These can be composed to combine their effects
use futures::ready;
use pin_project::pin_project;
use sha1::Sha1;
use std::cmp::min;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::{
    io::{AsyncRead, ReadBuf},
    time::{Instant, Sleep},
};
use tokio_util::codec::{BytesCodec, FramedRead};

/// Wraps an `AsyncRead`, computing the Sha1 hash along the way and returning it when the inner reader is done
///
/// The hash is returned as 40 hexadecimal digits

#[pin_project]
pub struct AsyncReadHashAtEnd<R: AsyncRead> {
    #[pin]
    inner: R,
    hash: Sha1,
    hash_read: usize,
}

impl<R: AsyncRead> AsyncReadHashAtEnd<R> {
    pub fn wrap(reader: R) -> Self {
        Self {
            inner: reader,
            hash: Sha1::new(),
            hash_read: 0,
        }
    }
}

impl<R: AsyncRead> AsyncRead for AsyncReadHashAtEnd<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.project();
        let before_filled_len = buf.filled().len();
        ready!(this.inner.poll_read(cx, buf))?;
        let after_filled = buf.filled();
        let read_amount = after_filled.len() - before_filled_len;
        let hash_read = *this.hash_read;
        // If the inner AsyncRead is still returning data, hash the data.
        if read_amount > 0 {
            this.hash.update(&after_filled[before_filled_len..]);
        }
        // If EOF is reached but we haven't write all 40 bytes of the hash yet, write the hash.
        else if hash_read < 40 {
            let digest = this.hash.hexdigest();
            let digest = digest.as_bytes();

            let remaining = buf.remaining();

            // 1. If the buffer isn't large enough for our hash, we need to split it up OR
            // 2. If we've already sent part of the hash, we need to only return the remainder
            if remaining < 40 || hash_read > 0 {
                // Determine how much we can return (in case the buffer is too small)
                let hash_end_index = min(40, hash_read + remaining);
                let hash_read_amount = hash_end_index - hash_read;

                buf.put_slice(&digest[hash_read..hash_end_index]);
                *this.hash_read += hash_read_amount;
            } else {
                // If there's enough room, just send the 40-byte hexdigest directly
                buf.put_slice(&digest);
                *this.hash_read = 40;
            }
        }
        Poll::Ready(Ok(()))
    }
}

/// Wraps an `AsyncRead`, limiting the bandwidth it can use. \
/// Useful for limiting upload bandwidth.
///
/// bandwidth: maximum bytes per second \

#[pin_project]
pub struct AsyncReadThrottled<R: AsyncRead> {
    #[pin]
    inner: R,
    bandwidth: f32,
    sleep: Pin<Box<Sleep>>,
}

impl<R: AsyncRead> AsyncReadThrottled<R> {
    pub fn wrap(reader: R, bandwidth: usize) -> Self {
        Self {
            inner: reader,
            bandwidth: bandwidth as f32,
            sleep: Box::pin(tokio::time::sleep_until(Instant::now())),
        }
    }
}

impl<R: AsyncRead> AsyncRead for AsyncReadThrottled<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        use futures::Future;
        let this = self.project();
        ready!(this.sleep.as_mut().poll(cx));
        let before_rmn = buf.remaining();
        ready!(this.inner.poll_read(cx, buf))?;
        let after_rmn = buf.remaining();
        let read_amount = before_rmn - after_rmn;
        let sleep_duration: f32 = (read_amount as f32) / *this.bandwidth;
        this.sleep
            .as_mut()
            .reset(Instant::now() + Duration::from_secs_f32(sleep_duration));
        Poll::Ready(Ok(()))
    }
}

/// Wrap an [AsyncRead] into a [reqwest::Body].
pub fn body_from_reader<R: AsyncRead + Send + Sync + 'static>(file: R) -> reqwest::Body {
    let stream = FramedRead::new(file, BytesCodec::new());
    reqwest::Body::wrap_stream(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_hash_at_end() {
        use tokio::io::AsyncReadExt;
        let content = "hello this is a test".as_bytes();
        let mut read = AsyncReadHashAtEnd::wrap(content);
        let mut buf = Vec::new();
        read.read_to_end(&mut buf).await.unwrap();
        let l = buf.len();
        let (content, appended_hash) = buf.split_at(l - 40);
        let mut hasher = Sha1::new();
        hasher.update(content);
        let digest = hasher.hexdigest();
        let computed_hash = digest.as_bytes();
        assert_eq!(
            computed_hash,
            "f291f60cafb2ef2e0013f5a5889b1da5af4b4657".as_bytes()
        );
        assert_eq!(appended_hash, computed_hash);
    }

    #[tokio::test]
    async fn test_thrrottled_read() {
        // Test reading 512 bytes at a bandwidth of 256 bytes / sec. Should complete in around 2 secs.
        use tokio::io::AsyncReadExt;
        let file = tokio::fs::File::open("/dev/urandom").await.unwrap();
        let mut read = AsyncReadThrottled::wrap(file, 256);
        let mut buf: Vec<u8> = vec![0; 512];
        let start = Instant::now();
        for chunk in buf.chunks_mut(16) {
            read.read_exact(chunk).await.unwrap();
        }
        let end = Instant::now();
        let elapsed = (end - start).as_secs_f32();
        let expected = 2f32;
        assert!((elapsed - expected).abs() < 0.2);
    }
}
