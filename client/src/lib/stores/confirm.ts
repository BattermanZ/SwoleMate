import { writable } from 'svelte/store';

export type ConfirmRequest = {
	title: string;
	message?: string;
	confirmLabel: string;
	cancelLabel: string;
	danger: boolean;
	input: { label?: string; placeholder?: string; value: string } | null;
	resolve: (value: boolean | string | null) => void;
};

/** The active dialog request, or null when nothing is open. Rendered by ConfirmHost. */
export const confirmRequest = writable<ConfirmRequest | null>(null);

type ConfirmOptions = {
	title: string;
	message?: string;
	confirmLabel?: string;
	cancelLabel?: string;
	danger?: boolean;
};

/** In-app replacement for window.confirm. Resolves true if confirmed, false otherwise. */
export function openConfirm(opts: ConfirmOptions): Promise<boolean> {
	return new Promise((resolve) => {
		confirmRequest.set({
			title: opts.title,
			message: opts.message,
			confirmLabel: opts.confirmLabel ?? 'Confirm',
			cancelLabel: opts.cancelLabel ?? 'Cancel',
			danger: opts.danger ?? false,
			input: null,
			resolve: (value) => resolve(value === true)
		});
	});
}

type PromptOptions = ConfirmOptions & {
	inputLabel?: string;
	placeholder?: string;
	defaultValue?: string;
};

/** In-app replacement for window.prompt. Resolves the entered string, or null if cancelled. */
export function openPrompt(opts: PromptOptions): Promise<string | null> {
	return new Promise((resolve) => {
		confirmRequest.set({
			title: opts.title,
			message: opts.message,
			confirmLabel: opts.confirmLabel ?? 'Save',
			cancelLabel: opts.cancelLabel ?? 'Cancel',
			danger: opts.danger ?? false,
			input: {
				label: opts.inputLabel,
				placeholder: opts.placeholder,
				value: opts.defaultValue ?? ''
			},
			resolve: (value) => resolve(typeof value === 'string' ? value : null)
		});
	});
}
