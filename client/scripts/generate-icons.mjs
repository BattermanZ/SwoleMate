import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { Resvg } from '@resvg/resvg-js';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '..');
const svgPath = resolve(root, 'static/logo.svg');
const svg = readFileSync(svgPath, 'utf8');

const targets = [
	{ out: 'favicon.png', size: 64 },
	{ out: 'pwa-192.png', size: 192 },
	{ out: 'pwa-512.png', size: 512 },
	{ out: 'apple-touch-icon.png', size: 192 },
	{ out: 'apple-touch-icon-precomposed.png', size: 192 }
];

for (const { out, size } of targets) {
	const png = new Resvg(svg, {
		fitTo: { mode: 'width', value: size },
		font: { loadSystemFonts: true, defaultFontFamily: 'sans-serif' }
	})
		.render()
		.asPng();
	const path = resolve(root, 'static', out);
	writeFileSync(path, png);
	console.log(`wrote ${out} (${size}x${size}, ${png.length} bytes)`);
}
