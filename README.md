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
