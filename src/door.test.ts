import { describe, expect, test } from 'bun:test'
import { arrive, cameBackFor, KEYS, providers, returning, signInWith, signOut, type Deps, type Laye } from './door'

const NODE = 'https://api.q.sbvh.nl'

type Answer = { status?: number; body: unknown }

/** A node that answers each path it is given, and remembers what was asked. */
function door(answers: Record<string, Answer>, url = 'https://sbvh.nl/') {
	const held = new Map<string, string>()
	const asked: { path: string; init?: RequestInit }[] = []
	const went: string[] = []
	let here = url
	const laye: Laye = {
		init: async () => {},
		self_peer_id: () => 'peer-1',
		did: () => 'did:key:z6Mk',
		sign: () => new Uint8Array([1, 2, 3]),
		accept_binding: () => {},
		errors: () => '',
	}
	const d: Deps = {
		node: NODE,
		fetch: (async (input: string, init?: RequestInit) => {
			const path = String(input).slice(NODE.length)
			asked.push({ path, init })
			const a = answers[path.split('?')[0]]
			if (!a) return new Response(`no answer for ${path}`, { status: 599 })
			return new Response(typeof a.body === 'string' ? a.body : JSON.stringify(a.body), { status: a.status ?? 200 })
		}) as typeof fetch,
		laye: async () => laye,
		storage: {
			getItem: (k) => held.get(k) ?? null,
			setItem: (k, v) => void held.set(k, v),
			removeItem: (k) => void held.delete(k),
		},
		url: () => here,
		replaceUrl: (u) => void (here = u),
		navigate: (u) => void went.push(u),
	}
	return { d, held, asked, went, here: () => here }
}

describe('arriving', () => {
	test('nobody holding nothing is nobody, and nothing is asked', async () => {
		const t = door({})
		expect(await arrive(t.d)).toBeNull()
		expect(t.asked).toHaveLength(0)
		expect(returning(t.d)).toBe(false)
	})

	test('a finished ceremony is collected, signed for, and held as a session', async () => {
		const t = door(
			{
				'/auth/binding/result': {
					body: { binding: { claim: { handle: 'ada@example.com' } }, name: 'Ada', picture: 'https://p/a.png' },
				},
				'/auth/laye/challenge': { body: { challenge: 'c-1' } },
				'/auth/laye/verify': { body: { admitted_as: 'google:1', next: 'nothing', session: 's-1', user: 'U-1' } },
			},
			'https://sbvh.nl/?ceremony=T-1',
		)
		expect(returning(t.d)).toBe(true)
		const a = await arrive(t.d)
		expect(a?.admitted_as).toBe('google:1')
		expect(t.held.get(KEYS.session)).toBe('s-1')
		expect(t.held.get(KEYS.email)).toBe('ada@example.com')
		expect(t.held.get(KEYS.name)).toBe('Ada')
		// The spent ticket is off the address bar.
		expect(t.here()).toBe('https://sbvh.nl/')
		const verify = t.asked.find((x) => x.path === '/auth/laye/verify')!
		expect(JSON.parse(String(verify.init?.body))).toMatchObject({ did: 'did:key:z6Mk', challenge: 'c-1', signature: 'AQID' })
	})

	test('an admission with no session is said, not shown as somebody', async () => {
		const t = door(
			{
				'/auth/binding/result': { body: { binding: {} } },
				'/auth/laye/challenge': { body: { challenge: 'c-1' } },
				'/auth/laye/verify': { body: { admitted_as: 'google:1', next: 'nothing' } },
			},
			'https://sbvh.nl/?ceremony=T-1',
		)
		await expect(arrive(t.d)).rejects.toThrow('handed back no session')
	})

	test('a refusal is said in the node\'s own words', async () => {
		const t = door({ '/auth/binding/result': { status: 401, body: 'refused' } }, 'https://sbvh.nl/?ceremony=T-1')
		await expect(arrive(t.d)).rejects.toThrow('GET /auth/binding/result came back 401: refused')
	})

	test('a root identity goes to the node for its passkey, naming the door', async () => {
		const t = door(
			{
				'/auth/binding/result': { body: { binding: {} } },
				'/auth/laye/challenge': { body: { challenge: 'c-1' } },
				'/auth/laye/verify': { body: { admitted_as: 'github:1739143', next: 'assert' } },
			},
			'https://sbvh.nl/?ceremony=T-1',
		)
		expect(await arrive(t.d)).toBeNull()
		expect(t.went).toEqual([`${NODE}/auth/door/home?door=${encodeURIComponent('https://sbvh.nl')}`])
	})

	test('back from the passkey, the ticket is redeemed for the session', async () => {
		const t = door(
			{ '/auth/door/home/result': { body: { admitted_as: 'github:1739143', session: 's-root', level: 'ROOT' } } },
			'https://sbvh.nl/?home=H-1',
		)
		expect((await arrive(t.d))?.level).toBe('ROOT')
		expect(t.held.get(KEYS.session)).toBe('s-root')
		expect(t.here()).toBe('https://sbvh.nl/')
	})

	test('a held session the node still knows is somebody', async () => {
		const t = door({ '/auth/status': { body: { identity: 'google:1', user: 'U-1', name: 'Ada' } } })
		t.held.set(KEYS.session, 's-1')
		expect((await arrive(t.d))?.user).toBe('U-1')
		expect(new Headers(t.asked[0].init?.headers).get('Authorization')).toBe('Bearer s-1')
	})

	test('a held session the node forgot is dropped', async () => {
		const t = door({ '/auth/status': { body: {} } })
		t.held.set(KEYS.session, 's-1')
		t.held.set(KEYS.name, 'Ada')
		expect(await arrive(t.d)).toBeNull()
		expect(t.held.size).toBe(0)
	})

	test('a node that did not answer keeps the session and says why', async () => {
		const t = door({ '/auth/status': { status: 502, body: 'bad gateway' } })
		t.held.set(KEYS.session, 's-1')
		await expect(arrive(t.d)).rejects.toThrow('502')
		expect(t.held.get(KEYS.session)).toBe('s-1')
	})
})

describe('leaving and coming back', () => {
	test('signing in goes to the node with this browser\'s key, and remembers why', async () => {
		const t = door({})
		await signInWith(t.d, 'google', 'book')
		expect(t.went).toEqual([`${NODE}/auth/binding/go?provider=google&peer_pubkey_hex=peer-1`])
		expect(cameBackFor(t.d)).toBe('book')
		expect(cameBackFor(t.d)).toBe('')
	})

	test('only a provider that redirects is offered as a button', async () => {
		const t = door({
			'/auth/binding/providers': {
				body: { providers: [{ id: 'google', label: 'Google', kind: 'redirect' }, { id: 'atproto', label: 'Bluesky', kind: 'credential' }] },
			},
		})
		expect((await providers(t.d)).map((p) => p.id)).toEqual(['google'])
	})

	test('signing out ends the session at the node and forgets it here', async () => {
		const t = door({ '/auth/logout': { body: {} } })
		t.held.set(KEYS.session, 's-1')
		t.held.set(KEYS.picture, 'p')
		await signOut(t.d)
		expect(t.asked[0].path).toBe('/auth/logout')
		expect(t.held.size).toBe(0)
	})
})
