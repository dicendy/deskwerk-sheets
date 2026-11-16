# Hosting Deskwerk Sheets for the web


`deskwerk-sheets-web-<version>.zip` (from the GitHub release, or `packaging/web/package.sh`) holds a
static site in `deskwerk-sheets-web-<version>/`:

| File | What it is |
|---|---|
| `index.html` | The page. It loads everything through relative URLs. |
| `deskwerk-sheets-web-<hash>.js` | wasm-bindgen glue (generated, ES module) |
| `deskwerk-sheets-web-<hash>_bg.wasm` | The app (compresses to roughly a third with gzip/Brotli) |
| `deskwerk-sheets-<hash>.ico`, `manifest.webmanifest`, `icon-192.png`, `icon-512.png` | Favicon, web app manifest and its icons |
| `_headers`, `.htaccess` | Sample header rules for Netlify/Cloudflare Pages and Apache |
| `fonts/` | The font pack: open fonts that draw the Office fonts a workbook names (Carlito for Calibri, Liberation for Arial/Times New Roman/Courier New, …) and popular open fonts, with `MANIFEST.tsv` (source, sha256, licence) and the licence texts in `fonts/licenses/`. About 46 MB in all (23 MB gzipped), but only Carlito (2.7 MB, 1.2 MB gzipped) loads at startup; every other file is fetched when a workbook or the user picks its font. |

There is no server-side code. Upload the folder's contents anywhere that serves static files.

## Any path works

All URLs in `index.html` are relative (`public_url = "./"` in `apps/deskwerk-sheets-web/Trunk.toml`),
so the site works at a domain root (`https://example.com/`), under a prefix
(`https://example.com/tools/deskwerk-sheets/`) and from a CDN bucket. The asset names carry a content
hash, so they can be cached forever. Only `index.html` needs revalidation.

## Required server settings

- **MIME type:** serve `.wasm` as `application/wasm`. Browsers refuse to stream-compile it under
  any other type, and the app then loads slowly or not at all. Serve `.js` as `text/javascript`.
  Most hosts already do both. For nginx, check that `mime.types` has `application/wasm wasm;`.
- **Compression:** turn on gzip or Brotli for `.wasm`, `.js` and `.html`. That cuts the
  download to roughly a third. You can also precompress (`brotli -k *.wasm`) and let
  the server send `Content-Encoding: br`.
- **Caching:** `Cache-Control: public, max-age=31536000, immutable` on the hashed `.wasm` and
  `.js` files, and `no-cache` on `index.html`. The font files keep their names; the app asks for
  them as `fonts/<file>?v=<sha256 prefix>`, so a long `max-age` on `fonts/` is safe too. Serve
  `.ttf` as `font/ttf` and compress it (it halves).
- **Fonts stay on your server:** the app fetches fonts only from `fonts/` next to `index.html`
  (never from Google Fonts or another CDN). Where the browser offers Local Font Access (Chrome,
  Edge), the font list also offers the user's own installed fonts, which then win over the pack.
- **HTTPS:** WebGPU (and the clipboard) only work in a secure context, which means `https://`
  or `http://localhost`. Over plain HTTP elsewhere, the app falls back to WebGL2.
- **No special isolation headers:** Deskwerk Sheets doesn't use `SharedArrayBuffer`, so it doesn't
  need `Cross-Origin-Opener-Policy` or `Cross-Origin-Embedder-Policy`. If your site already sends
  COEP `require-corp`, also send `Cross-Origin-Resource-Policy: same-origin` (or `cross-origin`
  when the files live on a CDN) on the app's files.

nginx example:

```nginx
location /deskwerk-sheets/ {
    types { application/wasm wasm; text/javascript js; text/html html; }
    gzip on;
    gzip_types application/wasm text/javascript text/html;
    types { font/ttf ttf; }
    location ~* \.(wasm|js)$ { add_header Cache-Control "public, max-age=31536000, immutable"; }
    location ~* \.ttf$ { add_header Cache-Control "public, max-age=2592000"; }
    location ~* index\.html$ { add_header Cache-Control "no-cache"; }
}
```

Local test: `python3 -m http.server 8765` inside the folder, then open http://localhost:8765/.

## Embedding in a page (iframe)

