<script lang="ts">
	import { onMount } from 'svelte'
	import { tray, configureElements } from '@teranos/elements'
	import type { Element } from '@teranos/elements'

	const isMobile = window.matchMedia('(max-width: 768px)').matches

	interface ExtraLink { label: string; url: string }

	// Window width is border-box. Subtract the 1px border per side and the
	// engine's 8px content padding per side to get the exact line box the text
	// wraps in — the off-screen measure must wrap identically or the window
	// commits to the wrong height.
	const WINDOW_WIDTH = 320
	const CONTENT_WIDTH = WINDOW_WIDTH - 2 - 16

	function project(id: string, title: string, url: string, subtitle: string, description: string, showAttestation = false, extraLinks: ExtraLink[] = []): Element & { url: string } {
		return {
			id, title, url,
			opensAs: isMobile ? 'panel' : 'window',
			// No size: an element never declares how big it is. The window
			// measures its content, which is capped at CONTENT_WIDTH below.
			color: 'var(--glyph-fill)',
			textColor: 'var(--text)',
			renderContent: () => projectContent(subtitle, description, url, showAttestation, extraLinks),
		}
	}

	const projects = [
		project('qntx', 'QNTX', 'https://github.com/teranos/QNTX', 'Experience Learning', 'Turns complex, high-dimensional data into something you can see and reason about.', true),
		project('ground', 'Ground', 'https://github.com/teranos/ground', 'Mission Control for AI', 'Grounds coding assistants to not drift off too much.'),
		project('loom', 'Loom', 'https://github.com/teranos/QNTX', 'Conversation Weaving', 'Browse what you and your coding assistant actually did — every prompt, response, and action, woven together.'),
		project('scry', 'Scry', 'https://github.com/teranos/QNTX', 'Live Inference', '3D shader visualization for local model inference.'),
		project('werf', 'Werf', 'https://github.com/teranos/QNTX', 'Job Matching', 'Your CV is the interface. Upload it, see where you could go next, and get matched to roles that fit your trajectory.'),
		project('elements', 'Elements', 'https://jsr.io/@teranos/elements', 'UI Primitive', 'An interface element with a permanent identity that smoothly changes form. This window is one.'),
		project('cleanamsterdam', 'Clean Amsterdam', 'https://cleanamsterdam.nl', 'House Cleaning', 'Amsterdam house cleaning service.'),
		project('laye', 'Laye', 'https://laye.sbvh.nl', 'Peer-to-Peer WASM', 'libp2p as a WASM module you drop into anything that needs to reach peers directly. Iroh-shaped, but embeddable wherever you already work.'),
		project('game', 'Game', 'https://game.sbvh.nl', 'Serverless Multiplayer', 'Rust and Bevy, rendering Cataclysm: Dark Days Ahead mapgen assets. Players connect peer-to-peer through Laye, so there is no server to run.', false, [
			{ label: 'Source →', url: 'https://github.com/teranos/tsot/tree/master/game' },
			{ label: 'Cataclysm: DDA →', url: 'https://cataclysmdda.org' },
		]),
	]

	function projectContent(subtitle: string, description: string, url: string, showAttestation = false, extraLinks: ExtraLink[] = []): HTMLElement {
		const el = document.createElement('div')
		el.className = 'glyph-content'
		// Caps the line box so the engine's off-screen measure wraps exactly as
		// the real window does — otherwise it measures one unwrapped line.
		el.style.maxWidth = `${CONTENT_WIDTH}px`

		const sub = document.createElement('div')
		sub.style.cssText = 'color: var(--text-secondary); margin-bottom: 8px;'
		sub.textContent = subtitle
		el.appendChild(sub)

		const desc = document.createElement('div')
		desc.style.cssText = 'color: var(--text-tertiary); margin-bottom: 12px;'
		desc.textContent = description
		el.appendChild(desc)

		if (showAttestation) {
			const ats = document.createElement('pre')
			ats.style.cssText = 'color: var(--text-tertiary); margin-bottom: 12px; white-space: pre; font-family: inherit; font-size: inherit;'
			ats.innerHTML = `<span style="color: var(--text-secondary)">Attestation</span> — a structured claim\n<span style="color: var(--accent)">+ as</span>  [subject]\n<span style="color: var(--accent)">= is</span>  [predicate]\n<span style="color: var(--accent)">∈ of</span>  [context]\n<span style="color: var(--accent)">⊢ by</span>  [actor]\n<span style="color: var(--accent)">✦ at</span>  [time]\n      {}`
			el.appendChild(ats)
		}

		const link = document.createElement('a')
		link.href = url
		link.target = '_blank'
		link.rel = 'noopener noreferrer'
		link.textContent = url.includes('github.com') ? 'View on GitHub →'
			: url.includes('jsr.io') ? 'View on JSR →'
			: 'Open →'
		link.style.cssText = 'color: var(--accent); text-decoration: none;'

		const row = document.createElement('div')
		row.style.cssText = 'display: flex; flex-wrap: wrap; gap: 12px;'
		row.appendChild(link)

		for (const extra of extraLinks) {
			const a = document.createElement('a')
			a.href = extra.url
			a.target = '_blank'
			a.rel = 'noopener noreferrer'
			a.textContent = extra.label
			a.style.cssText = 'color: var(--accent); text-decoration: none;'
			row.appendChild(a)
		}

		el.appendChild(row)

		return el
	}

	onMount(() => {
		// Resting dot doubled from the 10px default. Expanded size is unchanged,
		// so the dot travels less distance to reach it.
		configureElements({ dotGeometry: { minWidth: 20, minHeight: 20 } })
		tray.init()

		for (const project of projects) {
			tray.add(project)
		}
	})
