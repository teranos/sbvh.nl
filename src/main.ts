import { mount } from 'svelte'
import App from './App.svelte'
import { browserLaye, browserStorage, type Deps } from './door'

const NODE = 'https://api.q.sbvh.nl'

const door: Deps = {
	node: NODE,
	fetch: (input, init) => window.fetch(input, init),
	laye: browserLaye(NODE),
	storage: browserStorage(),
	url: () => window.location.href,
	replaceUrl: (url) => window.history.replaceState(null, '', url),
	navigate: (url) => {
		window.location.href = url
	},
}

mount(App, { target: document.getElementById('app')!, props: { door } })

// Every link click, unconditionally — no per-link wiring to remember.
document.addEventListener('click', (e) => {
	const a = (e.target as Element).closest?.('a')
	if (!a) return
	;(window as any).stand?.('click', { target: a.getAttribute('aria-label') || a.href })
})
