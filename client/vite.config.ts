import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		host: true,
		port: 2470,
		proxy: {
			'/api': {
				target: process.env.VITE_API_PROXY_TARGET ?? 'http://127.0.0.1:2469',
				changeOrigin: true
			}
		}
	},
	preview: {
		host: true,
		port: 2470,
		proxy: {
			'/api': {
				target: process.env.VITE_API_PROXY_TARGET ?? 'http://127.0.0.1:2469',
				changeOrigin: true
			}
		}
	}
});
