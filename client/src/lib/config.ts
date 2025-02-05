// Default values for development
const defaults = {
    apiUrl: 'http://localhost:2469'
};

// Environment variables from Vite (client.env)
export const config = {
    apiUrl: import.meta.env.VITE_API_URL || defaults.apiUrl
};

// Validate configuration
function validateConfig() {
    if (!config.apiUrl) {
        console.warn('API URL not configured. Using default:', defaults.apiUrl);
    }
}

// Run validation in development
if (import.meta.env.DEV) {
    validateConfig();
} 