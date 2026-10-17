// The door at sbvh.nl, from the browser's side (QNTX ADR-032). QNTX picks the
// door from the origin the page is on, so nothing here names a namespace:
// being served from sbvh.nl is what puts a registration in SBVH.
//
// The node sends a person back to the origin root with the ceremony ticket,
// and the page there collects it. What the page holds afterwards is a bearer;
// a cookie from api.q.sbvh.nl never reaches this origin.
//
// The same ceremony cleanamsterdam.nl and vakportaal run, said once more here
// because neither publishes it.

/** What laye, the wasm holding this browser's key, is asked for. */
export type Laye = {
	init(configJson: string): Promise<void>
	self_peer_id(): string
	did(): string
	sign(bytes: Uint8Array): Uint8Array
	accept_binding(bindingJson: string): void
	errors(): string
}

/** Who the node let in. `next` is "nothing" on the public rung: no passkey follows. */
export type Admission = {
	admitted_as: string
	next?: string
	level?: string
	user?: string
	name?: string
	picture?: string
	session?: string
}

/** A way in this door offers. Only a redirect provider is a button here. */
export type Provider = { id: string; label: string; kind: string }

export type Held = {
	getItem(key: string): string | null
	setItem(key: string, value: string): void
	removeItem(key: string): void
}

/** Everything the ceremony touches outside itself, so a test can hand in each. */
export type Deps = {
	node: string
	fetch: typeof globalThis.fetch
	laye: () => Promise<Laye>
	storage: Held
	url: () => string
	replaceUrl: (url: string) => void
	navigate: (url: string) => void
}

export const KEYS = {
	session: 'sbvh.session',
	name: 'sbvh.name',
	picture: 'sbvh.picture',
	email: 'sbvh.email',
	// What the person was doing when they left for the provider, so the page
	// they come back to opens where they were rather than at the top.
	naar: 'sbvh.naar',
} as const

/** A node answer that is not ok, in the node's own words. */
export async function refused(said: Response, what: string): Promise<never> {
	throw new Error(`${what} came back ${said.status}: ${(await said.text()).trim()}`)
}

/** Who this browser says it is, for drawing. Empty until a ceremony has said. */
export function whoIsHere(d: Pick<Deps, 'storage'>): { name: string; picture: string; email: string } {
	return {
		name: d.storage.getItem(KEYS.name) ?? '',
		picture: d.storage.getItem(KEYS.picture) ?? '',
		email: d.storage.getItem(KEYS.email) ?? '',
	}
}

/** Whether this browser came back from a ceremony, before anything is fetched. */
export function returning(d: Pick<Deps, 'url'>): boolean {
	const here = new URL(d.url())
	return here.searchParams.has('ceremony') || here.searchParams.has('home')
}

/** Called on arrival: collects a ceremony that just finished, or recognises a
 *  returning browser. Null is nobody, the ordinary state. */