```html
<iframe
  src="https://example.com/deskwerk-sheets/"
  title="Deskwerk Sheets spreadsheet"
  style="width: 100%; height: 720px; border: 0;"
  allow="fullscreen; clipboard-read; clipboard-write"
  allowfullscreen>
</iframe>
```

- The app fills the iframe and follows its size, so size the iframe and not the app.
- Keyboard shortcuts go to the iframe after the user clicks into it, as with any embedded app.
- **Cross-origin embeds** work. Preferences are kept in the iframe's `localStorage`. Browsers
  that partition or block third-party storage may forget them between visits, and the app
  then starts with defaults.
- **Sandboxed iframes** need at least
  `sandbox="allow-scripts allow-same-origin allow-downloads allow-popups"`. Without
  `allow-same-origin` there's no storage. Without `allow-downloads`, Save and Export (browser
  downloads) are blocked.
- Don't send `X-Frame-Options: DENY` or a `frame-ancestors` CSP that excludes the embedding page.
- **Window chrome:** `?chrome=none` on the iframe `src` leaves out the title bar and the File tab
  (whose Backstage view then never opens); `?chrome=notitlebar` or `?chrome=nofiletab` leave out just
  one. Use them when the host window already shows the file name and owns the files, so the sheet's
  own Open or Save As cannot replace what the host thinks is open. Everything stays on by default.

### The parent bridge (`?host=parent`)

A page framing Deskwerk Sheets in a **same-origin** iframe can add `?host=parent` to the iframe's URL
(`https://example.com/deskwerk-sheets/?host=parent`) and have Deskwerk Sheets hand its files over instead of
downloading them. The two sides talk `postMessage` with plain window messages that carry a
`type`:

| Message | Direction | Fields |
|---|---|---|
| `deskwerk-sheets:ready` | app → page | `version`, `appVersion`: the app is listening; send a file now |
| `deskwerk-sheets:open` | page → app | `name`, `bytes` (an `ArrayBuffer` or `Uint8Array`): open like File › Open |
| `deskwerk-sheets:save` | app → page | `name`, `bytes` (a transferred `ArrayBuffer`): every file the app would otherwise download (Save, Save As, Export as PDF). The page stores it and reports failures itself. |
| `deskwerk-sheets:dirty` | app → page | `dirty`: unsaved changes appeared or went away |
| `deskwerk-sheets:failed` | app → page | `error`: the app couldn't start |

In host mode the app starts with an empty workbook instead of a sample, so the framing page
should guard leaving the page while changes are unsaved. `&author=` names the user for the
notes and comments the app creates. Opening `?host=parent` outside an iframe gives the normal
app with downloads.

### Embedding without an iframe (page host mode)

A page that loads the Deskwerk Sheets module into its own document — its own `<canvas
id="deskwerk_sheets_canvas">` inside a component, sized to a box beside the page's own header and
controls — opts in with `data-host="page"` on that canvas. The behaviour is the same as the
parent bridge, except the transport is DOM `CustomEvent`s on the canvas instead of
`postMessage`: the app dispatches `deskwerk-sheets:ready`, `deskwerk-sheets:save`, `deskwerk-sheets:dirty` and
`deskwerk-sheets:failed` on the canvas with the message fields above as the event's `detail`, and the
page dispatches `deskwerk-sheets:open` on the canvas with `detail: { name, bytes }` to open a
workbook. Events don't bubble, so two editors on one page can't hear each other; listen on the
canvas itself.

```html
<canvas id="deskwerk_sheets_canvas" data-host="page" style="width: 100%; height: 600px; border: 0;"></canvas>
```

```js
import init from "./deskwerk-sheets-web-<hash>.js"; // the wasm-bindgen glue shipped with the app

// Listen first: the app announces itself while `init` runs.
const canvas = document.getElementById("deskwerk_sheets_canvas");

canvas.addEventListener("deskwerk-sheets:ready", async () => {
  // Open a workbook of the page's own, as with File › Open.
  const bytes = await workbookBytes(); // an ArrayBuffer or Uint8Array
  canvas.dispatchEvent(new CustomEvent("deskwerk-sheets:open", {
    detail: { name: "Budget.xlsx", bytes },
  }));
});

// Every file Deskwerk Sheets would otherwise download (Save, Save As, Export as PDF) arrives here.
canvas.addEventListener("deskwerk-sheets:save", (event) => {
  const { name, bytes } = event.detail; // bytes is an ArrayBuffer
  storeFile(name, bytes);
});

canvas.addEventListener("deskwerk-sheets:dirty", (event) => {
  warnBeforeLeaving(event.detail.dirty);
});
canvas.addEventListener("deskwerk-sheets:failed", (event) => {
  showError(event.detail.error);
});

await init({ module_or_path: "./deskwerk-sheets-web-<hash>_bg.wasm" });
```

