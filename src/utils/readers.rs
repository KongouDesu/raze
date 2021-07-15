///! Different `Read` wrappers, useful for file uploading.
///! These can be composed to combine their effects
use bytes::Bytes;
use futures::{ready, Stream, TryStreamExt};
use pin_project::pin_project;
use sha1::Sha1;
use std::io::Error as IoError;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::{
    io::AsyncRead,
    time::{Instant, Sleep},
};
use tokio_util::codec::{BytesCodec, FramedRead};

/// Wraps an [Stream] of [Result<Bytes, std::io::Error>], computing the Sha1 hash along the way and returning it when the inner stream is done
///
/// The hash is returned as 40 hexadecimal digits
#[pin_project]
pub struct BytesStreamHashAtEnd<R>
where
    R: Stream<Item = Result<Bytes, IoError>>,
{
    #[pin]
    inner: R,
    hash: Sha1,
    done: bool,
}

impl<R> BytesStreamHashAtEnd<R>
where
    R: Stream<Item = Result<Bytes, IoError>>,
{
    pub fn wrap(inner: R) -> Self {
        Self {
            inner,
            hash: Sha1::new(),
            done: false,
        }
    }
}

impl<R> Stream for BytesStreamHashAtEnd<R>
where
    R: Stream<Item = Result<Bytes, IoError>>,
{
    type Item = Result<Bytes, IoError>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let bytes: Option<Result<Bytes, IoError>> = ready!(this.inner.poll_next(cx));
        match bytes {
            Some(Ok(bytes)) => {
                this.hash.update(&bytes);
                Poll::Ready(Some(Ok(bytes)))
            }
            None => {
                if !*this.done {
                    let digest = this.hash.hexdigest();
                    let digest_bytes = Bytes::copy_from_slice(digest.as_bytes());
                    *this.done = true;
                    Poll::Ready(Some(Ok(digest_bytes)))
                } else {
                    Poll::Ready(None)
                }
            }
            other => Poll::Ready(other),
        }
    }
}

/// Wraps an [Stream] of [Result<Bytes, std::io::Error>], limiting the bandwidth it can use. \
/// Useful for limiting upload bandwidth.
///
/// bandwidth: maximum bytes per second \
#[pin_project]
pub struct BytesStreamThrottled<R>
where
    R: Stream<Item = Result<Bytes, IoError>>,
{
    #[pin]
    inner: R,
    bandwidth: f32,
    sleep: Pin<Box<Sleep>>,
}

impl<R> BytesStreamThrottled<R>
where
    R: Stream<Item = Result<Bytes, IoError>>,
{
    pub fn wrap(reader: R, bandwidth: usize) -> Self {
        Self {
            inner: reader,
            bandwidth: bandwidth as f32,
            sleep: Box::pin(tokio::time::sleep_until(Instant::now())),
        }
    }
}

impl<R> Stream for BytesStreamThrottled<R>
where
    R: Stream<Item = Result<Bytes, IoError>>,
{
    type Item = Result<Bytes, IoError>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use futures::Future;
        let this = self.project();
        ready!(this.sleep.as_mut().poll(cx));
        let res: Option<Result<Bytes, IoError>> = ready!(this.inner.poll_next(cx));
        if let Some(Ok(bytes)) = &res {
            let read_amount = bytes.len();
            let sleep_duration: f32 = (read_amount as f32) / *this.bandwidth;
            this.sleep
                .as_mut()
                .reset(Instant::now() + Duration::from_secs_f32(sleep_duration));
        }
        Poll::Ready(res)
    }
}

/// Wrap an [AsyncRead] into a [Stream] of [Result<Bytes, IoError>].
pub fn reader_to_stream<R: AsyncRead + Send + Sync + 'static>(
    file: R,
) -> impl Stream<Item = Result<Bytes, IoError>> {
    let stream = FramedRead::new(file, BytesCodec::new()).map_ok(bytes::BytesMut::freeze);
    stream
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use futures::AsyncReadExt;

    use super::*;

    #[tokio::test]
    async fn test_read_hash_at_end() {
        use futures_util::TryStreamExt;
        let content = "hello this is a test".as_bytes();
        let stream = reader_to_stream(content);
        let stream = BytesStreamHashAtEnd::wrap(stream);
        let mut read = stream.into_async_read();
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
        use futures::TryStreamExt;
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests/resources/512bytes.txt");
        let file = tokio::fs::File::open(&path).await.unwrap();
        let stream = reader_to_stream(file);
        let stream = BytesStreamThrottled::wrap(stream, 256);
        let start = Instant::now();
        let _res: Vec<Bytes> = stream.try_collect().await.unwrap();
        let end = Instant::now();
        let elapsed = (end - start).as_secs_f32();
        let expected = 2f32;
        assert!((elapsed - expected).abs() < 0.2);
    }
}
