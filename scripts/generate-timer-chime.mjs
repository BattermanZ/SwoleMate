// Generates client/static/timer-done.wav — a short, bright "timer finished"
// chime (an ascending C6–E6–G6 arpeggio with an exponential decay). Mono,
// 44.1 kHz, 16-bit PCM. Re-run with `node scripts/generate-timer-chime.mjs`.
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const SAMPLE_RATE = 44100;
const here = dirname(fileURLToPath(import.meta.url));
const outPath = join(here, '..', 'client', 'static', 'timer-done.wav');

// Three overlapping notes, each starting a little after the previous one.
const notes = [
	{ freq: 1046.5, start: 0.0 }, // C6
	{ freq: 1318.5, start: 0.12 }, // E6
	{ freq: 1567.98, start: 0.24 } // G6
];
const noteDuration = 0.45; // seconds a note keeps ringing
const totalDuration = 0.24 + noteDuration; // last note fully decays
const sampleCount = Math.ceil(totalDuration * SAMPLE_RATE);

const samples = new Float32Array(sampleCount);
for (const { freq, start } of notes) {
	const startSample = Math.floor(start * SAMPLE_RATE);
	for (let i = startSample; i < sampleCount; i++) {
		const t = (i - startSample) / SAMPLE_RATE;
		const envelope = Math.exp(-5 * t); // quick percussive decay
		// fundamental + a soft second harmonic for a bell-like tone
		const tone =
			Math.sin(2 * Math.PI * freq * t) + 0.3 * Math.sin(2 * Math.PI * freq * 2 * t);
		samples[i] += envelope * tone;
	}
}

// Normalise to avoid clipping from overlapping notes.
let peak = 0;
for (const s of samples) peak = Math.max(peak, Math.abs(s));
const gain = peak > 0 ? 0.85 / peak : 1;

const bytesPerSample = 2;
const dataSize = sampleCount * bytesPerSample;
const buffer = Buffer.alloc(44 + dataSize);

buffer.write('RIFF', 0);
buffer.writeUInt32LE(36 + dataSize, 4);
buffer.write('WAVE', 8);
buffer.write('fmt ', 12);
buffer.writeUInt32LE(16, 16); // PCM chunk size
buffer.writeUInt16LE(1, 20); // PCM format
buffer.writeUInt16LE(1, 22); // mono
buffer.writeUInt32LE(SAMPLE_RATE, 24);
buffer.writeUInt32LE(SAMPLE_RATE * bytesPerSample, 28); // byte rate
buffer.writeUInt16LE(bytesPerSample, 32); // block align
buffer.writeUInt16LE(16, 34); // bits per sample
buffer.write('data', 36);
buffer.writeUInt32LE(dataSize, 40);

for (let i = 0; i < sampleCount; i++) {
	const clamped = Math.max(-1, Math.min(1, samples[i] * gain));
	buffer.writeInt16LE(Math.round(clamped * 32767), 44 + i * bytesPerSample);
}

writeFileSync(outPath, buffer);
console.log(`Wrote ${outPath} (${(dataSize / 1024).toFixed(1)} KiB, ${totalDuration.toFixed(2)}s)`);
