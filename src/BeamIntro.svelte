<script lang="ts">
	// ── Beam tree ────────────────────────────────────────────────────────
	// Each beam is a coherent path through the intro sentence.
	// Selecting a slot filters downstream candidates to beams that match.

	const slotKeys = ['role', 'domain', 'scope', 'verb', 'focus', 'craft', 'mission'] as const
	type SlotKey = typeof slotKeys[number]
	type Beam = Record<SlotKey, string> & { prob: number }

	const beams: Beam[] = [
		{ prob: 0.25, role: 'Systems engineer', domain: 'infrastructure', scope: 'at scale', verb: 'Now working on', focus: 'AI that runs on your own machine', craft: 'visual programming languages', mission: 'genomic sovereignty' },
		{ prob: 0.15, role: 'Systems architect', domain: 'ML infrastructure', scope: 'at scale', verb: 'Building tools for', focus: 'AI interpretability', craft: 'visual programming languages', mission: 'genomic sovereignty' },
		{ prob: 0.15, role: 'SRE', domain: 'infrastructure', scope: 'and resilience', verb: 'Building tools for', focus: 'local inference', craft: 'automation', mission: 'data-sovereign bio-infrastructure' },
		{ prob: 0.12, role: 'DevOps', domain: 'infrastructure', scope: 'as code', verb: 'Building tools for', focus: 'local AI', craft: 'automation', mission: 'secure health data' },
		{ prob: 0.10, role: 'Full stack', domain: 'DevSecOps', scope: 'and data privacy', verb: 'Now working on', focus: 'AI that runs on your own machine', craft: 'cloud architecture', mission: 'personal genomics' },
		{ prob: 0.10, role: 'Founder', domain: 'genomic computing', scope: 'and open platforms', verb: 'Now working on', focus: 'data-sovereign bio-infrastructure', craft: 'secure health data', mission: 'genomic emancipation' },
		{ prob: 0.08, role: 'Systems engineer', domain: 'ML infrastructure', scope: 'at scale', verb: 'Now working on', focus: 'local inference', craft: 'visual programming languages', mission: 'genomic sovereignty' },
		{ prob: 0.05, role: 'Systems architect', domain: 'infrastructure', scope: 'and resilience', verb: 'Now working on', focus: 'AI that runs on your own machine', craft: 'cloud architecture', mission: 'personal genomics' },
		{ prob: 0.07, role: 'SRE', domain: 'MLOps', scope: 'at scale', verb: 'Building tools for', focus: 'local inference', craft: 'automation', mission: 'genomic sovereignty' },
		{ prob: 0.05, role: 'Systems engineer', domain: 'MLOps', scope: 'at scale', verb: 'Building tools for', focus: 'AI interpretability', craft: 'automation', mission: 'data-sovereign bio-infrastructure' },
		{ prob: 0.04, role: 'SRE', domain: 'infrastructure', scope: 'at scale', verb: 'Building tools for', focus: 'local inference', craft: 'CI/CD', mission: 'data-sovereign bio-infrastructure' },
		{ prob: 0.04, role: 'DevOps', domain: 'infrastructure', scope: 'as code', verb: 'Building tools for', focus: 'local AI', craft: 'CI/CD', mission: 'secure health data' },
		{ prob: 0.03, role: 'Systems engineer', domain: 'infrastructure', scope: 'at scale', verb: 'Building tools for', focus: 'local inference', craft: 'CI/CD', mission: 'genomic sovereignty' },
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

	// ── City (independent of beam tree) ──────────────────────────────────
	const cities = [
		{ text: '', prob: 0.35 },
		{ text: 'Amsterdam', prob: 0.15 },
		{ text: 'Berlin', prob: 0.12 },
		{ text: 'London', prob: 0.10 },
		{ text: 'San Francisco', prob: 0.10 },
		{ text: 'Paris', prob: 0.08 },
		{ text: 'Remote', prob: 0.10 },
	]

	function seedCity(): string {
		try {
			const tz = Intl.DateTimeFormat().resolvedOptions().timeZone
			if (tz.includes('Amsterdam')) return 'Amsterdam'
			if (tz.includes('Berlin')) return 'Berlin'
			if (tz.includes('London')) return 'London'
			if (tz.includes('Paris')) return 'Paris'
			if (tz.includes('Los_Angeles') || tz.includes('San_Francisco')) return 'San Francisco'
		} catch {}
		return ''
	}

	let city = $state(seedCity())

	// ── Contextual links: beam + city ────────────────────────────────────
	const aiKeywords = ['AI that runs on your own machine', 'local AI', 'local inference', 'AI interpretability']
	interface ContextLink { label: string; url: string }

	let contextLinks = $derived.by((): ContextLink[] => {
		const links: ContextLink[] = []
		const isAi = aiKeywords.includes(selected.focus)
		if (selected.domain === 'MLOps' && city === 'Amsterdam')
			links.push({ label: 'Amsterdam MLOps →', url: 'https://luma.com/amsterdam-mlops' })
		if (isAi && city === 'Amsterdam' && selected.domain !== 'MLOps')
			links.push({ label: 'AI House Amsterdam →', url: 'https://luma.com/AIHouseAmsterdam' })
		if (isAi && city === 'San Francisco')
			links.push({ label: 'GenAI SF →', url: 'https://luma.com/genai-sf' })
		if (selected.role === 'Founder' && city === 'San Francisco')
			links.push({ label: 'Frontier Tower →', url: 'https://luma.com/frontiertower' })
		if (selected.role === 'Founder' && city === 'Paris')
			links.push({ label: 'Station F →', url: 'https://stationf.co' })
		if (selected.role === 'SRE')
			links.push({ label: 'SREday →', url: 'https://sreday.com/' })
		return links
	})

	// ── Popup ────────────────────────────────────────────────────────────
	let popup = $state<{ key: SlotKey | 'city'; x: number; y: number } | null>(null)
	let popupEl: HTMLDivElement | undefined = $state()
	let hideTimer: ReturnType<typeof setTimeout> | null = null

	function togglePopup(key: SlotKey | 'city', e: MouseEvent) {
		if (popup?.key === key) { popup = null } else { showPopup(key, e) }
	}

	function showPopup(key: SlotKey | 'city', e: MouseEvent) {
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

<p class="intro">{#each template as seg}{'slot' in seg ? '' : ''}{#if 'slot' in seg}<span
		class="logit-span"
		class:logit-single={candidatesFor(seg.slot).length <= 1}
		onmouseenter={(e) => showPopup(seg.slot, e)}
		onmouseleave={scheduleHide}
		onclick={(e) => togglePopup(seg.slot, e)}
	>{selected[seg.slot]}</span>{:else}{seg.text}{/if}{/each}{#if city}{' '}Based in{' '}<span class="logit-span" onmouseenter={(e) => showPopup('city', e)} onmouseleave={scheduleHide} onclick={(e) => togglePopup('city', e)}>{city}</span>.{:else}<span class="logit-span logit-empty" onmouseenter={(e) => showPopup('city', e)} onmouseleave={scheduleHide} onclick={(e) => togglePopup('city', e)}>&ZeroWidthSpace;</span>{/if}</p>

{#each contextLinks as link}
	<a class="context-link" href={link.url} target="_blank" rel="noopener noreferrer">{link.label}</a>
{/each}

{#if popup}
	{@const items = popup.key === 'city' ? cities : candidatesFor(popup.key)}
	{@const current = popup.key === 'city' ? city : selected[popup.key]}
	{#if items.length > 1}
		<div
			class="token-popup"
			bind:this={popupEl}
			style="left: {popup.x}px; top: {popup.y}px;"
			onmouseenter={cancelHide}
			onmouseleave={scheduleHide}
		>
			{#each items as candidate}
				<button
					class="token-popup-candidate"
					class:token-popup-chosen={current === candidate.text}
					onclick={() => {
						if (popup!.key === 'city') { city = candidate.text; popup = null }
						else { selectSlot(popup!.key as SlotKey, candidate.text) }
					}}
				>
					<span class="token-popup-token-text">{candidate.text || '·'}</span>
					<div class="token-popup-bar-track">
						<div class="token-popup-bar" style="width: {candidate.prob * 100}%"></div>
					</div>
					<span class="token-popup-prob">{(candidate.prob * 100).toFixed(0)}%</span>
				</button>
			{/each}
		</div>
	{/if}
{/if}

<style>
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

	.logit-empty {
		display: inline-block;
		width: 1em;
		vertical-align: baseline;
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

	.context-link {
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		color: var(--accent);
		text-decoration: none;
		transition: color 0.2s ease;
	}

	.context-link:hover {
		color: var(--accent-hover);
	}

	@media (max-width: 480px) {
		.intro { font-size: var(--font-size-md); }
	}
</style>
