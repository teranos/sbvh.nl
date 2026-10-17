// Booking a call, against sbvhAPI on the node. Every answer that is not ok is
// thrown in the node's own words, so the panel shows what the node said.

import { asked, type Deps } from './door'

const API = '/api/sbvhAPI'

export type Call = { meet_id: string; start: number; minutes: number; about: string }

export type Slots = { zone: string; minutes: number; place: string; slots: number[] }

/** A mail's outcome: sent, or the reason it did not leave. */
export type Told = true | { error: string }

export type Booked = { call: Call; told: { you: Told; owner: Told } }

async function answered<T>(said: Response, what: string): Promise<T> {
	const text = await said.text()
	let body: unknown = null
	try {
		body = JSON.parse(text)
	} catch {
		// Not JSON is said as it came, below.
	}
	if (!said.ok) {
		const error = (body as { error?: string } | null)?.error
		throw new Error(error ? error : `${what} came back ${said.status}: ${text.trim()}`)
	}
	if (body === null) throw new Error(`${what} answered something that is not JSON: ${text.slice(0, 200)}`)
	return body as T
}

export async function slots(d: Deps): Promise<Slots> {
	return answered(await asked(d, `${API}/slots`), 'GET /slots')
}

export async function mine(d: Deps): Promise<Call[]> {
	const { calls } = await answered<{ calls: Call[] }>(await asked(d, `${API}/mine`), 'GET /mine')
	return calls
}

export async function book(
	d: Deps,
	ask: { start: number; about: string; name: string; email: string; replaces?: string },
): Promise<Booked> {
	return answered(await asked(d, `${API}/book`, { method: 'POST', body: JSON.stringify(ask) }), 'POST /book')
}

export async function cancel(d: Deps, meet_id: string): Promise<void> {
	await answered(await asked(d, `${API}/cancel`, { method: 'POST', body: JSON.stringify({ meet_id }) }), 'POST /cancel')
}

/** The slots as days to pick from, each day's times in order, in the zone the
 *  viewer's clock is in. */
export function byDay(starts: number[], zone?: string): { day: string; label: string; starts: number[] }[] {
	const key = new Intl.DateTimeFormat('en-CA', { timeZone: zone, year: 'numeric', month: '2-digit', day: '2-digit' })
	const label = new Intl.DateTimeFormat('en-GB', { timeZone: zone, weekday: 'short', day: 'numeric', month: 'short' })
	const days = new Map<string, { day: string; label: string; starts: number[] }>()
	for (const start of starts) {
		const at = new Date(start * 1000)
		const day = key.format(at)
		if (!days.has(day)) days.set(day, { day, label: label.format(at), starts: [] })
		days.get(day)!.starts.push(start)
	}
	return [...days.values()]
}

export function clock(start: number, zone?: string): string {
	return new Intl.DateTimeFormat('en-GB', { timeZone: zone, hour: '2-digit', minute: '2-digit' }).format(new Date(start * 1000))
}

export function when(start: number, minutes: number, zone?: string): string {
	const day = new Intl.DateTimeFormat('en-GB', { timeZone: zone, weekday: 'long', day: 'numeric', month: 'long' }).format(
		new Date(start * 1000),
	)
	return `${day}, ${clock(start, zone)}–${clock(start + minutes * 60, zone)}`
}

/** A calendar file for one call, so it lands in whatever calendar the person keeps. */
export function ics(call: Call, place: string): string {
	const stamp = (s: number) => new Date(s * 1000).toISOString().replace(/[-:]/g, '').replace(/\.\d{3}/, '')
	const escaped = (t: string) => t.replace(/\\/g, '\\\\').replace(/\n/g, '\\n').replace(/([,;])/g, '\\$1')
	return [
		'BEGIN:VCALENDAR',
		'VERSION:2.0',
		'PRODID:-//SBVH//sbvh.nl//EN',
		'BEGIN:VEVENT',
		`UID:${call.meet_id}@sbvh.nl`,
		`DTSTAMP:${stamp(Math.floor(Date.now() / 1000))}`,
		`DTSTART:${stamp(call.start)}`,
		`DTEND:${stamp(call.start + call.minutes * 60)}`,
		'SUMMARY:Call with Sebastiaan van Houten',
		...(place ? [`LOCATION:${escaped(place)}`] : []),
		...(call.about ? [`DESCRIPTION:${escaped(call.about)}`] : []),
		'END:VEVENT',
		'END:VCALENDAR',
		'',
	].join('\r\n')
}
