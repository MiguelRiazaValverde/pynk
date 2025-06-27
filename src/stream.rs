use arti_client::DataStream;
use napi::bindgen_prelude::Buffer;
use napi::bindgen_prelude::ObjectFinalize;
use napi::tokio::io::AsyncReadExt;
use napi::tokio::io::AsyncWriteExt;
use rustls::pki_types::ServerName;
use rustls::ClientConfig;
use rustls::RootCertStore;
use std::sync::Arc;
use tokio_rustls::TlsConnector;
use tokio_rustls::TlsStream;
use tokio_util::sync::CancellationToken;

use crate::utils;

enum MaybeTlsStream {
  Plain(DataStream),
  Tls(Box<TlsStream<DataStream>>),
}

impl MaybeTlsStream {
  async fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
    match self {
      MaybeTlsStream::Plain(s) => s.write_all(buf).await,
      MaybeTlsStream::Tls(s) => s.write_all(buf).await,
    }
  }

  async fn flush(&mut self) -> std::io::Result<()> {
    match self {
      MaybeTlsStream::Plain(s) => s.flush().await,
      MaybeTlsStream::Tls(s) => s.flush().await,
    }
  }

  async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    match self {
      MaybeTlsStream::Plain(s) => s.read(buf).await,
      MaybeTlsStream::Tls(s) => s.read(buf).await,
    }
  }
}

#[napi(js_name = "TorStream", custom_finalize)]
pub struct NativeTorStream {
  stream: Option<MaybeTlsStream>,
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
      stream: Some(MaybeTlsStream::Plain(stream)),
      cancel_token: CancellationToken::new(),
    }
  }

  /**
   * Upgrade the stream to use TLS.
   *
   * This wraps the underlying stream in a TLS layer using the provided domain
   * (e.g. "httpbin.org") as the server name for certificate verification (SNI).
   *
   * **Important:** You must call `waitForConnection()` before invoking this method.
   * Upgrading to TLS before the Tor stream is fully established will fail.
   *
   * @throws If the stream is already upgraded to TLS, or the stream is closed, or TLS handshake fails.
   */
  #[napi]
  pub async unsafe fn enable_tls(&mut self, domain: String) -> napi::Result<()> {
    let plain = match self.stream.take() {
      Some(MaybeTlsStream::Plain(s)) => s,
      Some(MaybeTlsStream::Tls(_)) => return Err(napi::Error::from_reason("TLS already enabled")),
      None => return Err(napi::Error::from_reason("Stream closed")),
    };

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
      .with_root_certificates(root_cert_store)
      .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let dnsname = utils::map_error(ServerName::try_from(domain))?;

    let stream = connector.connect(dnsname, plain).await?;
    let stream = TlsStream::Client(stream);

    self.stream = Some(MaybeTlsStream::Tls(Box::new(stream)));
    Ok(())
  }

  /**
   * Wait until a CONNECTED cell is received, or some other cell is received to indicate an error.
   * This must be called before upgrading the stream to TLS using `enableTls()`.
   * Does nothing if this stream is already connected.
   */
  #[napi]
  pub async unsafe fn wait_for_connection(&mut self) -> napi::Result<()> {
    if let Some(MaybeTlsStream::Plain(stream)) = &mut self.stream {
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
