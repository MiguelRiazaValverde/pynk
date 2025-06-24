use arti_client::DataStream;
use napi::bindgen_prelude::Buffer;
use napi::tokio::io::AsyncReadExt;
use napi::tokio::io::AsyncWriteExt;

use crate::utils;

#[napi(js_name = "TorStream")]
pub struct NativeTorStream {
  stream: Option<DataStream>,
}

#[napi]
impl NativeTorStream {
  /**
   * This class cannot be constructed manually.
   */
  #[napi(constructor)]
  pub fn new() -> napi::Result<NativeTorStream> {
    Err(napi::Error::new(
      napi::Status::GenericFailure,
      "This class cannot be constructed manually.".to_string(),
    ))
  }

  pub fn from_stream(stream: DataStream) -> Self {
    Self {
      stream: Some(stream),
    }
  }

  /**
   * Wait until a CONNECTED cell is received, or some other cell is received to indicate an error.
   * Does nothing if this stream is already connected.
   */
  #[napi]
  pub async unsafe fn wait_for_connection(&mut self) -> napi::Result<()> {
    if let Some(stream) = &mut self.stream {
      utils::map_error(stream.wait_for_connection().await)?;
    }
    Ok(())
  }

  /**
   * Attempts to write an entire buffer into this writer.
   * @example
   * ```ts
   * const client = await TorClient.create();
   * const stream = await client.connect("httpbin.org:80");
   * const request =
   *   "GET / HTTP/1.1\r\n" +
   *   "Host: httpbin.org\r\n" +
   *   "Connection: close\r\n\r\n";
   * await stream.write(Buffer.from(request));
   * await stream.flush();
   * ```
   */
  #[napi]
  pub async unsafe fn write(&mut self, src: Buffer) -> napi::Result<()> {
    if let Some(stream) = &mut self.stream {
      utils::map_error(stream.write_all(&src).await)?;
    }
    Ok(())
  }

  /**
   * Flushes this output stream, ensuring that all intermediately buffered contents reach their destination.
   */
  #[napi]
  pub async unsafe fn flush(&mut self) -> napi::Result<()> {
    if let Some(stream) = &mut self.stream {
      utils::map_error(stream.flush().await)?;
    }
    Ok(())
  }

  /**
   * Pulls some bytes from this source into the specified buffer.
   */
  #[napi]
  pub async unsafe fn read(&mut self, len: u32) -> napi::Result<Buffer> {
    let buf = if let Some(stream) = &mut self.stream {
      let mut buf = vec![0u8; len as usize];
      let n = utils::map_error(stream.read(&mut buf).await)?;
      buf.truncate(n);
      buf
    } else {
      vec![]
    };
    Ok(Buffer::from(buf))
  }

  /**
   * Close the stream.
   */
  #[napi]
  pub async unsafe fn close(&mut self) {
    self.stream.take();
  }
}
