///! Different `Read` wrappers, for use with b2_upload_file

use std::io::Read;
use sha1::Sha1;
use std::cmp::min;

// When the main reader is done, we start returning from the hash
// The 'hash_read' is used to keep track of how much we've read of the hash
pub struct ReadHashAtEnd<R: Read> {
    inner: R,
    hash: Sha1,
    hash_read: usize,
}

impl<R: Read> ReadHashAtEnd<R> {
    pub fn wrap(reader: R) -> Self {
        ReadHashAtEnd {
            inner: reader,
            hash: Sha1::new(),
            hash_read: 0,
        }
    }
}

impl<R: Read> Read for ReadHashAtEnd<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let read_inner = self.inner.read(buf);
        let read_inner = read_inner?;

        // 1. The main reader is done and we should start returning the hash OR
        // 2. We're in the middle of sending the hash and should continue doing so
        if read_inner == 0 && self.hash_read == 0 || self.hash_read > 0 && self.hash_read < 40 {
            let digest = self.hash.hexdigest();
            let digest = digest.as_bytes();
            let buf_size = buf.len();

            // 1. If the buffer isn't large enough for our hash, we need to split it up OR
            // 2. If we've already sent part of the hash, we need to only return the remainder
            if buf_size < 40 || self.hash_read > 0 {
                // Determine how much we can return (in case the buffer is too small)
                let end_index = min(40, self.hash_read+buf_size);
                let read_amount = end_index-self.hash_read;

                buf[0..read_amount].clone_from_slice(&digest[self.hash_read..end_index]);
                self.hash_read += read_amount;
                Ok(read_amount)
            } else {
                // If there's enough room, just send the 40-byte hexdigest directly
                buf[0..40].clone_from_slice(&digest);
                self.hash_read = 40;
                Ok(40)
            }
        } else if read_inner == 0 || self.hash_read == 40 {
            // We read nothing and we have added the hash - We're done, return 0
            Ok(0)
        } else {
            // We read something from the inner reader, update the hash
            self.hash.update(&buf[0..read_inner]);
            Ok(read_inner)
        }
    }
}