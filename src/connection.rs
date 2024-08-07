use mini_redis::frame::Error::Incomplete;
use mini_redis::Frame;
use crate::Result;
use std::io::Cursor;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use bytes::{BytesMut, Buf};

pub struct Connection {
    stream: BufWriter<TcpStream>, // use BufWriter to optimize write performance
    // buffer: Vec<u8>,
    // cursor: usize,
    buffer: BytesMut,
}

impl Connection {
    /// create a new connection
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            // buffer: vec![0; 4 * 1024],
            // cursor: 0,
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    /// read
    /// return Some(Frame) if a frame is parsed
    /// or None if the connection is closed
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            // Attempt to parse a frame from the buffered data.
            // If enough data has been buffered, the frame is returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            // Dead code: use vector and cursor instead of BytesMut
            // // make sure there is capacity to read more data
            // // grow the buffer if not
            // if self.buffer.len() == self.cursor {
            //     self.buffer.resize(self.cursor * 2, 0);
            // }

            // // read data from the socket
            // // and write it to the buffer starting at the current cursor
            // let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

            // if 0 == n {
            //     // the remote closed the connection
            //     if self.cursor == 0 {
            //         return Ok(None);
            //     } else {
            //         return Err("connection reset by peer".into());
            //     }
            // } else {
            //     self.cursor += n;
            // }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    /// write frame into connection
    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Array(frames) => {
                self.stream.write_all(b"*").await?;
                self.stream
                    .write_all(frames.len().to_string().as_bytes())
                    .await?;
                self.stream.write_all(b"\r\n").await?;
                for frame in frames {
                    self.write_single_frame(frame).await?;
                }
            }
            _ => self.write_single_frame(frame).await?,
        }

        self.stream.flush().await
    }

    /// write single frame into connection
    async fn write_single_frame(&mut self, frame: &Frame) -> io::Result<()> {
        match frame {
            Frame::Simple(s) => {
                self.stream.write_all(b"+").await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(e) => {
                self.stream.write_all(b"-").await?;
                self.stream.write_all(e.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(i) => {
                self.stream.write_all(b":").await?;
                self.stream.write_all(i.to_string().as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(data) => {
                self.stream.write_all(b"$").await?;
                self.stream
                    .write_all(data.len().to_string().as_bytes())
                    .await?;
                self.stream.write_all(b"\r\n").await?;
                self.stream.write_all(data).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            _ => {
                unreachable!();
            }
        }

        Ok(())
    }

    /// parse frame from buffer
    /// return Some(Frame) if a frame is parsed
    /// or None if the buffer does not contain a complete frame
    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // Create a new Parser with the buffer
        let mut buf = Cursor::new(&self.buffer[..]);

        // check if a frame can be parsed
        match Frame::check(&mut buf) {
            Ok(_) => {
                // Update the cursor based on how many bytes were parsed
                let len = buf.position() as usize;
                buf.set_position(0);

                let frame = Frame::parse(&mut buf)?;
                // discard the frame from the buffer
                // self.cursor -= len;
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
