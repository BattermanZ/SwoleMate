import { config } from './config';

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
	timestamp: string;
	level: LogLevel;
	target: string;
	message: string;
	metadata?: Record<string, unknown>;
}

class Logger {
	private static instance: Logger;
	private logQueue: LogEntry[] = [];
	private isProcessing: boolean = false;
	private remoteEnabled: boolean = false;
	private readonly API_BASE = config.apiUrl;
	private isBrowser: boolean;
	private readonly MAX_QUEUE = 500;

	private constructor() {
		this.isBrowser = typeof window !== 'undefined';

		if (this.isBrowser) {
			// Process queued logs periodically
			setInterval(() => this.processLogQueue(), 1000);

			// Process logs before page unload
			window.addEventListener('beforeunload', () => {
				this.processLogQueue();
			});

			// Best-effort flush when the page is being hidden (more reliable than beforeunload on mobile).
			window.addEventListener('pagehide', () => {
				void this.processLogQueue();
			});

			document.addEventListener('visibilitychange', () => {
				if (document.visibilityState === 'hidden') {
					void this.processLogQueue();
				}
			});

			window.addEventListener('online', () => {
				void this.processLogQueue();
			});

			// Log application startup. Record only the pathname (no query/hash) and
			// omit the user agent, so this line can never carry a sensitive URL
			// parameter or fingerprintable UA off-device if the remote-send gate is
			// ever loosened (F-LOW-5).
			this.info('app', 'Frontend application started', {
				path: window.location.pathname
			});
		}
	}

	public static getInstance(): Logger {
		if (!Logger.instance) {
			Logger.instance = new Logger();
		}
		return Logger.instance;
	}

	public setRemoteEnabled(enabled: boolean) {
		if (!this.isBrowser) return;
		this.remoteEnabled = enabled;
		if (enabled) void this.processLogQueue();
	}

	private createLogEntry(
		level: LogLevel,
		target: string,
		message: string,
		metadata?: Record<string, unknown>
	): LogEntry {
		return {
			timestamp: new Date().toISOString(),
			level,
			target,
			message,
			metadata
		};
	}

	private formatLogEntry(entry: LogEntry): string {
		let log = `[${entry.timestamp}] ${entry.level.toUpperCase()} - ${entry.target} - ${entry.message}`;
		if (entry.metadata && Object.keys(entry.metadata).length > 0) {
			log += ` | ${JSON.stringify(entry.metadata)}`;
		}
		return log;
	}

	private async processLogQueue() {
		if (!this.isBrowser || this.isProcessing || this.logQueue.length === 0) return;
		if (!this.remoteEnabled) return;

		this.isProcessing = true;
		const logs = [...this.logQueue];
		this.logQueue = [];

		try {
			const response = await fetch(`${this.API_BASE}/api/logs`, {
				method: 'POST',
				credentials: 'include',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(logs)
			});

			if (!response.ok) {
				if (response.status === 401 || response.status === 403) {
					// Auth lapsed (e.g. mid token-refresh). Preserve this batch by
					// putting it back on the queue and pause remote sending until it's
					// explicitly re-enabled at the next auth bootstrap, instead of
					// silently dropping the diagnostics (F-LOW-3).
					this.logQueue.unshift(...logs);
					this.remoteEnabled = false;
					return;
				}
				// If failed, add logs back to queue
				this.logQueue.unshift(...logs);
			}
		} catch {
			// If failed, add logs back to queue
			this.logQueue.unshift(...logs);
		} finally {
			this.isProcessing = false;
		}
	}

	private shouldSendRemotely(level: LogLevel): boolean {
		// Keep remote logs minimal to avoid noise/PII; send warnings+errors only.
		return level === 'warn' || level === 'error';
	}

	private log(
		level: LogLevel,
		target: string,
		message: string,
		metadata?: Record<string, unknown>
	) {
		// Skip logging during SSR
		if (!this.isBrowser) return;

		const logEntry = this.createLogEntry(level, target, message, metadata);

		// Queue for server-side ingestion (stdout) when enabled.
		if (this.remoteEnabled && this.shouldSendRemotely(level)) {
			this.logQueue.push(logEntry);
			if (this.logQueue.length > this.MAX_QUEUE) {
				this.logQueue.splice(0, this.logQueue.length - this.MAX_QUEUE);
			}
		}

		// Console logging:
		// - dev: all levels
		// - prod: warn+error only
		if (import.meta.env.DEV || level === 'warn' || level === 'error') {
			const consoleMethod = level === 'debug' ? 'log' : level;
			console[consoleMethod](this.formatLogEntry(logEntry));
		}
	}

	public debug(target: string, message: string, metadata?: Record<string, unknown>) {
		this.log('debug', target, message, metadata);
	}

	public info(target: string, message: string, metadata?: Record<string, unknown>) {
		this.log('info', target, message, metadata);
	}

	public warn(target: string, message: string, metadata?: Record<string, unknown>) {
		this.log('warn', target, message, metadata);
	}

	public error(target: string, message: string, metadata?: Record<string, unknown>) {
		this.log('error', target, message, metadata);
	}
}

export const logger = Logger.getInstance();
