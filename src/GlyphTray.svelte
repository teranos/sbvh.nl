<script lang="ts">
	import { onMount } from 'svelte'
	import { glyphRun, configureGlyphs } from '@qntx/glyphs'
	import type { Glyph } from '@qntx/glyphs'

	const isMobile = window.matchMedia('(max-width: 768px)').matches

	interface ExtraLink { label: string; url: string }

	function project(id: string, title: string, url: string, subtitle: string, description: string, showAttestation = false, extraLinks: ExtraLink[] = []): Glyph & { url: string } {
		return {
			id, title, url,
			manifestationType: isMobile ? 'panel' : 'window',
			initialWidth: '320px',
			initialHeight: showAttestation ? '260px' : extraLinks.length ? '170px' : '140px',
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
		project('glyphs', 'Glyphs', 'https://jsr.io/@qntx/glyphs', 'UI Primitive', 'An interface element with a permanent identity that smoothly changes form. This window is one.'),
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

		const sub = document.createElement('div')
		sub.style.cssText = 'font-size: 11px; color: var(--text-secondary); margin-bottom: 8px;'
		sub.textContent = subtitle
		el.appendChild(sub)

		const desc = document.createElement('div')
		desc.style.cssText = 'font-size: 11px; color: var(--text-tertiary); line-height: 1.6; margin-bottom: 12px;'
		desc.textContent = description
		el.appendChild(desc)

		if (showAttestation) {
			const ats = document.createElement('pre')
			ats.style.cssText = 'font-size: 11px; color: var(--text-tertiary); line-height: 1.6; margin-bottom: 12px; white-space: pre;'
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
		link.style.cssText = 'font-size: 11px; color: var(--accent); text-decoration: none;'

		const row = document.createElement('div')
		row.style.cssText = 'display: flex; flex-wrap: wrap; gap: 12px;'
		row.appendChild(link)

		for (const extra of extraLinks) {
			const a = document.createElement('a')
			a.href = extra.url
			a.target = '_blank'
			a.rel = 'noopener noreferrer'
			a.textContent = extra.label
			a.style.cssText = 'font-size: 11px; color: var(--accent); text-decoration: none;'
			row.appendChild(a)
		}

		el.appendChild(row)

		return el
	}

	onMount(() => {
		// Resting dot doubled from the 10px default. Expanded size is unchanged,
		// so the dot travels less distance to reach it.
		configureGlyphs({ dotGeometry: { minWidth: 20, minHeight: 20 } })
		glyphRun.init()

		for (const project of projects) {
			glyphRun.add(project)
		}
	})
</script>

<style>
	/* Tray — fixed right edge, vertically centered */
	:global(.glyph-run) {
		position: fixed;
		top: 50%;
		right: 4px;
		transform: translateY(-50%);
		z-index: 1000;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		pointer-events: none;
		touch-action: none;
		width: fit-content;
		margin-left: auto;
	}

	:global(.glyph-run[data-empty="true"]) {
		pointer-events: none;
	}

	:global(.glyph-run-indicators) {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 2px;
		width: fit-content;
		height: fit-content;
	}

	@media (max-width: 768px) {
		:global(.glyph-run) {
			top: 60%;
			z-index: 10004;
		}
		:global(.glyph-run-indicators) {
			gap: 6px;
		}
	}

	/* Dot base — pointer events on dots only */
	:global(.glyph-run-glyph) {
		pointer-events: auto;
		cursor: pointer;
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: 3px;
	}

	/* Morph transitions */
	:global(.glyph-morphing-to-window) {
		position: fixed !important;
		z-index: 1001 !important;
		pointer-events: auto;
	}

	:global(.window-morphing-to-glyph) {
		position: fixed !important;
		z-index: 1001 !important;
		pointer-events: auto;
	}

	/* Window title bar */
	:global(.glyph-title-bar) {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 0 8px;
		background-color: var(--glyph-fill);
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
	:global(.glyph-title-bar button) {
		width: 22px;
		height: 22px;
		padding: 1px 2px;
		font-size: 13px;
		background: var(--bg);
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

	:global(.glyph-title-bar button:hover) {
		background: var(--border);
		color: var(--text);
	}

	@media (pointer: coarse) {
		:global(.glyph-title-bar) {
			height: 44px;
		}
		:global(.glyph-title-bar button) {
			width: 44px;
			height: 44px;
			font-size: 17px;
		}
	}

	/* Hover/touch-hold feedback — brighter while pointing */
	:global(.glyph-run-glyph:hover),
	:global(.glyph-run-glyph:active) {
		filter: brightness(1.3);
	}

	/* Window surface — border for contrast */
	:global(.glyph-run-glyph[data-window-state="true"]) {
		border: 1px solid var(--border);
	}

	/* Window content */
	:global(.glyph-content-area) {
		flex: 1;
		overflow: auto;
		padding: 8px;
	}

	:global(.glyph-content) {
		padding: 4px;
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		line-height: 1.4;
	}

	/* Reduced motion */
	@media (prefers-reduced-motion: reduce) {
		:global(.glyph-run-indicators),
		:global(.glyph-morphing-to-window),
		:global(.window-morphing-to-glyph),
		:global(.glyph-run-glyph) {
			transition-duration: 0s !important;
			animation-duration: 0s !important;
		}
	}
</style>
