// Loaded as a classic (non-module) script in <head> so window.stand
// exists before main.js runs.

// The id that once outlived the visit, taken off every browser still holding it.
try { localStorage.removeItem("stand_id"); } catch (refused) { /* nothing held */ }

window.stand = (event, params = {}) => {
  // Empty means absent: an id nobody has is a key the node never stores,
  // rather than a blank one every storage-refusing browser shares.
  const hold = (store, key) => {
    try {
      let id = window[store].getItem(key);
      if (!id) { id = crypto.randomUUID(); window[store].setItem(key, id); }
      return id;
    } catch (refused) { return ""; }
  };
  const u = new URL("https://api.q.sbvh.nl/s/SBVH/profiel.png");
  u.searchParams.set("e", event);
  u.searchParams.set("page", location.pathname);

  // The sitting, and nothing longer. One id per tab, gone when it closes:
  // nothing outlives the visit, so nothing here asks for consent. A visitor
  // who comes back next week is a new visit, which is what we chose.
  const sitting = hold("sessionStorage", "stand_visit");
  if (sitting) u.searchParams.set("visit", sitting);

  // Your own pages are not a referrer: moving inside the site is the
  // walk, not the way in.
  try {
    if (document.referrer && new URL(document.referrer).hostname !== location.hostname) {
      u.searchParams.set("ref", document.referrer);
    }
  } catch (notAUrl) { /* no referrer worth sending */ }

  const here = new URLSearchParams(location.search);
  for (const k of ["utm_source", "utm_medium", "utm_campaign", "utm_content", "utm_term"]) {
    const value = here.get(k);
    if (value) u.searchParams.set(k, value);
  }

  // What you pass is what is stored, under the key you passed it with.
  for (const k in params) u.searchParams.set(k, params[k]);

  // keepalive, so an event fired as the page goes still arrives.
  fetch(u, { method: "GET", mode: "no-cors", keepalive: true }).catch(() => {});
};
stand("i");