As with the parent bridge, the app starts with an empty workbook instead of a sample, and
`?author=` on the page's URL names the user for new notes and comments. The `data-host` value
must be exactly `page`; anything else is ignored, and the app runs as a standalone page with
downloads.

The app fetches its fonts from `fonts/` beside the script it was loaded from (a
`<link>`/`<script>` whose URL contains `deskwerk-sheets-web`), else beside the page. When the page
imports the module from elsewhere, say where the folder is with `data-fonts-base` on the canvas:
`<canvas id="deskwerk_sheets_canvas" data-host="page" data-fonts-base="/static/deskwerk-sheets/">` loads
`/static/deskwerk-sheets/fonts/…`.

The messages and both transports are the shared contract of [`craft-web-host`](https://github.com/storytold/craft-libs/tree/main/crates/web-host) (storytold/craft-libs#420), the same for every Craft app: its README lists every message, including `saved`, with the compatibility notes. In `deskwerk-sheets:ready`, `version` is the contract's (`"1"`) and `appVersion` is this app's release. Deskwerk Sheets reads bytes, so a host sends `bytes`, not a `File`.

### Trust and limits

- **Who is trusted.** In page host mode the page and Deskwerk Sheets share one document, so any script on the page can drive the app: open files in it and receive what it saves. Use this mode only for a page you control. The iframe mode (`?host=parent`) answers only the parent window, on the page's own origin.
- **Files from the host are untrusted input.** An opened file goes through the same reading as one picked with File ▸ Open or dropped on the window, with the same format limits; before it is copied into the app's memory, its size is checked against the host open limit (512 MiB) and a larger file is refused, and a name is cut to 255 characters.
- **The event and message names are the interface.** `deskwerk-sheets:ready`, `deskwerk-sheets:open`, `deskwerk-sheets:save`, `deskwerk-sheets:dirty` and `deskwerk-sheets:failed`, and the fields they carry (the same as the `?host=parent` messages), stay stable; changing one is a breaking change.

### Driving it from the page (`window.deskwerk_sheets`)

Every Deskwerk Sheets web build puts `window.deskwerk_sheets` on the page, from [`craft-web-api`](https://github.com/storytold/craft-libs/tree/main/crates/web-api) (storytold/craft-libs#413), the same shape in every Craft app. `deskwerk_sheets.request(method, params)` calls any method of the control channel the desktop app serves (`docs/control-protocol.md`), and `execute(command, params)`, `commands()`, `docInspect(params)`, `uiInspect(params)` and `screenshot(params)` are shortcuts for the common ones. Each returns a Promise that resolves with the method's result and rejects with an `Error`; a call the app does not answer within 30 seconds rejects, and the app skips it if it has not started it; a call the app has already taken still finishes, so a retry after a timeout can apply a command twice.

```js
const book = await deskwerk_sheets.docInspect();               // sheets, used ranges, tables, charts, names, selection
await deskwerk_sheets.execute("cell.set", { cell: "A1", input: "Total" });
const { data } = await deskwerk_sheets.screenshot();          // the canvas as PNG (base64)
```

The API is there in every mode, with or without a host. Opening a workbook, handing a saved or exported file to the page, the file name and the unsaved flag stay with the host messages above.

## Renderer selection

Deskwerk Sheets' web build is the same egui app as the desktop one, compiled to wasm
(`apps/deskwerk-sheets-web`). It renders with the backend `apps/deskwerk-sheets-web` selects (WebGPU
where available, otherwise WebGL2). If the web app adds URL flags to force a backend (as the
sibling Craft apps do with `?webgl` / `?cpu`), document them here; they also work on the
iframe `src`.

A browser with neither WebGPU nor WebGL2 gets a message in place of the app.
