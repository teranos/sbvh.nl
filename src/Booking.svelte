<script lang="ts">
	import { onMount, tick } from 'svelte'
	import {
		arrive, cameBackFor, providers as offered, returning, signInWith, signOut, whoIsHere, KEYS,
		type Admission, type Deps, type Provider,
	} from './door'
	import { book, byDay, cancel, clock, ics, mine, slots as openSlots, when, type Call, type Slots, type Told } from './calls'
	import { ABOUT_MOST } from './limits'

	let { d, open = $bindable(false) }: { d: Deps; open?: boolean } = $props()

	const stand = (event: string, params: Record<string, unknown> = {}) =>
		(window as unknown as { stand?: (e: string, p: Record<string, unknown>) => void }).stand?.(event, params)

	const here = Intl.DateTimeFormat().resolvedOptions().timeZone
	const AMSTERDAM = 'Europe/Amsterdam'

	// Who is here. `arriving` is the moment between a page load holding a
	// session or a ticket and the node saying who that is.
	let phase = $state<'arriving' | 'nobody' | 'known'>('nobody')
	let admission = $state<Admission | null>(null)
	// Read once mounted: nothing draws a face before the node has said who.
	let who = $state({ name: '', picture: '', email: '' })

	// Nobody yet: the ways in, and the one being taken.
	let ways = $state<Provider[] | null>(null)
	let leaving = $state('')

	// Somebody: what they have and what they could have.
	let calls = $state<Call[] | null>(null)
	let offer = $state<Slots | null>(null)
	let day = $state('')
	let chosen = $state<number | null>(null)
	let about = $state('')
	let moving = $state<Call | null>(null)
	let busy = $state('')
	let confirmCancel = $state(false)
	let told = $state<{ you: Told; owner: Told } | null>(null)

	let error = $state('')
	let panel: HTMLDivElement | undefined = $state()

	const days = $derived(offer ? byDay(offer.slots, here) : [])
	const times = $derived(days.find((x) => x.day === day)?.starts ?? [])
	const call = $derived(calls?.[0] ?? null)
	const picking = $derived(phase === 'known' && calls !== null && (call === null || moving !== null))

	function said(e: unknown): string {
		// A fetch that never got an answer says only "Failed to fetch", which
		// tells nobody which side is down.
		if (e instanceof TypeError) return `Could not reach ${new URL(d.node).host}. Try again in a moment.`
		return e instanceof Error ? e.message : String(e)
	}

	onMount(() => {
		who = whoIsHere(d)
		const back = returning(d)
		if (!back && !d.storage.getItem(KEYS.session)) return
		phase = 'arriving'
		if (back) open = cameBackFor(d) === 'book' || open
		arrive(d)
			.then((a) => {
				who = whoIsHere(d)
				admission = a
				phase = a ? 'known' : 'nobody'
				if (a && back) stand('signed_in', { level: a.level ?? '' })
			})
			.catch((e) => {
				who = whoIsHere(d)
				phase = d.storage.getItem(KEYS.session) ? 'known' : 'nobody'
				error = said(e)
				if (back) open = true
			})
	})

	// What the panel needs, fetched when it opens onto the phase that needs it.
	$effect(() => {
		if (!open) return
		if (phase === 'nobody' && ways === null) {
			offered(d)
				.then((p) => (ways = p))
				.catch((e) => {
					ways = []
					error = said(e)
				})
			// The key's wasm is nearly two megabytes. Fetched while the person
			// reads, so the tap that needs it does not wait for it.
			d.laye().catch(() => {})
		}
		if (phase === 'known' && calls === null && !busy) load()
	})

	$effect(() => {
		if (!open) return
		const keydown = (e: KeyboardEvent) => {
			if (e.key === 'Escape') close()
		}
		window.addEventListener('keydown', keydown)
		// The dialog itself takes focus, so a stray Enter signs nobody out.
		tick().then(() => panel?.focus())
		return () => window.removeEventListener('keydown', keydown)
	})

	async function load() {
		busy = 'loading'
		error = ''
		try {
			const [c, s] = await Promise.all([mine(d), openSlots(d)])
			calls = c
			offer = s
			day = byDay(s.slots, here)[0]?.day ?? ''
		} catch (e) {
			error = said(e)
		} finally {
			busy = ''
		}
	}

	function close() {
		open = false
		confirmCancel = false
	}

	async function signIn(p: Provider) {
		leaving = p.id
		error = ''
		stand('signin', { provider: p.id })
		try {
			await signInWith(d, p.id, 'book')
		} catch (e) {
			leaving = ''
			error = said(e)
		}
	}

	async function out() {
		stand('signout')
		await signOut(d)
		phase = 'nobody'
		admission = null
		calls = null
		offer = null
		moving = null
		told = null
		who = whoIsHere(d)
	}

	async function confirm() {
		if (chosen === null) return
		busy = 'booking'
		error = ''
		try {
			const booked = await book(d, {
				start: chosen,
				about,
				name: who.name,
				email: who.email,
				replaces: moving?.meet_id,
			})
			stand(moving ? 'moved' : 'booked')
			calls = [booked.call]
			told = booked.told
			moving = null
			chosen = null
			about = ''
		} catch (e) {
			error = said(e)
			// The slot may have gone while they looked; what is open now is
			// the honest answer to that.
			openSlots(d).then((s) => (offer = s)).catch(() => {})
		} finally {
			busy = ''
		}
	}

	async function drop() {
		if (!call) return
		busy = 'cancelling'
		error = ''
		try {
			await cancel(d, call.meet_id)
			stand('cancelled')
			calls = []
			told = null
			confirmCancel = false
			offer = await openSlots(d)
			day = byDay(offer.slots, here)[0]?.day ?? ''
		} catch (e) {
			error = said(e)
		} finally {
			busy = ''
		}
	}

	function move() {
		if (!call) return
		moving = call
		chosen = null
		about = call.about
		told = null
	}

	function calendarFile() {
		if (!call) return
		const blob = new Blob([ics(call, offer?.place ?? '')], { type: 'text/calendar' })
		const a = document.createElement('a')
		a.href = URL.createObjectURL(blob)
		a.download = 'call-with-sebastiaan.ics'
		a.click()
		setTimeout(() => URL.revokeObjectURL(a.href), 1000)
		stand('calendar')
	}

	function initial(name: string): string {
		return (name.trim()[0] ?? '·').toUpperCase()
	}

	function toldYou(t: Told | undefined): string {
		if (t === undefined) return ''
		if (t === true) return who.email ? `A confirmation is on its way to ${who.email}.` : 'A confirmation is on its way to you.'
		return `Booked, but the confirmation mail did not leave: ${t.error}`
	}
