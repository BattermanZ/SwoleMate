import { writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import * as fontkit from 'fontkit';
import { Resvg } from '@resvg/resvg-js';
import { pngToIco } from './pngToIco.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '..');
const fontPath = resolve(here, 'fonts/Onest-ExtraBold.ttf');
const svgOut = resolve(root, 'static/logo.svg');

// Load Onest variable font and instance at weight 800.
const font = fontkit.openSync(fontPath);
const onest800 = font.getVariation({ wght: 800 });

const TEXT = 'SM';
const TARGET_FONT_PX = 220; // matches in-app Logo at this tile size
const VIEWBOX = 512;
const LETTER_TRACKING = -6; // matches Logo's letter-spacing: -0.02em ≈ -6 at fontSize 220

function glyphsToPath(run) {
	const scale = TARGET_FONT_PX / onest800.unitsPerEm;
	let advance = 0;
	const segs = [];
	for (let i = 0; i < run.glyphs.length; i++) {
		const glyph = run.glyphs[i];
		const pos = run.positions[i];
		const dx = (advance + pos.xOffset) * scale;
		const dy = pos.yOffset * scale;
		const path = glyph.path.scale(scale, -scale).translate(dx, dy);
		segs.push(path.toSVG());
		advance += pos.xAdvance + LETTER_TRACKING / scale;
	}
	return { d: segs.join(' '), width: advance * scale };
}

const run = onest800.layout(TEXT);
const { d, width } = glyphsToPath(run);
const ascent = onest800.ascent * (TARGET_FONT_PX / onest800.unitsPerEm);
const descent = -onest800.descent * (TARGET_FONT_PX / onest800.unitsPerEm);
const lineHeight = ascent + descent;
const x = (VIEWBOX - width) / 2;
const y = (VIEWBOX + lineHeight) / 2 - descent;

const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${VIEWBOX} ${VIEWBOX}" role="img" aria-label="SwoleMate">
  <defs>
    <linearGradient id="clay" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#ff7a2a"/>
      <stop offset="100%" stop-color="#ff5e1f"/>
    </linearGradient>
  </defs>
  <rect width="${VIEWBOX}" height="${VIEWBOX}" rx="118" ry="118" fill="url(#clay)"/>
  <g transform="translate(${x.toFixed(2)} ${y.toFixed(2)})" fill="#ffffff">
    <path d="${d}"/>
  </g>
</svg>
`;
writeFileSync(svgOut, svg);
console.log(`wrote logo.svg (${svg.length} bytes)`);

const targets = [
	{ out: 'favicon.png', size: 64 },
	{ out: 'pwa-192.png', size: 192 },
	{ out: 'pwa-512.png', size: 512 },
	{ out: 'apple-touch-icon.png', size: 192 },
	{ out: 'apple-touch-icon-precomposed.png', size: 192 }
];

const pngBuffers = {};
for (const { out, size } of targets) {
	const png = new Resvg(svg, { fitTo: { mode: 'width', value: size } }).render().asPng();
	writeFileSync(resolve(root, 'static', out), png);
	pngBuffers[size] = png;
	console.log(`wrote ${out} (${size}x${size}, ${png.length} bytes)`);
}

// Build favicon.ico with 16, 32, and 48px layers
const icoSizes = [16, 32, 48];
const icoPngs = icoSizes.map((size) =>
	Buffer.from(new Resvg(svg, { fitTo: { mode: 'width', value: size } }).render().asPng())
);
const ico = pngToIco(icoPngs);
writeFileSync(resolve(root, 'static/favicon.ico'), ico);
console.log(`wrote favicon.ico (${icoSizes.join('/')}px, ${ico.length} bytes)`);
