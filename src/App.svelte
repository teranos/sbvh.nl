<script lang="ts">
	import GlyphTray from './GlyphTray.svelte'

	let inverted = $state(false)

	function toggleTheme() {
		inverted = !inverted
		document.documentElement.classList.toggle('inverted', inverted)
	}

	// ── Beam tree ────────────────────────────────────────────────────────
	// Each beam is a coherent path through the intro sentence.
	// Selecting a slot filters downstream candidates to beams that match.

	const slotKeys = ['role', 'domain', 'scope', 'verb', 'focus', 'craft', 'mission'] as const
	type SlotKey = typeof slotKeys[number]
	type Beam = Record<SlotKey, string> & { prob: number }

	const beams: Beam[] = [
		{ prob: 0.25, role: 'Systems engineer', domain: 'infrastructure', scope: 'at scale', verb: 'Now working on', focus: 'AI that runs on your own machine', craft: 'visual programming languages', mission: 'genomic emancipation' },
		{ prob: 0.15, role: 'Systems architect', domain: 'ML infrastructure', scope: 'at scale', verb: 'Building tools for', focus: 'AI interpretability', craft: 'visual programming languages', mission: 'genomic sovereignty' },
		{ prob: 0.15, role: 'SRE', domain: 'infrastructure', scope: 'and resilience', verb: 'Building tools for', focus: 'local inference', craft: 'automation', mission: 'data-sovereign bio-infrastructure' },
		{ prob: 0.12, role: 'DevOps', domain: 'infrastructure', scope: 'as code', verb: 'Building tools for', focus: 'local AI', craft: 'automation', mission: 'secure health data' },
		{ prob: 0.10, role: 'Full stack', domain: 'DevSecOps', scope: 'and data privacy', verb: 'Now working on', focus: 'AI that runs on your own machine', craft: 'cloud architecture', mission: 'personal genomics' },
		{ prob: 0.10, role: 'Founder', domain: 'genomic computing', scope: 'and open platforms', verb: 'Now working on', focus: 'data-sovereign bio-infrastructure', craft: 'secure health data', mission: 'genomic emancipation' },
		{ prob: 0.08, role: 'Systems engineer', domain: 'ML infrastructure', scope: 'as code', verb: 'Now working on', focus: 'local inference', craft: 'visual programming languages', mission: 'genomic sovereignty' },
		{ prob: 0.05, role: 'Systems architect', domain: 'infrastructure', scope: 'and resilience', verb: 'Now working on', focus: 'AI that runs on your own machine', craft: 'cloud architecture', mission: 'genomic emancipation' },
	]

	// Sentence template — segments reference slot keys or static text
	const template: ({ slot: SlotKey } | { text: string })[] = [
		{ slot: 'role' },
		{ text: '. Background in ' },
		{ slot: 'domain' },
		{ text: ' ' },
		{ slot: 'scope' },
		{ text: '. ' },
		{ slot: 'verb' },
		{ text: ' ' },
		{ slot: 'focus' },
		{ text: ', ' },
		{ slot: 'craft' },
		{ text: ', and ' },
		{ slot: 'mission' },
		{ text: '.' },
	]

	// ── Context seeding ──────────────────────────────────────────────────
	function seedBeam(): Beam {
		const params = new URLSearchParams(window.location.search)
		const roleParam = params.get('role')
		if (roleParam) {
			const match = beams.find(b => b.role.toLowerCase() === roleParam.toLowerCase())
			if (match) return match
		}
		const ref = document.referrer
		if (ref.includes('genomicos')) return beams.find(b => b.role === 'Founder')!
		if (ref.includes('github.com')) return beams.find(b => b.role === 'Systems engineer')!
		return beams[0]
	}

	const seed = seedBeam()
	let selected: Record<SlotKey, string> = $state(
		Object.fromEntries(slotKeys.map(k => [k, seed[k]])) as Record<SlotKey, string>
	)

	// ── Beam filtering ───────────────────────────────────────────────────
	// For a given slot, filter beams by all upstream selections,
	// then return unique candidates with normalized probabilities.
	function candidatesFor(key: SlotKey): { text: string; prob: number }[] {
		const idx = slotKeys.indexOf(key)
		let matching = beams
		for (let i = 0; i < idx; i++) {
			const up = slotKeys[i]
			matching = matching.filter(b => b[up] === selected[up])
		}
		const groups: Record<string, number> = {}
		for (const b of matching) {
			groups[b[key]] = (groups[b[key]] || 0) + b.prob
		}
		const total = Object.values(groups).reduce((a, b) => a + b, 0)
		return Object.entries(groups)
			.map(([text, prob]) => ({ text, prob: total > 0 ? prob / total : 0 }))
			.sort((a, b) => b.prob - a.prob)
	}

	// When a slot changes, cascade: snap downstream slots that are
	// no longer reachable to the highest-probability option.
	function selectSlot(key: SlotKey, value: string) {
		selected[key] = value
		const idx = slotKeys.indexOf(key)
		for (let i = idx + 1; i < slotKeys.length; i++) {
			const candidates = candidatesFor(slotKeys[i])
			if (!candidates.some(c => c.text === selected[slotKeys[i]])) {
				selected[slotKeys[i]] = candidates[0]?.text ?? ''
			}
		}
		popup = null
	}

	// ── Popup ────────────────────────────────────────────────────────────
	let popup = $state<{ key: SlotKey; x: number; y: number } | null>(null)
	let popupEl: HTMLDivElement | undefined = $state()
	let hideTimer: ReturnType<typeof setTimeout> | null = null

	function togglePopup(key: SlotKey, e: MouseEvent) {
		if (popup?.key === key) { popup = null } else { showPopup(key, e) }
	}

	function showPopup(key: SlotKey, e: MouseEvent) {
		if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
		const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
		popup = { key, x: rect.left, y: rect.bottom + 4 }
	}

	function scheduleHide() {
		if (hideTimer) clearTimeout(hideTimer)
		hideTimer = setTimeout(() => { popup = null }, 150)
	}

	function cancelHide() {
		if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
	}

	$effect(() => {
		if (popup && popupEl) {
			const rect = popupEl.getBoundingClientRect()
			if (rect.right > window.innerWidth - 4) {
				popup.x = window.innerWidth - rect.width - 4
			}
			if (rect.bottom > window.innerHeight - 4) {
				popup.y = popup.y - rect.height - 30
			}
		}
	})
