# Pynk

[![CI](https://github.com/MiguelRiazaValverde/pynk/actions/workflows/CI.yml/badge.svg)](https://github.com/MiguelRiazaValverde/pynk/actions/workflows/CI.yml)
[![Tag](https://img.shields.io/github/v/tag/MiguelRiazaValverde/pynk?label=version)](https://github.com/MiguelRiazaValverde/pynk/tags)
[![npm](https://img.shields.io/npm/v/@pynk/pynk?color=crimson&logo=npm)](https://www.npmjs.com/package/@pynk/pynk)

> **Pynk** is a low-level, minimalistic Node.js package to launch and control the Tor network connection **without requiring a local Tor installation**.  
> It uses [`arti-client`](https://crates.io/crates/arti-client), a Rust implementation of Tor, to connect directly to the Tor network, enabling secure and anonymous HTTP requests from Node.js environments.

## Basic usage

```js
const { TorClient } = require("@pynk/pynk");

(async () => {
  // Target hostname and path
  const hostname = "httpbin.org";
  const path = "/ip";

  // Create a new Tor client with the default configuration
  const client = await TorClient.create();

  // Connect to the target hostname and port (host:port is required)
  const stream = await client.connect(`${hostname}:80`);

  // Create the HTTP GET request
  const request = Buffer.from(
    `GET ${path} HTTP/1.1\r\nHost: ${hostname}\r\nConnection: close\r\n\r\n`,
    "utf8"
  );

  // Send the request
  await stream.write(request);
  await stream.flush(); // Important: make sure all data is sent

  // Read the response
  let response = Buffer.alloc(0);
  while (true) {
    const chunk = await stream.read(4096);
    if (!chunk || chunk.length === 0) break;
    response = Buffer.concat([response, chunk]);
  }

  // Print the full response as a UTF-8 string
  console.log(response.toString("utf8"));
})();
```

```js
const {
  TorClient,
  TorClientBuilder,
  TorClientConfig,
  OnionServiceConfig,
} = require("@pynk/pynk");
const { generateOnionV3 } = require("torv3"); // for generating keys

(async () => {
  // Configure the Tor client with temporary storage
  const torConfig = TorClientConfig.create();
  torConfig.storage.keystore(true);
  // Set the directory where service keys will be stored
  torConfig.storage.stateDir(`./temp/pynk-${Date.now()}`);

  // Create a client using the configuration
  const client = await TorClient.create(TorClientBuilder.create(torConfig));

  // Create a hidden service configuration
  const config = OnionServiceConfig.create();
  // The nickname is used as the username for the hidden service.
  // See directory: ./temp/pynk-*/keystore/hss/my-hidden-service
  config.nickname("my-hidden-service");

  // Generate ed25519 keys for the .onion service
  const keys = generateOnionV3();

  // Create the hidden service using the private key
  const service = client.createOnionServiceWithKey(config, keys.privateKey);
  console.log("Hidden service address:", service.address());

  // Wait for an incoming connection and establish a stream
  // Call service.poll() in a loop to handle each incoming client connection
  service.poll().then(async (rendRequest) => {
    // Accept the client; from now on, it can request new streams to our server
    const streams = await rendRequest.accept();
    // Wait for the next incoming stream request
    const streamRequest = await streams.poll();
    // Accept the new stream
    const stream = await streamRequest.accept();

    // Write and flush data to the client
    await stream.write(Buffer.from("Hello from hidden service!"));
    await stream.flush();
  });

  // Connect to the hidden service as a client
  const clientStream = await client.connect(service.address() + ":80");

  // Read the response from the service
  const messageFromServer = await clientStream.read(4096);

  console.log("Message from server:", messageFromServer.toString("utf8"));
})();
```
