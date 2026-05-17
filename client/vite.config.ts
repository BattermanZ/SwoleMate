import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import { fileURLToPath } from 'node:url';

const proxyTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://127.0.0.1:2469';
const backendProxy = {
	target: proxyTarget,
	changeOrigin: true
};
const contentSecurityPolicy = [
	"default-src 'self'",
	"base-uri 'self'",
	"frame-ancestors 'none'",
	"object-src 'none'",
	"form-action 'self'",
	"script-src 'self' 'unsafe-eval' 'sha256-XxsyBzghD6XiU6EbSkMzj+Ob6G4ncvvkwDcYat6QTXA='",
	"style-src 'self' 'unsafe-inline'",
	"img-src 'self' data: blob: https:",
	"font-src 'self' data:",
	"connect-src 'self' ws: wss: https:",
	"worker-src 'self' blob:",
	"manifest-src 'self'"
].join('; ');
const securityHeaders = {
	'Content-Security-Policy': contentSecurityPolicy
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
		headers: securityHeaders,
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
		headers: securityHeaders,
		proxy: {
			'/api': backendProxy,
			'/mcp': backendProxy,
			'/oauth': backendProxy,
			'/.well-known': backendProxy
		}
	}
});
