import { mount } from 'svelte'
import App from './App.svelte'

mount(App, { target: document.getElementById('app')! })

// Every link click, unconditionally — no per-link wiring to remember.
document.addEventListener('click', (e) => {
	const a = (e.target as Element).closest?.('a')
	if (!a) return
	;(window as any).stand?.('click', { target: a.getAttribute('aria-label') || a.href })
})
