import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';

const proxyTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://127.0.0.1:2469';
const backendProxy = {
	target: proxyTarget,
	changeOrigin: true
};

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	resolve: process.env.VITEST
		? {
				alias: [
					{
						find: /^svelte$/,
						replacement: fileURLToPath(
							new URL('./node_modules/svelte/src/index-client.js', import.meta.url)
						)
					}
				]
			}
		: undefined,
	test: {
		environment: 'jsdom',
		setupFiles: ['./src/test/setup.ts'],
		include: ['src/**/*.test.{js,ts}', 'src/**/*.spec.{js,ts}'],
		clearMocks: true,
		restoreMocks: true
	},
	server: {
		host: true,
		allowedHosts: true,
		port: 2470,
		proxy: {
			'/api': backendProxy,
			'/mcp': backendProxy,
			'/oauth': backendProxy,
			'/.well-known': backendProxy
		}
	},
	preview: {
		host: true,
		port: 2470,
		proxy: {
			'/api': backendProxy,
			'/mcp': backendProxy,
			'/oauth': backendProxy,
			'/.well-known': backendProxy
		}
	}
});
