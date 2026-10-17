import { describe, expect, test } from 'bun:test'
import { byDay, clock, ics, when } from './calls'

// Monday 19 October 2026, 10:00 and 10:30 Amsterdam; Tuesday 10:00.
const MON_10 = 1_792_396_800
const MON_1030 = MON_10 + 1_800
const TUE_10 = MON_10 + 86_400

describe('showing slots', () => {
	test('slots are grouped by the day they fall on, in the zone asked', () => {
		const days = byDay([MON_10, MON_1030, TUE_10], 'Europe/Amsterdam')
		expect(days.map((d) => d.label)).toEqual(['Mon 19 Oct', 'Tue 20 Oct'])
		expect(days[0].starts).toEqual([MON_10, MON_1030])
	})

	test('a viewer far west sees the same slots on their own clock', () => {
		expect(clock(MON_10, 'America/New_York')).toBe('04:00')
		expect(when(MON_10, 30, 'Europe/Amsterdam')).toBe('Monday 19 October, 10:00–10:30')
	})
})

describe('the calendar file', () => {
	test('names the call in UTC, with what was written, escaped', () => {
		const file = ics({ meet_id: 'M-1', start: MON_10, minutes: 30, about: 'DORA; agents, and more' }, 'Video call')
		expect(file).toContain('DTSTART:20261019T080000Z')
		expect(file).toContain('DTEND:20261019T083000Z')
		expect(file).toContain('UID:M-1@sbvh.nl')
		expect(file).toContain('DESCRIPTION:DORA\\; agents\\, and more')
		expect(file).toContain('LOCATION:Video call')
		expect(file.split('\r\n')[0]).toBe('BEGIN:VCALENDAR')
	})
})
