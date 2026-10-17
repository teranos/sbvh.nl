# sbvhAPI

A QNTX plugin. It is what sbvh.nl cannot be by living on GitHub Pages: somewhere
a person who signed in can book a call, and where that booking is attested in
the SBVH namespace rather than kept in somebody else's calendar.

Nobody books anonymously. The site sends a prospect through the SBVH door on
the node first (QNTX ADR-032), so every call has a User behind it, with the
name and address their provider gave.

## What it serves

    GET /api/sbvhAPI/openapi.json

Generated from the routes and their own doc comments, and the only answer to
what this build serves. In short: `slots`, `book`, `mine` and `cancel` for a
signed-in person, `calls` for ROOT.

## What it keeps

Nothing but attestations, in the namespace store the node hands it:

- `meet-book`: a call, its start, length, what the person wrote, and who they are.
- `meet-cancel`: a call that will not happen, and the call it moved to, if it moved.

What is open is worked out from those each time. Bookings are serialised
inside the plugin and the slot is checked again under that lock, so two people
taking the same time at once get one call and one refusal. One standing call
per person: booking again names the call it replaces.

Every booking and cancellation is mailed to the person and to the owner
through the node's MailService.

## Config

| Key | What it is |
| --- | --- |
| `namespace` | `SBVH`. The store the node mints a token for. |
| `owner` | The User told of every call booked and cancelled. |
| `place` | Where the call happens, as the mails say it. |
| `hours` | Open hours in Amsterdam time, e.g. `mon-fri 10:00-12:00,14:00-17:00`. |
| `slot_minutes`, `notice_hours`, `horizon_days` | Length of a call, how far ahead the first slot is, how many days are offered. |
| `sentry_dsn`, `sentry_environment` | Where its log and errors go. |

A value it cannot read is not quietly replaced by a default: `slots` and `book`
answer with the key and what is wrong with it until the record is fixed.

## Building

`nix develop` pins the toolchain; cargo does the build. The build needs
`protoc` on the path for the QNTX protocol crates.

    make test
    make install

`make install` bumps the patch version, builds, installs to `~/.qntx/plugins`,
then tells the running QNTX to restart and polls until it reports the version
just built.

## The name

`sbvhAPI` is the last segment of the plugin's repository URL
(`https://github.com/teranos/sbvh.nl/tree/master/qntx/sbvhAPI`), and the loader
refuses to load on mismatch. It appears in `metadata()`, in the binary name and
in the route prefix, and all of them have to agree.
