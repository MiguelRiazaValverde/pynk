use arti_client::DataStream;
use napi::bindgen_prelude::Buffer;
use napi::bindgen_prelude::ObjectFinalize;
use napi::tokio::io::AsyncReadExt;
use napi::tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use crate::utils;

#[napi(js_name = "TorStream", custom_finalize)]
pub struct NativeTorStream {
  stream: Option<DataStream>,
  cancel_token: CancellationToken,
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
      cancel_token: CancellationToken::new(),
    }
  }

  /**
   * Wait until a CONNECTED cell is received, or some other cell is received to indicate an error.
   * Does nothing if this stream is already connected.
   */
  #[napi]
  pub async unsafe fn wait_for_connection(&mut self) -> napi::Result<()> {
    if let Some(stream) = &mut self.stream {
      utils::map_error(stream.wait_for_connection().await)
    } else {
      Err(napi::Error::from_reason("Stream was closed"))
    }
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
      utils::map_error(stream.write_all(&src).await)
    } else {
      Err(napi::Error::from_reason("Stream was closed"))
    }
  }

  /**
   * Flushes this output stream, ensuring that all intermediately buffered contents reach their destination.
   */
  #[napi]
  pub async unsafe fn flush(&mut self) -> napi::Result<()> {
    if let Some(stream) = &mut self.stream {
      utils::map_error(stream.flush().await)
    } else {
      Err(napi::Error::from_reason("Stream was closed"))
    }
  }

  /**
   * Pulls some bytes from this source into the specified buffer.
   */
  #[napi]
  pub async unsafe fn read(&mut self, len: u32) -> napi::Result<Buffer> {
    let token = self.cancel_token.clone();

    let read_fut = async {
      if let Some(stream) = &mut self.stream {
        let mut buf = vec![0u8; len as usize];
        let n = utils::map_error(stream.read(&mut buf).await)?;
        buf.truncate(n);
        Ok(Buffer::from(buf))
      } else {
        Ok(Buffer::from(vec![]))
      }
    };

    tokio::select! {
      biased;

      _ = token.cancelled() => {
        Err(napi::Error::from_reason("Stream was closed during read"))
      }

      result = read_fut => result
    }
  }

  /**
   * Close the stream.
   */
  #[napi]
  pub unsafe fn close(&mut self) {
    self.stream.take();
    self.cancel_token.cancel();
  }
}

impl ObjectFinalize for NativeTorStream {
  fn finalize(mut self, _env: napi::Env) -> napi::Result<()> {
    unsafe { self.close() };
    Ok(())
  }
}
