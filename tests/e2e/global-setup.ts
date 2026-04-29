import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';

let serverProcess: ChildProcess | null = null;
let dioxusProcess: ChildProcess | null = null;

export default async () => {
  const workspaceRoot = path.resolve(__dirname, '..', '..', '..');

  console.log('Starting reverie server...');

  // Start reverie server in background
  serverProcess = spawn('cargo', ['run', '-p', 'reverie-server'], {
    cwd: workspaceRoot,
    stdio: 'pipe',
    shell: true,
    env: { ...process.env, RUST_LOG: 'warn' }
  });

  serverProcess.stdout?.on('data', (data) => {
    process.stdout.write(`[server] ${data}`);
  });

  serverProcess.stderr?.on('data', (data) => {
    process.stderr.write(`[server] ${data}`);
  });

  // Wait for server to be ready (check /rest/ping)
  const maxWait = 60000; // 60 seconds
  const start = Date.now();
  while (Date.now() - start < maxWait) {
    try {
      const resp = await fetch('http://127.0.0.1:4533/rest/ping?f=json');
      if (resp.ok) {
        console.log('Reverie server ready at http://127.0.0.1:4533');
        break;
      }
    } catch {
      // Not ready yet
    }
    await new Promise(r => setTimeout(r, 1000));
  }

  if (Date.now() - start >= maxWait) {
    throw new Error('Failed to start reverie server within 60 seconds');
  }

  console.log('Starting Dioxus dev server...');

  // Check if Dioxus CLI is available
  try {
    const uiPath = path.join(workspaceRoot, 'reverie-ui');
    dioxusProcess = spawn('dx', ['serve', '--port', '8080'], {
      cwd: uiPath,
      stdio: 'pipe',
      shell: false,
      env: { ...process.env }
    });

    dioxusProcess.stdout?.on('data', (data) => {
      process.stdout.write(`[dioxus] ${data}`);
    });

    dioxusProcess.stderr?.on('data', (data) => {
      process.stderr.write(`[dioxus] ${data}`);
    });

    // Wait for Dioxus dev server to be ready
    const dioxusMaxWait = 120000; // 120 seconds for Dioxus
    const dioxusStart = Date.now();
    while (Date.now() - dioxusStart < dioxusMaxWait) {
      try {
        const resp = await fetch('http://localhost:8080');
        if (resp.ok) {
          console.log('Dioxus dev server ready at http://localhost:8080');
          break;
        }
      } catch {
        // Not ready yet
      }
      await new Promise(r => setTimeout(r, 2000));
    }
  } catch (e) {
    console.warn('Dioxus dev server not available, tests will run against server-only endpoints');
  }

  console.log('Global setup complete');
};

export async function globalTeardown() {
  console.log('Stopping servers...');
  if (serverProcess) {
    serverProcess.kill('SIGTERM');
    await new Promise(r => setTimeout(r, 1000));
  }
  if (dioxusProcess) {
    dioxusProcess.kill('SIGTERM');
    await new Promise(r => setTimeout(r, 1000));
  }
  console.log('Servers stopped');
}
