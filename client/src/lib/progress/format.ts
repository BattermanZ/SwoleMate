export function formatDuration(seconds: number): string {
	const safeSeconds = Math.max(0, Math.round(seconds));
	const hours = Math.floor(safeSeconds / 3600);
	const minutes = Math.floor((safeSeconds % 3600) / 60);
	const remainingSeconds = safeSeconds % 60;

	if (hours > 0) return `${hours}h ${String(minutes).padStart(2, '0')}m`;
	if (minutes > 0) return `${minutes}m ${String(remainingSeconds).padStart(2, '0')}s`;
	return `${remainingSeconds}s`;
}

export function formatSignedNumber(value: number, suffix = ''): string {
	const rounded = Math.round(value);
	if (rounded === 0) return `0${suffix}`;
	return `${rounded > 0 ? '+' : ''}${rounded}${suffix}`;
}

export function formatSignedDuration(seconds: number): string {
	const rounded = Math.round(seconds);
	if (rounded === 0) return '0s';
	return `${rounded > 0 ? '+' : '-'}${formatDuration(Math.abs(rounded))}`;
}
