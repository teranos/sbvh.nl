<script lang="ts">
	let inverted = $state(false)

	function toggleTheme() {
		inverted = !inverted
		document.documentElement.classList.toggle('inverted', inverted)
	}
</script>

<main>
	<div class="content">
		<button class="logo" onclick={toggleTheme} aria-label="Toggle color mode">
			<img src="/logo.svg" alt="SBvH" width="160" height="160" />
		</button>

		<h1>Sebastiaan Brandon van Houten</h1>

		<p class="intro">Systems engineer. Background in SRE and ML infrastructure. Now working on local inference, AI interpretability, visual programming languages, and genomic emancipation.</p>

		<div class="primitives">
			<span class="primitive attestation-hover">
				<strong>Attestation</strong> — a structured claim
				<pre class="ats-tooltip"><span class="ats">+ as</span>  [subject]
<span class="ats">= is</span>  [predicate]
<span class="ats">∈ of</span>  [context]
<span class="ats">⊢ by</span>  [actor]
<span class="ats">✦ at</span>  [time]
      {'{}' }</pre>
			</span>
			<p class="primitive"><strong>Glyph</strong> — a single DOM element that morphs between visual states for its entire lifetime</p>
		</div>

		<div class="projects">
			<a href="https://github.com/teranos/QNTX" target="_blank" rel="noopener noreferrer">QNTX</a>
			<a href="https://github.com/teranos/graunde" target="_blank" rel="noopener noreferrer">Graunde</a>
			<a href="https://github.com/teranos/QNTX" target="_blank" rel="noopener noreferrer">Loom</a>
			<a href="https://github.com/teranos/QNTX" target="_blank" rel="noopener noreferrer">Live Inference 3D Shader Viz</a>
			<a href="https://github.com/teranos/QNTX" target="_blank" rel="noopener noreferrer">Glyphs</a>
		</div>

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

	.primitives {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-width: 520px;
		text-align: center;
	}

	.primitive {
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		line-height: 1.6;
	}

	.primitive strong {
		color: var(--text);
	}

	.attestation-hover {
		position: relative;
		cursor: default;
	}

	.ats-tooltip {
		display: none;
		position: absolute;
		top: 100%;
		left: 50%;
		transform: translateX(-50%);
		margin-top: 0.5rem;
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		border-radius: var(--border-radius);
		padding: 0.75rem 1rem;
		line-height: 1.6;
		white-space: pre;
		z-index: 10;
	}

	.attestation-hover:hover .ats-tooltip {
		display: block;
	}

	.ats {
		color: var(--accent);
	}

	.label {
		font-family: Georgia, 'Times New Roman', serif;
		font-size: var(--font-size-md);
		letter-spacing: 0.05em;
		color: var(--text-secondary);
	}

	.projects {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 0.5rem 1.5rem;
		max-width: 380px;
	}

	.projects a {
		font-size: var(--font-size-lg);
		color: var(--accent);
		text-decoration: none;
		transition: color 0.2s ease;
	}

	.projects a:hover {
		color: var(--accent-hover);
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
		.social { gap: 1.5rem; }
	}
</style>