export async function arrive(d: Deps): Promise<Admission | null> {
	const held = d.storage.getItem(KEYS.session)
	if (held) {
		const who = await d.fetch(`${d.node}/auth/status`, {
			headers: { Authorization: `Bearer ${held}` },
		})
		// A node that did not answer has not said anybody is nobody, so the
		// session stays and the refusal is shown rather than swallowed.
		if (!who.ok) await refused(who, 'GET /auth/status')
		const { identity, user, name, level, picture } = (await who.json()) as {
			identity?: string
			user?: string
			name?: string
			level?: string
			picture?: string
		}
		if (identity) {
			if (picture) d.storage.setItem(KEYS.picture, picture)
			if (name) d.storage.setItem(KEYS.name, name)
			return { admitted_as: identity, next: 'nothing', user, name, level }
		}
		// Answered, and the answer is nobody: the node forgot this session.
		forget(d)
	}

	const home = await backFromHome(d)
	if (home) return home

	const binding = await collected(d)
	if (!binding) return null

	const laye = await d.laye()
	// laye keeps what it holds in IndexedDB, so handing it over is what stops
	// the next visit costing another trip to the provider.
	laye.accept_binding(JSON.stringify(binding))

	const asked = await d.fetch(`${d.node}/auth/laye/challenge`, { credentials: 'include' })
	if (!asked.ok) await refused(asked, 'GET /auth/laye/challenge')
	const { challenge } = (await asked.json()) as { challenge: string }

	const signature = laye.sign(new TextEncoder().encode(challenge))
	if (signature.length === 0) throw new Error(`laye signed nothing: ${laye.errors()}`)

	const verified = await d.fetch(`${d.node}/auth/laye/verify`, {
		method: 'POST',
		credentials: 'include',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({
			did: laye.did(),
			challenge,
			signature: base64url(signature),
			bindings: [binding],
		}),
	})
	if (!verified.ok) await refused(verified, 'POST /auth/laye/verify')

	const admission = (await verified.json()) as Admission
	// A root identity stands on a passkey, and the passkey lives at the node's
	// own domain. The node does it there and sends the browser back with a
	// ticket. A top-level navigation sends no Origin and a browser may strip
	// the Referer, so the door is named in the address.
	if (admission.next && admission.next !== 'nothing') {
		d.navigate(`${d.node}/auth/door/home?door=${encodeURIComponent(new URL(d.url()).origin)}`)
		return null
	}
	return keep(d, admission)
}

/** Holds what an admission handed over, and answers it. */
function keep(d: Deps, admission: Admission): Admission {
	// No session is nothing to come back with, and a face shown until the
	// page reloads is the failure that looks like success.
	if (!admission.session) {
		throw new Error(`the node admitted ${admission.admitted_as} and handed back no session`)
	}
	d.storage.setItem(KEYS.session, admission.session)
	if (admission.name) d.storage.setItem(KEYS.name, admission.name)
	if (admission.picture) d.storage.setItem(KEYS.picture, admission.picture)
	return admission
}

/** Back from the passkey at the node. Collected once, and the ticket is spent. */
async function backFromHome(d: Deps): Promise<Admission | null> {
	const here = new URL(d.url())
	const ticket = here.searchParams.get('home')
	if (!ticket) return null

	here.searchParams.delete('home')
	d.replaceUrl(here.toString())

	const answer = await d.fetch(`${d.node}/auth/door/home/result?home=${encodeURIComponent(ticket)}`)
	if (!answer.ok) await refused(answer, 'GET /auth/door/home/result')
	return keep(d, (await answer.json()) as Admission)
}

/** What the node signed for the ceremony this browser started. The ticket
 *  rides the URL: no cookie the node sets comes back on a fetch from here. */
async function collected(d: Deps): Promise<unknown | null> {
	const here = new URL(d.url())
	const ticket = here.searchParams.get('ceremony')
	if (!ticket) return null

	// Off the address bar on read. It is spent, and it has no business in history.
	here.searchParams.delete('ceremony')
	d.replaceUrl(here.toString())

	const answer = await d.fetch(`${d.node}/auth/binding/result?ceremony=${encodeURIComponent(ticket)}`)
	if (!answer.ok) await refused(answer, 'GET /auth/binding/result')

	// The name and the picture come beside the binding, not inside it.
	const { binding, name, picture } = (await answer.json()) as {
		binding: { claim?: { handle?: string | null } } | null
		name?: string
		picture?: string
	}
	if (!binding) throw new Error('GET /auth/binding/result answered no binding')
	if (name) d.storage.setItem(KEYS.name, name)
	if (picture) d.storage.setItem(KEYS.picture, picture)
	const handle = binding.claim?.handle
	if (handle) d.storage.setItem(KEYS.email, handle)
	return binding
}

