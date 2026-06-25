import { describe, it, expect } from 'vitest';
import { pngToIco } from '../../scripts/pngToIco.mjs';

/** A stand-in PNG buffer carrying only what the packer reads: the IHDR dims. */
function fakePng(width: number, height: number, fill = 0): Buffer {
	const b = Buffer.alloc(40, fill);
	b.set([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0); // PNG signature
	b.writeUInt32BE(width, 16); // IHDR width
	b.writeUInt32BE(height, 20); // IHDR height
	return b;
}

describe('pngToIco', () => {
	it('writes an ICONDIR header with reserved=0, type=1, and the icon count', () => {
		const ico = pngToIco([fakePng(16, 16), fakePng(32, 32)]);
		expect(ico.readUInt16LE(0)).toBe(0);
		expect(ico.readUInt16LE(2)).toBe(1);
		expect(ico.readUInt16LE(4)).toBe(2);
	});

	it('records each image size, byte length and offset, and embeds the PNG data', () => {
		const a = fakePng(16, 16, 0xaa);
		const b = fakePng(48, 48, 0xbb);
		const ico = pngToIco([a, b]);
		const dirSize = 6 + 16 * 2;

		// Entry 0
		expect(ico.readUInt8(6)).toBe(16); // width
		expect(ico.readUInt8(7)).toBe(16); // height
		expect(ico.readUInt16LE(6 + 4)).toBe(1); // color planes
		expect(ico.readUInt16LE(6 + 6)).toBe(32); // bits per pixel
		expect(ico.readUInt32LE(6 + 8)).toBe(a.length); // bytes in resource
		const off0 = ico.readUInt32LE(6 + 12);
		expect(off0).toBe(dirSize);
		expect(ico.subarray(off0, off0 + a.length).equals(a)).toBe(true);

		// Entry 1, packed immediately after entry 0's data
		const e1 = 6 + 16;
		expect(ico.readUInt8(e1)).toBe(48);
		expect(ico.readUInt32LE(e1 + 8)).toBe(b.length);
		const off1 = ico.readUInt32LE(e1 + 12);
		expect(off1).toBe(dirSize + a.length);
		expect(ico.subarray(off1, off1 + b.length).equals(b)).toBe(true);
	});

	it('encodes a 256px dimension as 0 (per the ICO spec)', () => {
		const ico = pngToIco([fakePng(256, 256)]);
		expect(ico.readUInt8(6)).toBe(0);
		expect(ico.readUInt8(7)).toBe(0);
	});
});
