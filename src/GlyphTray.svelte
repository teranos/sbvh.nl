<script lang="ts">
	import { onMount } from 'svelte'
	import { GlyphProximity } from '@qntx/glyphs/proximity'
	import type { Glyph } from '@qntx/glyphs/glyph'

	const projects: (Glyph & { url: string })[] = [
		{ id: 'qntx', title: 'QNTX', url: 'https://github.com/teranos/QNTX', renderContent: () => document.createElement('div') },
		{ id: 'graunde', title: 'Graunde', url: 'https://github.com/teranos/graunde', renderContent: () => document.createElement('div') },
		{ id: 'loom', title: 'Loom', url: 'https://github.com/teranos/QNTX', renderContent: () => document.createElement('div') },
		{ id: 'shader', title: 'Live Inference 3D Shader Viz', url: 'https://github.com/teranos/QNTX', renderContent: () => document.createElement('div') },
		{ id: 'glyphs', title: 'Glyphs', url: 'https://github.com/teranos/QNTX', renderContent: () => document.createElement('div') },
	]

	let container: HTMLElement
	let proximity: GlyphProximity
	let items: Map<string, Glyph> = new Map()
	let raf: number | null = null

	function loop() {
		if (!container) return
		proximity.updateProximity(container, items, false)
		raf = requestAnimationFrame(loop)
	}

	onMount(() => {
		proximity = new GlyphProximity()

		// Create glyph elements — once and only once (DOM axiom)
		for (const project of projects) {
			items.set(project.id, project)

			const el = document.createElement('div')
			el.className = 'glyph-run-glyph'
			el.setAttribute('data-glyph-id', project.id)
			el.addEventListener('click', () => {
				window.open(project.url, '_blank', 'noopener,noreferrer')
			})
			container.appendChild(el)
		}

		raf = requestAnimationFrame(loop)

		return () => {
			if (raf) cancelAnimationFrame(raf)
		}
	})
</script>

<div bind:this={container} class="glyph-tray"></div>

<style>
	.glyph-tray {
		position: fixed;
		right: 0;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 4px;
		padding: 8px 6px;
		z-index: 100;
	}

	.glyph-tray :global(.glyph-run-glyph) {
		background-color: var(--text-tertiary) !important;
		cursor: pointer;
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		color: var(--bg);
		overflow: hidden;
	}
</style>
