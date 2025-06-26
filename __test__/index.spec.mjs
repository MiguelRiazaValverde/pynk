import os from 'os';
import test from 'ava';
import * as pathNode from 'path';
import fs from 'fs/promises';
import http from 'http';
import { OnionV3, TorClient, TorClientBuilder, TorClientConfig } from '../index.js';
import { OnionServiceConfig } from '../index.js';
import { TorStream } from '../index.js';

/**
 * Parse a raw HTTP response buffer into status, headers, and body.
 */
function parseHttpResponse(buffer) {
  const response = buffer.toString('utf8');
  const [head, ...bodyParts] = response.split('\r\n\r\n');
  const [statusLine, ...headerLines] = head.split('\r\n');
  const body = bodyParts.join('\r\n\r\n');

  const headers = Object.fromEntries(
    headerLines
      .map(line => line.split(/:\s(.+)/))
      .filter(([k, v]) => k && v)
  );

  const [, statusCodeStr] = statusLine.split(' ');
  const statusCode = parseInt(statusCodeStr, 10);

  return { statusCode, headers, body };
}

/**
 * Perform a raw HTTP request through Tor.
 */
async function torHttpRequest(hostname, path = '/') {
  const cacheDir = pathNode.join(os.tmpdir(), `pynk-${Date.now()}-${Math.random()}`);
  await fs.mkdir(cacheDir, { recursive: true });
  const conf = TorClientConfig.create();
  conf.storage.cacheDir(cacheDir);
  const builder = TorClientBuilder.create(conf);
  const client = await TorClient.create(builder);
  const stream = await client.connect(`${hostname}:80`);

  const request = Buffer.from(
    `GET ${path} HTTP/1.1\r\nHost: ${hostname}\r\nConnection: close\r\n\r\n`,
    'utf8'
  );

  await stream.write(request);
  await stream.flush();

  let response = Buffer.alloc(0);
  while (true) {
    const chunk = await stream.read(4096);
    if (!chunk || chunk.length === 0) break;
    response = Buffer.concat([response, chunk]);
  }

  return parseHttpResponse(response);
}

/**
 * Perform a direct HTTP request using Node's http module.
 */
function directHttpRequest(hostname, path = '/') {
  return new Promise((resolve, reject) => {
    const req = http.request(
      {
        hostname,
        port: 80,
        path,
        method: 'GET',
        headers: {
          Connection: 'close',
          Host: hostname,
        },
      },
      res => {
        let data = '';
        res.setEncoding('utf8');
        res.on('data', chunk => (data += chunk));
        res.on('end', () => {
          resolve({
            statusCode: res.statusCode,
            headers: res.headers,
            body: data,
          });
        });
      }
    );
    req.on('error', reject);
    req.end();
  });
}

test('Tor request returns HTTP 200 OK', async t => {
  const response = await torHttpRequest('httpbin.org', '/ip');
  t.is(response.statusCode, 200);
});

test('Tor IP and direct IP differ', async t => {
  const torResponse = await torHttpRequest('httpbin.org', '/ip');
  const directResponse = await directHttpRequest('httpbin.org', '/ip');

  t.is(torResponse.statusCode, 200);
  t.is(directResponse.statusCode, 200);

  const torIp = JSON.parse(torResponse.body).origin;
  const directIp = JSON.parse(directResponse.body).origin;

  t.not(torIp, directIp);
});

test('Tor can access .onion site (DuckDuckGo)', async t => {
  const response = await torHttpRequest(
    'duckduckgogg42xjoc72x3sjasowoarfbgcmvfimaftt6twagswzczad.onion'
  );

  t.true(
    response.body.includes('DuckDuckGo') || response.body.includes('<html'),
    'Response should contain HTML'
  );
});

test('Hidden service', async t => {
  const torConfig = TorClientConfig.create();
  torConfig.storage.keystore(true);
  const client = await TorClient.create(TorClientBuilder.create(torConfig));
  const config = OnionServiceConfig.create();
  config.nickname(`nickname-${Math.floor(Math.random() * 10000)}`);

  const keys = new OnionV3();
  const service = client.createOnionServiceWithKey(config, keys.getSecret());
  await service.waitRunning();

  const [serverStream, clientStream] = await Promise.all([
    service.poll().then(async rendRequest => {
      const streams = await rendRequest.accept();
      const streamRequest = await streams.poll();
      const stream = await streamRequest.accept();
      return stream;
    }),
    client.connect(service.address() + ":80")
  ]);

  t.is(service.address(), keys.address, "service address should match expected onion URL");
  t.truthy(serverStream instanceof TorStream, "serverStream should be an instance of TorStream");
  t.truthy(clientStream instanceof TorStream, "clientStream should be an instance of TorStream");
});

test('Onion v3', async t => {
  const dir = new OnionV3();
  t.true(dir.address.endsWith('.onion'), 'Address should end with .onion');
  t.is(dir.getSecret().length, 32, 'Private key buffer should be 32 bytes');
  t.is(dir.getPublic().length, 32, 'Public key buffer should be 32 bytes');

  const prefix = 'pynk';
  const dirAsync = await OnionV3.generateVanityAsync(prefix);

  t.true(dirAsync.address.startsWith(prefix), `Vanity address should start with '${prefix}'`);
  t.true(dirAsync.address.endsWith('.onion'), 'Address should end with .onion');

  const dirFromPrivate = OnionV3.fromSecret(dirAsync.getSecret());

  t.deepEqual(dirAsync.getSecret(), dirFromPrivate.getSecret(), 'Private keys should match');
  t.deepEqual(dirAsync.getPublic(), dirFromPrivate.getPublic(), 'Public keys should match');
  t.is(dirAsync.address, dirFromPrivate.address, 'Addresses should match');
});

test('Closed stream', async t => {
  const torConfig = TorClientConfig.create();
  torConfig.storage.keystore(true);
  const client = await TorClient.create(TorClientBuilder.create(torConfig));
  const config = OnionServiceConfig.create();
  config.nickname(`nickname-${Math.floor(Math.random() * 10000)}`);

  const keys = new OnionV3();
  const service = client.createOnionServiceWithKey(config, keys.getSecret());
  await service.waitRunning();

  const [serverStream, clientStream] = await Promise.all([
    service.poll().then(async rendRequest => {
      const streams = await rendRequest.accept();
      const streamRequest = await streams.poll();
      const stream = await streamRequest.accept();
      return stream;
    }),
    client.connect(service.address() + ":80")
  ]);

  await serverStream.write(Buffer.from("Hello!"));
  await serverStream.flush();
  await clientStream.read(4096);

  serverStream.close();

  await t.throwsAsync(
    () => clientStream.read(4096),
    { instanceOf: Error, code: 'GenericFailure', message: /end cell/i },
    'The stream should throw an error after being closed on the server side'
  );
});

test('Closed hidden service', async t => {
  const torConfig = TorClientConfig.create();
  torConfig.storage.keystore(true);
  const client = await TorClient.create(TorClientBuilder.create(torConfig));
  const config = OnionServiceConfig.create();
  config.nickname(`nickname-${Math.floor(Math.random() * 10000)}`);

  const keys = new OnionV3();
  const service = client.createOnionServiceWithKey(config, keys.getSecret());
  await service.waitRunning();

  const promise = t.throwsAsync(
    () => service.poll().then(async _ => { }),
    { instanceOf: Error, code: 'GenericFailure', message: /Hidden service was closed/i },
    'The hidden service poll should throw an error after being closed'
  );

  service.close();
  await promise;
});
