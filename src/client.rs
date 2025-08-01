use crate::client_builder::NativeTorClientBuilder;
use crate::hs_config::NativeOnionServiceConfig;
use crate::hs_service::NativeOnionService;
use crate::stream::NativeTorStream;
use crate::stream_prefs::NativeStreamPrefs;
use crate::utils;
use arti_client::TorClient;
use napi::JsBuffer;
use tor_hscrypto::pk::HsIdKeypair;
use tor_llcrypto::pk::ed25519::{ExpandedKeypair, Keypair};
use tor_rtcompat::PreferredRuntime;

#[napi(js_name = "TorClient")]
pub struct NativeTorClient {
  client: TorClient<PreferredRuntime>,
}

#[napi]
impl NativeTorClient {
  pub fn from_client(client: TorClient<PreferredRuntime>) -> Self {
    Self { client }
  }

  /**
   * Creates a new instance of the Tor client.
   *
   * If a builder is provided, it will be used to configure and build the client.
   * Otherwise, a default builder will be used.
   *
   * @param builder - Optional reference to a `NativeTorClientBuilder` to customize the client configuration.
   *
   * @returns A new `NativeTorClient` instance wrapped in a Promise.
   */
  #[napi(factory)]
  pub async fn create(builder: Option<&NativeTorClientBuilder>) -> napi::Result<Self> {
    let client = if let Some(builder) = builder {
      builder.build().await
    } else {
      NativeTorClientBuilder::default().build().await
    };

    let client = utils::map_error(client)?;
    Ok(Self { client })
  }

  /**
   * Return a new isolated TorClient handle.
   * The two TorClients will share internal state and configuration, but their streams will never share circuits with one another.
   * Use this function when you want separate parts of your program to each have a TorClient handle, but where you don't want their activities to be linkable to one another over the Tor network.
   * Calling this function is usually preferable to creating a completely separate TorClient instance, since it can share its internals with the existing TorClient.
   * Connections made with clones of the returned TorClient may share circuits with each other.)
   */
  #[napi]
  pub fn isolated(&self) -> Self {
    Self::from_client(self.client.isolated_client())
  }

  /**
   * Launch an anonymized connection to the provided address and port over the Tor network.
   * Note that because Tor prefers to do DNS resolution on the remote side of the network, this function takes its address as a string:
   *
   *  @param address - The target address and port as a string, **important:** it must be in the format `url:port` (e.g. `"httpbin.org:80"`).
   *
   * @example
   * ```ts
   * const client = await TorClient.create();
   * const stream = await client.connect("httpbin.org:80");
   *
   * // It is recommended to wait for the connection to be fully established
   * // by calling `waitForConnection()` after `connect()`.
   * await stream.waitForConnection();
   * ```
   */
  #[napi]
  pub async fn connect(&self, address: String) -> napi::Result<NativeTorStream> {
    let stream = self.client.connect(&address).await;
    let stream = utils::map_error(stream)?;
    Ok(NativeTorStream::from_stream(stream))
  }

  /**
   * Sets the default preferences for future connections made with this client.
   * The preferences set with this function will be inherited by clones of this client, but updates to the preferences in those clones will not propagate back to the original. I.e., the preferences are copied by clone.
   * Connection preferences always override configuration, even configuration set later (eg, by a config reload).
   */
  #[napi]
  pub fn set_stream_prefs(&mut self, stream_prefs: &NativeStreamPrefs) -> &Self {
    self.client.set_stream_prefs(stream_prefs.get());
    self
  }

  /**
   * Creates and returns a new hidden service.
   */
  #[napi]
  pub fn create_onion_service(
    &self,
    onion_service_config: &NativeOnionServiceConfig,
  ) -> napi::Result<NativeOnionService> {
    let (service, rend_request) = utils::map_error(
      self
        .client
        .launch_onion_service(utils::map_error(onion_service_config.build())?),
    )?;
    Ok(NativeOnionService::from_service(service, rend_request))
  }

  /**
   * Creates a new hidden service using a provided private key.
   * The key format must have the private key in the first 32 bytes.
   */
  #[napi]
  pub fn create_onion_service_with_key(
    &self,
    onion_service_config: &NativeOnionServiceConfig,
    bytes: JsBuffer,
  ) -> napi::Result<NativeOnionService> {
    let bytes_value = bytes.into_value().unwrap();
    let slice = bytes_value.as_ref();

    if slice.len() < 32 {
      return Err(napi::Error::from_reason::<String>(
        "Expected 32 bytes for the key".into(),
      ));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&slice[0..32]);

    let secret: [u8; 32] = utils::map_error(key_bytes[0..32].try_into())?;
    let kay_pair = Keypair::from_bytes(&secret);
    let expanded = ExpandedKeypair::from(&kay_pair);

    let hsid_keypair = HsIdKeypair::from(expanded);

    let (service, rend_request) = utils::map_error(self.client.launch_onion_service_with_hsid(
      utils::map_error(onion_service_config.build())?,
      hsid_keypair,
    ))?;

    Ok(NativeOnionService::from_service(service, rend_request))
  }
}
