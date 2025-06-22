use crate::config::NativeTorClientConfig;
use crate::utils;
use arti_client::TorClient;
use arti_client::TorClientBuilder;
use tor_rtcompat::PreferredRuntime;

#[napi(js_name = "TorClientBuilder")]
pub struct NativeTorClientBuilder {
  builder: TorClientBuilder<PreferredRuntime>,
}

impl Default for NativeTorClientBuilder {
  fn default() -> Self {
    Self {
      builder: TorClient::builder(),
    }
  }
}

#[napi]
impl NativeTorClientBuilder {
  /**
   * Constructs a new `TorClientBuilder` with an optional configuration.
   *
   * If a configuration is provided, it will be used to initialize the builder.
   * Otherwise, the default configuration will be applied.
   *
   * @param config - Optional configuration object to customize the Tor client builder.
   * @returns A new instance of `NativeTorClientBuilder`.
   */
  #[napi(constructor)]
  pub fn new(config: Option<&NativeTorClientConfig>) -> napi::Result<Self> {
    let config = config
      .map(|c| c.build())
      .unwrap_or_else(|| NativeTorClientConfig::default().build());

    let config = utils::map_error(config)?;

    Ok(Self {
      builder: TorClient::builder().config(config),
    })
  }

  /**
   * Constructs a new `TorClientBuilder` with an optional configuration.
   *
   * If a configuration is provided, it will be used to initialize the builder.
   * Otherwise, the default configuration will be applied.
   *
   * @param config - Optional configuration object to customize the Tor client builder.
   * @returns A new instance of `NativeTorClientBuilder`.
   */
  #[napi(factory)]
  pub fn create(config: Option<&NativeTorClientConfig>) -> napi::Result<Self> {
    Self::new(config)
  }

  /**
   * Set the configuration for the TorClient under construction.
   * If not called, then a compiled-in default configuration will be used.
   */
  #[napi]
  pub fn config(&mut self, config: &NativeTorClientConfig) -> napi::Result<&Self> {
    let config = utils::map_error(config.build())?;
    self.builder = self.builder.clone().config(config);
    Ok(self)
  }

  pub async fn build(&self) -> Result<TorClient<PreferredRuntime>, arti_client::Error> {
    self.builder.create_bootstrapped().await
  }
}
