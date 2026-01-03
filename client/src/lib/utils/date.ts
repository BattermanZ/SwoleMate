const DAYS = [
	'Sunday',
	'Monday',
	'Tuesday',
	'Wednesday',
	'Thursday',
	'Friday',
	'Saturday'
] as const;
const MONTHS = [
	'January',
	'February',
	'March',
	'April',
	'May',
	'June',
	'July',
	'August',
	'September',
	'October',
	'November',
	'December'
] as const;

function getOrdinal(n: number): string {
	const suffixes = ['th', 'st', 'nd', 'rd'] as const;
	const v = n % 100;
	return `${n}${suffixes[(v - 20) % 10] || suffixes[v] || suffixes[0]}`;
}

export function formatTime(dateString: string): string {
	return new Date(dateString).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export function formatDateRelative(dateString: string): string {
	const date = new Date(dateString);
	const now = new Date();

	if (date.toDateString() === now.toDateString()) return 'Today';

	const yesterday = new Date(now);
	yesterday.setDate(yesterday.getDate() - 1);
	if (date.toDateString() === yesterday.toDateString()) return 'Yesterday';

	const lastWeek = new Date(now);
	lastWeek.setDate(lastWeek.getDate() - 7);
	if (date > lastWeek) return `Last ${DAYS[date.getDay()]}`;

	return `${DAYS[date.getDay()]}, ${getOrdinal(date.getDate())} of ${MONTHS[date.getMonth()]}`;
}

export function formatDateLongWithTime(date: Date): string {
	return `${DAYS[date.getDay()]}, ${getOrdinal(date.getDate())} of ${MONTHS[date.getMonth()]} at ${date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
}
