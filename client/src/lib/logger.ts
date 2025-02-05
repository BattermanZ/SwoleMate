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
    private readonly API_BASE = config.apiUrl;
    private isBrowser: boolean;

    private constructor() {
        this.isBrowser = typeof window !== 'undefined';
        
        if (this.isBrowser) {
            // Create logs directory if it doesn't exist
            this.createLogsDirectory();
            
            // Process queued logs periodically
            setInterval(() => this.processLogQueue(), 1000);
            
            // Process logs before page unload
            window.addEventListener('beforeunload', () => {
                this.processLogQueue();
            });

            // Log application startup
            this.info('app', 'Frontend application started', {
                url: window.location.href,
                userAgent: navigator.userAgent
            });
        }
    }

    public static getInstance(): Logger {
        if (!Logger.instance) {
            Logger.instance = new Logger();
        }
        return Logger.instance;
    }

    private async createLogsDirectory() {
        if (!this.isBrowser) return;

        try {
            const response = await fetch(`${this.API_BASE}/api/logs/init`, {
                method: 'POST',
            });
            if (!response.ok) {
                console.error('Failed to create logs directory');
            } else {
                this.info('logger', 'Logs directory initialized');
            }
        } catch (error) {
            console.error('Error creating logs directory:', error);
        }
    }

    private createLogEntry(
        level: LogLevel,
        target: string,
        message: string,
        metadata?: Record<string, unknown>
    ): LogEntry {
        return {
            timestamp: new Date().toLocaleString('en-GB', {
                year: 'numeric',
                month: '2-digit',
                day: '2-digit',
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
                hour12: false
            }).replace(',', ' -'),
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

        this.isProcessing = true;
        const logs = [...this.logQueue];
        this.logQueue = [];

        try {
            const response = await fetch(`${this.API_BASE}/api/logs`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(logs.map(this.formatLogEntry)),
            });

            if (!response.ok) {
                // If failed, add logs back to queue
                this.logQueue.unshift(...logs);
                console.error('Failed to send logs to server');
            }
        } catch (error) {
            // If failed, add logs back to queue
            this.logQueue.unshift(...logs);
            console.error('Failed to write logs:', error);
        } finally {
            this.isProcessing = false;
        }
    }

    private log(level: LogLevel, target: string, message: string, metadata?: Record<string, unknown>) {
        // Skip logging during SSR
        if (!this.isBrowser) return;

        const logEntry = this.createLogEntry(level, target, message, metadata);
        
        // Add to queue for file logging
        this.logQueue.push(logEntry);
        
        // Also log to console in development
        if (import.meta.env.DEV) {
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