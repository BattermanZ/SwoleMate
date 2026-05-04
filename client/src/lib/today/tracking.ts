import type { UiExerciseSetting } from './types';

export const TRACKING_FIELDS_SETTING_KEY = '_tracking_fields';

export type TrackingFields = {
	reps: boolean;
	time: boolean;
	weight: boolean;
};

export const DEFAULT_TRACKING_FIELDS: TrackingFields = {
	reps: true,
	time: false,
	weight: true
};

export function encodeTrackingFields(fields: TrackingFields): string {
	return [fields.reps ? 'reps' : '', fields.time ? 'time' : '', fields.weight ? 'weight' : '']
		.filter(Boolean)
		.join(',');
}

export function decodeTrackingFields(value: string | undefined): TrackingFields {
	if (!value) return { ...DEFAULT_TRACKING_FIELDS };
	const parts = new Set(value.split(',').map((part) => part.trim()));
	const next = {
		reps: parts.has('reps'),
		time: parts.has('time'),
		weight: parts.has('weight')
	};
	return next.reps || next.time ? next : { ...DEFAULT_TRACKING_FIELDS };
}

export function trackingFieldsSetting(
	fields: TrackingFields
): Pick<UiExerciseSetting, 'key' | 'value'> {
	return {
		key: TRACKING_FIELDS_SETTING_KEY,
		value: encodeTrackingFields(fields)
	};
}

export function isTrackingFieldsSetting(setting: Pick<UiExerciseSetting, 'key'>): boolean {
	return setting.key === TRACKING_FIELDS_SETTING_KEY;
}