</script>

<style>
	/* The tray's place, layout and motion are @teranos/elements'. Its layer is the site's. */
	:global(.tray) {
		z-index: 1000;
	}

	@media (max-width: 768px) {
		:global(.tray) {
			z-index: 10004;
		}
	}

	/* Dot theme */
	:global(.dot) {
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: 3px;
	}

	/* Morph transitions */
	:global(.morphing) {
		position: fixed !important;
		z-index: 1001 !important;
		pointer-events: auto;
	}

	/* Window and panel surface. !important because the engine writes
	   backgroundColor inline from the element's color, which is the dot colour. */
	:global([data-form="window"]),
	:global([data-form="panel"]) {
		background-color: var(--glyph-surface) !important;
		border: 1px solid var(--border);
	}

	/* Window title bar */
	:global(.title-bar) {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 0 8px;
		background-color: var(--glyph-titlebar);
		border-bottom: 1px solid var(--border);
		border-radius: 4px 4px 0 0;
		height: 32px;
		width: 100%;
		flex-shrink: 0;
		user-select: none;
		cursor: move;
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--text);
	}

	/* Window control buttons */
	:global(.title-bar button) {
		width: 22px;
		height: 22px;
		padding: 1px 2px;
		font-size: 13px;
		background: var(--glyph-surface);
		color: var(--text-secondary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		line-height: 1;
		margin-left: 2px;
		transition: background 0.15s ease;
	}

	:global(.title-bar button:hover) {
		background: var(--border);
		color: var(--text);
	}

	@media (pointer: coarse) {
		:global(.title-bar) {
			height: 44px;
		}
		:global(.title-bar button) {
			width: 44px;
			height: 44px;
			font-size: 17px;
		}
	}

	/* Hover/touch-hold feedback — brighter while pointing */
	:global(.dot:hover),
	:global(.dot:active) {
		filter: brightness(1.3);
	}

	/* Window content. `flex: 1 1 auto` — basis auto so the content area reports
	   its real height to a fit-content window, and still fills a fixed-height
	   mobile panel. */
	:global(.content-area) {
		flex: 1 1 auto;
		overflow: auto;
		padding: 8px;
	}

	/* No padding here — the content area already pads 8px, which is the title
	   bar's own indent. A second padding put body text 4px off the title. */
	:global(.glyph-content) {
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		line-height: 1.6;
	}

	/* Reduced motion */
	@media (prefers-reduced-motion: reduce) {
		:global(.tray-dots),
		:global(.morphing),
		:global(.dot) {
			transition-duration: 0s !important;
			animation-duration: 0s !important;
		}
	}
</style>
