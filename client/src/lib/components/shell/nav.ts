/** Shared active-route logic for BottomNav (mobile) and SideNav (desktop). */
export function isActive(href: string, current?: string): boolean {
	if (!current) return false;
	if (href === '/') return current === '/';
	return current === href || current.startsWith(`${href}/`);
}