</script>

<main>
	<div class="content">
		<button class="logo" onclick={toggleTheme} aria-label="Toggle color mode">
			<img src="/logo.svg" alt="SBvH" width="160" height="160" />
		</button>

		<h1>Sebastiaan Brandon van Houten</h1>

		<p class="intro">{#each template as seg}{'slot' in seg ? '' : ''}{#if 'slot' in seg}<span
				class="logit-span"
				class:logit-single={candidatesFor(seg.slot).length <= 1}
				onmouseenter={(e) => showPopup(seg.slot, e)}
				onmouseleave={scheduleHide}
				onclick={(e) => togglePopup(seg.slot, e)}
			>{selected[seg.slot]}</span>{:else}{seg.text}{/if}{/each}</p>

		<p class="label">Previously at</p>

		<div class="companies">
			<a href="https://dtact.com" target="_blank" rel="noopener noreferrer" aria-label="dtact">
				<img src="/companies/dtact.png" alt="dtact" width="28" height="28" />
			</a>
			<a href="https://www.ing.com/Newsroom/News/Yolt-to-phase-out-its-business-to-business-open-banking-operations.htm" target="_blank" rel="noopener noreferrer" aria-label="ING">
				<img src="/companies/ING.png" alt="ING" width="28" height="28" />
			</a>
			<a href="https://www.sap.com" target="_blank" rel="noopener noreferrer" aria-label="SAP">
				<img src="/companies/SAP.ico" alt="SAP" width="28" height="28" />
			</a>
		</div>
	</div>

	{#if popup && candidatesFor(popup.key).length > 1}
		<div
			class="token-popup"
			bind:this={popupEl}
			style="left: {popup.x}px; top: {popup.y}px;"
			onmouseenter={cancelHide}
			onmouseleave={scheduleHide}
		>
			{#each candidatesFor(popup.key) as candidate}
				<button
					class="token-popup-candidate"
					class:token-popup-chosen={selected[popup.key] === candidate.text}
					onclick={() => selectSlot(popup!.key, candidate.text)}
				>
					<span class="token-popup-token-text">{candidate.text}</span>
					<div class="token-popup-bar-track">
						<div class="token-popup-bar" style="width: {candidate.prob * 100}%"></div>
					</div>
					<span class="token-popup-prob">{(candidate.prob * 100).toFixed(0)}%</span>
				</button>
			{/each}
		</div>
	{/if}

	<nav class="social">
		<a href="/calendly" aria-label="Schedule a time" title="Schedule a time">
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/><path d="M8 14h.01"/><path d="M12 14h.01"/><path d="M16 14h.01"/><path d="M8 18h.01"/><path d="M12 18h.01"/><path d="M16 18h.01"/></svg>
		</a>
		<a href="https://x.com/01F1R3" target="_blank" rel="noopener noreferrer" aria-label="X">
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M22 4s-.7 2.1-2 3.4c1.6 10-9.4 17.3-18 11.6 2.2.1 4.4-.6 6-2C3 15.5.5 9.6 3 5c2.2 2.6 5.6 4.1 9 4-.9-4.2 4-6.6 7-3.8 1.1 0 3-1.2 3-1.2z"/></svg>
		</a>
		<a href="https://t.me/sb_vh" target="_blank" rel="noopener noreferrer" aria-label="Telegram">
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M7.9 20A9 9 0 1 0 4 16.1L2 22z"/></svg>
		</a>
		<a href="https://www.linkedin.com/in/sbvh/" target="_blank" rel="noopener noreferrer" aria-label="LinkedIn">
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z"/><rect width="4" height="12" x="2" y="9"/><circle cx="4" cy="4" r="2"/></svg>
		</a>
		<a href="https://github.com/teranos" target="_blank" rel="noopener noreferrer" aria-label="GitHub">
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"/><path d="M9 18c-4.51 2-5-2-7-2"/></svg>
		</a>
		<a href="https://www.genomicos.com" target="_blank" rel="noopener noreferrer" aria-label="GenomicOS">
			<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M2 15c6.667-6 13.333 0 20-6"/><path d="M9 22c1.798-1.998 2.518-3.995 2.807-5.993"/><path d="M15 2c-1.798 1.998-2.518 3.995-2.807 5.993"/><path d="m17 6-2.5-2.5"/><path d="m14 8-1-1"/><path d="m7 18 2.5 2.5"/><path d="m3.5 14.5.5.5"/><path d="m10 16 1.5 1.5"/></svg>
		</a>
	</nav>
</main>

<GlyphTray />

<style>
	main {
		display: flex;
		flex-direction: column;
		align-items: center;
		min-height: 100dvh;
		padding: 2rem;
		padding-bottom: 5rem;
	}

	.content {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: 1rem;
		max-width: 440px;
	}

	.logo {
		display: block;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		color: var(--text);
	}

	.logo img {
		width: 160px;
		height: 160px;
		transition: filter 0.2s ease;
	}

	@media (prefers-color-scheme: dark) {
		.logo img { filter: invert(0.7); }
	}

	@media (prefers-color-scheme: light) {
		:global(.inverted) .logo img { filter: invert(0.7); }
	}

	@media (prefers-color-scheme: dark) {
		:global(.inverted) .logo img { filter: invert(0); }
	}

	h1 {
		font-family: Georgia, 'Times New Roman', serif;
		font-size: 1.5rem;
		font-weight: 400;
		letter-spacing: 0.02em;
		text-align: center;
		color: var(--text);
	}

	.intro {
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		text-align: center;
		max-width: 480px;
		line-height: 1.6;
	}

	.logit-span {
		color: var(--text);
		cursor: pointer;
		transition: outline-color 0.15s ease;
		outline: 1px solid transparent;
	}

	.logit-span:hover {
		outline: 1px solid rgba(255, 220, 100, 0.35);
	}

	.logit-single {
		cursor: default;
		color: var(--text-secondary);
	}

	.logit-single:hover {
		outline-color: transparent;
	}

	.token-popup {
		position: fixed;
		z-index: 100000;
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		padding: 6px 8px;
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		color: var(--text);
		min-width: 180px;
		max-width: 320px;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.token-popup-candidate {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 22px;
		background: none;
		border: none;
		padding: 0 2px;
		font-family: inherit;
		font-size: inherit;
		color: var(--text-tertiary);
		cursor: pointer;
		width: 100%;
		text-align: left;
	}

	.token-popup-candidate:hover {
		color: var(--text);
	}

	.token-popup-chosen {
		color: var(--text);
	}

	.token-popup-token-text {
		flex: 1;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		font-size: 11px;
	}

	.token-popup-bar-track {
		width: 60px;
		height: 8px;
		background: var(--border);
		flex-shrink: 0;
		position: relative;
	}

	.token-popup-bar {
		height: 100%;
		background: var(--accent);
	}

	.token-popup-prob {
		width: 28px;
		text-align: right;
		flex-shrink: 0;
		font-size: 10px;
		color: var(--text-tertiary);
	}

	.label {
		font-family: Georgia, 'Times New Roman', serif;
		font-size: var(--font-size-md);
		letter-spacing: 0.05em;
		color: var(--text-secondary);
	}

.companies {
		display: flex;
		gap: 1rem;
	}

	.companies a {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border: 1px solid var(--border);
		padding: 0.5rem;
		transition: all 0.3s ease;
		filter: grayscale(1);
		opacity: 0.6;
	}

	.companies a:hover {
		filter: grayscale(0);
		opacity: 1;
	}

	.companies img {
		width: 100%;
		height: 100%;
		object-fit: contain;
	}

	.social {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		display: flex;
		justify-content: center;
		gap: 2rem;
		padding: 1rem;
		border-top: 1px solid var(--border);
		background: var(--bg);
	}

	.social a {
		color: var(--text-tertiary);
		text-decoration: none;
		transition: color 0.2s ease;
	}

	.social a:hover {
		color: var(--text);
	}

	@media (max-width: 480px) {
		.logo img { width: 96px; height: 96px; }
		h1 { font-size: 1.25rem; }
		.intro { font-size: var(--font-size-md); }
		.social { gap: 1.5rem; }
	}
</style>
