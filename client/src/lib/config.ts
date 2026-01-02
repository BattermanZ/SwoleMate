// Default values
const defaults = {
	devApiUrl: '',
	prodApiUrl: ''
};

// Environment variables from Vite (client.env)
export const config = {
	apiUrl:
		(import.meta.env.VITE_API_URL as string | undefined) ??
		(import.meta.env.DEV ? defaults.devApiUrl : defaults.prodApiUrl)
};

// Validate configuration
function validateConfig() {
	if (import.meta.env.DEV && import.meta.env.VITE_API_URL === undefined) {
		console.warn('API URL not configured. Using same-origin /api via Vite proxy.');
	}
}

// Run validation in development
if (import.meta.env.DEV) {
	validateConfig();
}
