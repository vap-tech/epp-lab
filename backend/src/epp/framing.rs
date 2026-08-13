use std::{io, time::Duration};

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const HEADER_SIZE: usize = 4;
pub const MIN_FRAME_SIZE: u32 = 5;

#[derive(Clone, Copy, Debug)]
pub struct FrameLimits {
    pub max_frame_size: u32,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
}

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("failed to read frame header: {0}")]
    Header(#[source] io::Error),
    #[error("failed to read frame body: {0}")]
    Body(#[source] io::Error),
    #[error("failed to write frame: {0}")]
    Write(#[source] io::Error),
    #[error("frame length {0} is smaller than the EPP minimum of {MIN_FRAME_SIZE}")]
    TooShort(u32),
    #[error("frame length {length} exceeds maximum {max}")]
    TooLarge { length: u32, max: u32 },
    #[error("frame payload is too large")]
    LengthOverflow,
    #[error("frame operation timed out")]
    Timeout,
}

pub async fn read_frame<R>(reader: &mut R, limits: &FrameLimits) -> Result<Vec<u8>, FrameError>
where
    R: AsyncRead + Unpin,
{
    let mut header = [0; HEADER_SIZE];
    tokio::time::timeout(limits.read_timeout, reader.read_exact(&mut header))
        .await
        .map_err(|_| FrameError::Timeout)?
        .map_err(FrameError::Header)?;

    let length = u32::from_be_bytes(header);
    if length < MIN_FRAME_SIZE {
        return Err(FrameError::TooShort(length));
    }
    if length > limits.max_frame_size {
        return Err(FrameError::TooLarge {
            length,
            max: limits.max_frame_size,
        });
    }

    let payload_length = (length - HEADER_SIZE as u32) as usize;
    let mut payload = vec![0; payload_length];
    tokio::time::timeout(limits.read_timeout, reader.read_exact(&mut payload))
        .await
        .map_err(|_| FrameError::Timeout)?
        .map_err(FrameError::Body)?;
    Ok(payload)
}

pub async fn write_frame<W>(
    writer: &mut W,
    payload: &[u8],
    limits: &FrameLimits,
) -> Result<(), FrameError>
where
    W: AsyncWrite + Unpin,
{
    let length = payload
        .len()
        .checked_add(HEADER_SIZE)
        .and_then(|length| u32::try_from(length).ok())
        .ok_or(FrameError::LengthOverflow)?;
    if length < MIN_FRAME_SIZE {
        return Err(FrameError::TooShort(length));
    }
    if length > limits.max_frame_size {
        return Err(FrameError::TooLarge {
            length,
            max: limits.max_frame_size,
        });
    }

    tokio::time::timeout(limits.write_timeout, async {
        writer
            .write_all(&length.to_be_bytes())
            .await
            .map_err(FrameError::Write)?;
        writer.write_all(payload).await.map_err(FrameError::Write)
    })
    .await
    .map_err(|_| FrameError::Timeout)?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, duplex};

    fn limits() -> FrameLimits {
        FrameLimits {
            max_frame_size: 1024,
            read_timeout: Duration::from_millis(100),
            write_timeout: Duration::from_millis(100),
        }
    }

    #[tokio::test]
    async fn reads_valid_frame() {
        let (mut client, mut server) = duplex(32);
        client
            .write_all(&[0, 0, 0, 7, b'h', b'e', b'l'])
            .await
            .unwrap();
        assert_eq!(read_frame(&mut server, &limits()).await.unwrap(), b"hel");
    }

    #[tokio::test]
    async fn writes_valid_frame() {
        let (mut client, mut server) = duplex(32);
        write_frame(&mut client, b"hello", &limits()).await.unwrap();
        let mut bytes = [0; 9];
        server.read_exact(&mut bytes).await.unwrap();
        assert_eq!(&bytes, b"\0\0\0\thello");
    }

    #[tokio::test]
    async fn rejects_short_frame() {
        let (mut client, mut server) = duplex(32);
        client.write_all(&[0, 0, 0, 4]).await.unwrap();
        assert!(matches!(
            read_frame(&mut server, &limits()).await,
            Err(FrameError::TooShort(4))
        ));
    }

    #[tokio::test]
    async fn rejects_oversized_frame() {
        let (mut client, mut server) = duplex(32);
        client.write_all(&[0, 0, 4, 1]).await.unwrap();
        assert!(matches!(
            read_frame(&mut server, &limits()).await,
            Err(FrameError::TooLarge { .. })
        ));
    }

    #[tokio::test]
    async fn times_out_on_partial_body() {
        let (mut client, mut server) = duplex(32);
        client.write_all(&[0, 0, 0, 7, b'h']).await.unwrap();
        assert!(matches!(
            read_frame(&mut server, &limits()).await,
            Err(FrameError::Timeout)
        ));
    }
}