/** The ways in this door offers, as the node says them. */
export async function providers(d: Deps): Promise<Provider[]> {
	const answer = await d.fetch(`${d.node}/auth/binding/providers`, { credentials: 'include' })
	if (!answer.ok) await refused(answer, 'GET /auth/binding/providers')
	const { providers } = (await answer.json()) as { providers?: Provider[] }
	return (providers ?? []).filter((p) => p.kind === 'redirect')
}

/** Go and link an account. A navigation, not a fetch: the ceremony cookie is
 *  set by the node the browser is going to, so it is first-party there. */
export async function signInWith(d: Deps, provider: string, naar: string): Promise<void> {
	const laye = await d.laye()
	d.storage.setItem(KEYS.naar, naar)
	d.navigate(
		`${d.node}/auth/binding/go` +
			`?provider=${encodeURIComponent(provider)}` +
			`&peer_pubkey_hex=${encodeURIComponent(laye.self_peer_id())}`,
	)
}

/** Where the person was before the provider, once: read and forgotten. */
export function cameBackFor(d: Pick<Deps, 'storage'>): string {
	const naar = d.storage.getItem(KEYS.naar) ?? ''
	d.storage.removeItem(KEYS.naar)
	return naar
}

/** Ends the session at the node, then here. */
export async function signOut(d: Deps): Promise<void> {
	const token = d.storage.getItem(KEYS.session)
	if (token) {
		await d
			.fetch(`${d.node}/auth/logout`, { method: 'POST', headers: { Authorization: `Bearer ${token}` } })
			.catch(() => {
				// The node not answering does not keep somebody signed in here.
			})
	}
	forget(d)
}

function forget(d: Pick<Deps, 'storage'>): void {
	for (const key of Object.values(KEYS)) d.storage.removeItem(key)
}

/** A call to the node as the signed-in person. */
export async function asked(d: Deps, path: string, init: RequestInit = {}): Promise<Response> {
	const token = d.storage.getItem(KEYS.session)
	if (!token) throw new Error('not signed in')
	const headers = new Headers(init.headers)
	headers.set('Authorization', `Bearer ${token}`)
	if (init.body) headers.set('Content-Type', 'application/json')
	return d.fetch(`${d.node}${path}`, { ...init, headers })
}

function base64url(bytes: Uint8Array): string {
	let binary = ''
	for (const b of bytes) binary += String.fromCharCode(b)
	return btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '')
}

/** Load and start laye, once. Served out of /laye/, so this is a URL at
 *  runtime and not a module any build step resolves. Nearly two megabytes, so
 *  nothing loads it until the sign-in panel opens or a ceremony needs it. */
export function browserLaye(node: string): () => Promise<Laye> {
	let starting: Promise<Laye> | null = null
	return () => {
		if (starting) return starting
		starting = (async () => {
			// Built at runtime, so the bundler leaves it alone rather than
			// trying to resolve a file that only exists once deployed.
			const from = new URL('/laye/laye_p2p.js', window.location.origin).href
			const mod = (await import(from)) as Laye & { default: (wasm: string) => Promise<unknown> }
			await mod.default('/laye/laye_p2p_bg.wasm')
			// Whose signature on a binding counts, from the node's own list;
			// without it laye believes any peer that signs a claim about anybody.
			const answer = await fetch(`${node}/auth/status`)
			const { binding_signers } = answer.ok
				? ((await answer.json()) as { binding_signers?: string[] })
				: { binding_signers: [] }
			await mod.init(
				JSON.stringify({
					bootstrap_addrs: [],
					identify_protocol: '/qntx/1.0.0',
					binding_signers: binding_signers ?? [],
				}),
			)
			return mod
		})()
		// A load that failed is not held: the next tap tries again.
		starting.catch(() => {
			starting = null
		})
		return starting
	}
}

/** localStorage, or a browser that refuses it, which costs this visit and nothing else. */
export function browserStorage(): Held {
	const none: Held = { getItem: () => null, setItem: () => {}, removeItem: () => {} }
	try {
		return window.localStorage ?? none
	} catch {
		return none
	}
}
