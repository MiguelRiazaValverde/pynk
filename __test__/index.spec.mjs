import os from 'os';
import test from 'ava';
import * as pathNode from 'path';
import fs from 'fs/promises';
import http from 'http';
import { TorClient, TorClientBuilder, TorClientConfig } from '../index.js';
import { OnionServiceConfig } from '../index.js';

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

  console.log('Tor IP:', torIp);
  console.log('Direct IP:', directIp);

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
  torConfig.storage.stateDir("./tor");
  const client = await TorClient.create(TorClientBuilder.create(torConfig));
  const config = OnionServiceConfig.create();
  config.nickname("30301");
  const service = client.createOnionService(config, Buffer.from(Uint8Array.from([
    138, 210, 223, 48, 194, 21, 181, 91, 28, 80, 87,
    180, 145, 180, 73, 216, 229, 62, 49, 219, 33, 166,
    26, 239, 226, 120, 199, 12, 111, 82, 73, 8, 155,
    162, 169, 164, 17, 202, 168, 209, 92, 253, 125, 71,
    66, 109, 66, 189, 239, 115, 3, 50, 14, 107, 184,
    46, 142, 42, 128, 109, 172, 225, 242, 136
  ])));
  console.log(service.address());
  t.is(1, 1);
});
