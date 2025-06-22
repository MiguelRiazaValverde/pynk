import os from 'os';
import test from 'ava';
import * as pathNode from 'path';
import fs from 'fs/promises';
import http from 'http';
import { TorClient, TorClientBuilder, TorClientConfig } from '../index.js';

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
  const cacheDir = pathNode.join(os.tmpdir(), `arti-cache-${Date.now()}-${Math.random()}`);
  await fs.mkdir(cacheDir, { recursive: true });
  const conf = TorClientConfig.create();
  conf.storage.cacheDir(cacheDir);
  const builder = TorClientBuilder.create(conf);
  const client = await TorClient.create();
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
