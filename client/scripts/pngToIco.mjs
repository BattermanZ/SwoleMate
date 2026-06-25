/**
 * Pack PNG buffers into a Windows ICO file (PNG-encoded entries, as supported by
 * all modern browsers). Replaces the deprecated `to-ico` dependency — the ICO
 * container is just a header, a directory, and the embedded PNG blobs.
 *
 * @param {Buffer[]} pngs - PNG buffers, each carrying its dimensions in its IHDR.
 * @returns {Buffer} the .ico file contents
 */
export function pngToIco(pngs) {
	const HEADER_SIZE = 6;
	const ENTRY_SIZE = 16;
	const dirSize = HEADER_SIZE + ENTRY_SIZE * pngs.length;

	const header = Buffer.alloc(HEADER_SIZE);
	header.writeUInt16LE(0, 0); // reserved, always 0
	header.writeUInt16LE(1, 2); // image type: 1 = icon
	header.writeUInt16LE(pngs.length, 4); // number of images

	const entries = [];
	let offset = dirSize;
	for (const png of pngs) {
		const width = png.readUInt32BE(16); // IHDR width
		const height = png.readUInt32BE(20); // IHDR height

		const entry = Buffer.alloc(ENTRY_SIZE);
		entry.writeUInt8(width >= 256 ? 0 : width, 0); // 0 means 256
		entry.writeUInt8(height >= 256 ? 0 : height, 1);
		entry.writeUInt8(0, 2); // palette colour count (0 for non-paletted)
		entry.writeUInt8(0, 3); // reserved
		entry.writeUInt16LE(1, 4); // colour planes
		entry.writeUInt16LE(32, 6); // bits per pixel
		entry.writeUInt32LE(png.length, 8); // size of the PNG data
		entry.writeUInt32LE(offset, 12); // offset of the PNG data
		entries.push(entry);
		offset += png.length;
	}

	return Buffer.concat([header, ...entries, ...pngs]);
}