</script>

{#if phase === 'known' && !open}
	<button class="me" onclick={() => (open = true)} aria-label="Your account and your call" title={who.name || 'Your account'}>
		{#if who.picture}<img src={who.picture} alt="" referrerpolicy="no-referrer" />{:else}<span>{initial(who.name)}</span>{/if}
	</button>
{/if}

{#if open}
	<div class="backdrop" onclick={close} role="presentation"></div>
	<div class="panel" role="dialog" aria-modal="true" aria-labelledby="booking-title" tabindex="-1" bind:this={panel}>
		<button class="x" onclick={close} aria-label="Close">×</button>

		{#if phase === 'arriving'}
			<h2 id="booking-title">Signing you in</h2>
			<p class="quiet working">One moment.</p>
		{:else if phase === 'nobody'}
			<h2 id="booking-title">Book a call</h2>
			<p>Sign in first, so I know who I'm meeting. One tap, no password.</p>
			<div class="ways">
				{#if ways === null}
					<div class="way placeholder"></div>
					<div class="way placeholder"></div>
				{:else}
					{#each ways as p (p.id)}
						<button class="way" onclick={() => signIn(p)} disabled={leaving !== ''}>
							{leaving === p.id ? `Opening ${p.label}…` : `Continue with ${p.label}`}
						</button>
					{/each}
				{/if}
			</div>
			<p class="quiet small">
				Your account lives on my own QNTX node, in the SBVH namespace, and holds what your provider shares: your name and email address.
				<a href="/privacy.html#sbvh-nl">Privacy</a>
			</p>
		{:else}
			<div class="who">
				{#if who.picture}<img src={who.picture} alt="" referrerpolicy="no-referrer" />{:else}<span class="face">{initial(who.name)}</span>{/if}
				<span class="name">{who.name || who.email || 'Signed in'}</span>
				<button class="link" onclick={out}>Sign out</button>
			</div>

			{#if calls === null}
				<h2 id="booking-title">Book a call</h2>
				<p class="quiet working">{busy === 'loading' ? 'Finding open times.' : ''}</p>
			{:else if picking}
				<h2 id="booking-title">{moving ? 'Move your call' : 'Pick a time'}</h2>
				{#if offer}
					<p class="quiet small">
						{offer.minutes} minutes{offer.place ? `, ${offer.place.charAt(0).toLowerCase()}${offer.place.slice(1)}` : ''}. Times in {here.replace('_', ' ')}.
					</p>
					{#if days.length === 0}
						<p>Nothing is open in the coming weeks. Check back soon.</p>
					{:else}
						<div class="days" role="tablist" aria-label="Day">
							{#each days as x (x.day)}
								<button role="tab" aria-selected={x.day === day} class:on={x.day === day} onclick={() => { day = x.day; chosen = null }}>{x.label}</button>
							{/each}
						</div>
						<div class="times" role="listbox" aria-label="Time">
							{#each times as t (t)}
								<button role="option" aria-selected={chosen === t} class:on={chosen === t} onclick={() => (chosen = t)}>{clock(t, here)}</button>
							{/each}
						</div>
						<label class="about">
							<span>What would you like to talk about? <span class="quiet">Optional</span></span>
							<textarea bind:value={about} rows="3" maxlength={ABOUT_MOST} placeholder="An audit coming up, agents in production, a network nobody documented…"></textarea>
							{#if about.length > ABOUT_MOST * 0.8}<span class="quiet small count">{about.length}/{ABOUT_MOST}</span>{/if}
						</label>
						<button class="go" disabled={chosen === null || busy === 'booking'} onclick={confirm}>
							{#if busy === 'booking'}Booking…{:else if chosen === null}Choose a time{:else}{moving ? 'Move to' : 'Book'} {when(chosen, offer.minutes, here)}{/if}
						</button>
						{#if chosen !== null && here !== AMSTERDAM}
							<p class="quiet small">That is {clock(chosen, AMSTERDAM)} in Amsterdam.</p>
						{/if}
					{/if}
				{/if}
				{#if moving}<button class="link back" onclick={() => (moving = null)}>Keep {when(moving.start, moving.minutes, here)}</button>{/if}
			{:else if call}
				<h2 id="booking-title">Your call</h2>
				<p class="call">{when(call.start, call.minutes, here)}</p>
				{#if here !== AMSTERDAM}<p class="quiet small">{clock(call.start, AMSTERDAM)} in Amsterdam.</p>{/if}
				{#if offer?.place}<p class="quiet">{offer.place}</p>{/if}
				{#if call.about}<blockquote>{call.about}</blockquote>{/if}
				{#if told}
					<p class="small" class:warn={told.you !== true}>{toldYou(told.you)}</p>
				{/if}
				{#if confirmCancel}
					<div class="row">
						<span>Cancel this call?</span>
						<button class="danger" onclick={drop} disabled={busy === 'cancelling'}>{busy === 'cancelling' ? 'Cancelling…' : 'Yes, cancel'}</button>
						<button class="link" onclick={() => (confirmCancel = false)}>Keep it</button>
					</div>
				{:else}
					<div class="row">
						<button onclick={calendarFile}>Add to calendar</button>
						<button onclick={move}>Move</button>
						<button class="link" onclick={() => (confirmCancel = true)}>Cancel</button>
					</div>
				{/if}
			{/if}

			{#if admission}
				<p class="quiet small proof">Signed in as {admission.admitted_as} · SBVH namespace on QNTX</p>
			{/if}
		{/if}

		{#if error}
			<p class="error" role="alert">{error}</p>
		{/if}
	</div>
{/if}

<style>
	.me {
		position: fixed;
		top: 1rem;
		right: 1rem;
		z-index: 1500;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 1px solid var(--border);
		background: var(--bg-secondary);
		color: var(--text);
		padding: 0;
		overflow: hidden;
		cursor: pointer;
		font-family: var(--font-mono);
		font-size: var(--font-size-md);
	}

	.me img, .who img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 200000;
		background: rgba(10, 12, 11, 0.35);
		backdrop-filter: blur(2px);
		animation: fade 0.15s ease;
	}

	.panel {
		position: fixed;
		z-index: 200001;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		width: min(440px, calc(100vw - 32px));
		max-height: calc(100dvh - 32px);
		overflow: auto;
		background: var(--bg);
		color: var(--text);
		border: 1px solid var(--border);
		border-radius: var(--border-radius-lg);
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.18);
		padding: 1.5rem;
		font-family: var(--font-mono);
		font-size: var(--font-size-md);
		line-height: 1.6;
		animation: rise 0.18s ease;
		outline: none;
	}

	@media (max-width: 480px) {
		.panel {
			top: auto;
			bottom: 0;
			transform: translateX(-50%);
			width: 100vw;
			border-radius: var(--border-radius-lg) var(--border-radius-lg) 0 0;
			border-bottom: none;
			padding-bottom: calc(1.5rem + env(safe-area-inset-bottom));
		}
	}

	h2 {
		font-family: Georgia, 'Times New Roman', serif;
		font-weight: 400;
		font-size: 1.35rem;
		letter-spacing: 0.02em;
		margin: 0 0 0.75rem;
	}

	p { margin: 0 0 0.75rem; }

	.quiet { color: var(--text-secondary); }
	.small { font-size: var(--font-size-sm); }
	.working::after { content: ''; display: inline-block; width: 1em; animation: dots 1.2s steps(4) infinite; overflow: hidden; vertical-align: bottom; }

	.x {
		position: absolute;
		top: 0.5rem;
		right: 0.5rem;
		width: 32px;
		height: 32px;
		border: none;
		background: none;
		color: var(--text-tertiary);
		font-size: 20px;
		cursor: pointer;
	}
	.x:hover { color: var(--text); }

	button {
		font-family: inherit;
		font-size: inherit;
		color: var(--text);
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 0.45rem 0.75rem;
		cursor: pointer;
		transition: border-color 0.15s ease, background 0.15s ease, color 0.15s ease;
	}
	button:hover:not(:disabled) { border-color: var(--text-tertiary); }
	button:disabled { cursor: default; opacity: 0.55; }
	button:focus-visible, textarea:focus-visible, a:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

	.link {
		background: none;
		border: none;
		padding: 0;
		color: var(--text-secondary);
		text-decoration: underline;
		text-underline-offset: 3px;
	}
	.link:hover:not(:disabled) { color: var(--text); }

	a { color: var(--accent); text-decoration: none; }
	a:hover { color: var(--accent-hover); }

	.ways { display: flex; flex-direction: column; gap: 0.5rem; margin: 1rem 0; }
	.way { width: 100%; padding: 0.7rem 1rem; text-align: center; }
	.placeholder { height: 2.6rem; border: 1px solid var(--border); border-radius: var(--border-radius); background: var(--bg-secondary); animation: pulse 1.2s ease-in-out infinite; }

	.who { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 1rem; padding-right: 2rem; font-size: var(--font-size-sm); }
	.who img, .face { width: 24px; height: 24px; border-radius: 50%; overflow: hidden; flex-shrink: 0; }
	.face { display: inline-flex; align-items: center; justify-content: center; background: var(--bg-secondary); border: 1px solid var(--border); }
	.name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text-secondary); }

	.days { display: flex; gap: 0.4rem; overflow-x: auto; padding-bottom: 0.4rem; margin: 0 -0.25rem 0.5rem; padding-inline: 0.25rem; scrollbar-width: thin; }
	.days button { flex-shrink: 0; font-size: var(--font-size-sm); }
	.times { display: grid; grid-template-columns: repeat(auto-fill, minmax(72px, 1fr)); gap: 0.4rem; margin-bottom: 1rem; }
	.on { border-color: var(--accent) !important; color: var(--accent); background: var(--bg); }

	.about { display: flex; flex-direction: column; gap: 0.35rem; margin-bottom: 1rem; font-size: var(--font-size-sm); }
	textarea {
		font-family: inherit;
		font-size: var(--font-size-md);
		color: var(--text);
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 0.5rem;
		resize: vertical;
	}
	.count { align-self: flex-end; }

	.go { width: 100%; padding: 0.7rem 1rem; background: var(--accent); border-color: var(--accent); color: var(--bg); }
	.go:hover:not(:disabled) { background: var(--accent-hover); border-color: var(--accent-hover); }
	.go:disabled { background: var(--bg-secondary); border-color: var(--border); color: var(--text-tertiary); opacity: 1; }
	.back { margin-top: 0.75rem; }

	.call { font-size: var(--font-size-lg); }
	blockquote { margin: 0 0 1rem; padding-left: 0.75rem; border-left: 2px solid var(--border); color: var(--text-secondary); white-space: pre-wrap; }
	.row { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; margin-top: 1rem; }
	.danger { border-color: #b4553f; color: #b4553f; }

	.proof { margin: 1.25rem 0 0; padding-top: 0.75rem; border-top: 1px solid var(--border); color: var(--text-tertiary); word-break: break-all; }
	.warn { color: #a06a1c; }
	.error { margin: 1rem 0 0; padding: 0.6rem 0.75rem; border: 1px solid #b4553f; border-radius: var(--border-radius); color: #b4553f; font-size: var(--font-size-sm); white-space: pre-wrap; word-break: break-word; }

	@keyframes fade { from { opacity: 0; } }
	@keyframes rise { from { opacity: 0; margin-top: 8px; } }
	@keyframes pulse { 50% { opacity: 0.5; } }
	@keyframes dots { from { width: 0; } to { width: 1.25em; } }

	@media (prefers-reduced-motion: reduce) {
		.backdrop, .panel, .placeholder, .working::after { animation: none; }
	}
</style>
