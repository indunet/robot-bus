import { build } from 'esbuild';
import { copyFile, mkdir } from 'node:fs/promises';

await mkdir('dist', { recursive: true });
await build({ entryPoints: ['web/main.ts'], bundle: true, platform: 'browser',
  target: 'es2022', outfile: 'dist/main.js', sourcemap: true });
await copyFile('web/index.html', 'dist/index.html');
