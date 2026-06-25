// Semver version injected at build time from package.json (see vite.config.ts).
// The `typeof` guard keeps it safe in any context where the define is absent.
export const APP_VERSION: string =
	typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.0.0';
