#!/usr/bin/env python3
import html
import json
from pathlib import Path
from urllib.parse import quote

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
BASE = "https://mixutin.github.io/ObsidianMods"

def esc(value):
    return html.escape(str(value), quote=True)

def mod_url(mod_id):
    return f"{BASE}/mods/{quote(mod_id)}/"

def render_mod(mod):
    url = mod_url(mod["id"])
    title = f'{mod["name"]} - Minecraft Dungeons II Mod | Obsidian Mods'
    description = (
        f'{mod["name"]} for Minecraft Dungeons II (Dungeons 2). '
        f'{mod["summary"]} Install with Obsidian Mods Manager.'
    )
    tags = "".join(
        f'<span class="tag">{esc(tag)}</span>'
        for tag in mod.get("tags", [])
    )
    dependencies = mod.get("dependencies") or []
    deps = "".join(f"<li>{esc(dep)}</li>" for dep in dependencies) or "<li>None</li>"
    status = mod.get("status", "stable")
    status_note = mod.get("status_note", "")
    disabled = " disabled" if status == "blocked" else ""
    button = "Install temporarily disabled" if status == "blocked" else "Install with Obsidian"

    structured = {
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": mod["name"],
        "applicationCategory": "GameApplication",
        "operatingSystem": "Windows, Linux",
        "softwareVersion": mod["version"],
        "description": description,
        "url": url,
        "author": {"@type": "Organization", "name": mod["author"]},
        "isPartOf": {"@type": "WebSite", "name": "Obsidian Mods", "url": BASE + "/"},
    }
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <meta name="description" content="{esc(description)}">
  <meta name="robots" content="index,follow,max-image-preview:large">
  <link rel="canonical" href="{esc(url)}">
  <meta property="og:type" content="website">
  <meta property="og:site_name" content="Obsidian Mods">
  <meta property="og:title" content="{esc(title)}">
  <meta property="og:description" content="{esc(description)}">
  <meta property="og:url" content="{esc(url)}">
  <meta name="twitter:card" content="summary_large_image">
  <title>{esc(title)}</title>
  <link rel="stylesheet" href="../../styles.css">
  <script type="application/ld+json">{json.dumps(structured, separators=(",", ":"))}</script>
</head>
<body>
  <header class="nav">
    <a class="brand" href="../../"><span class="gem">◆</span> Obsidian <b>Mods</b></a>
    <nav>
      <a href="../../discover.html">Discover</a>
      <a href="../../manager.html">Manager</a>
      <a href="../../overlay.html">In-Game Menu</a>
      <a href="https://github.com/mixutin/ObsidianMods">GitHub</a>
    </nav>
  </header>
  <main>
    <section class="section mod-detail">
      <a class="back-link" href="../../discover.html">← Back to Minecraft Dungeons II mods</a>
      <div class="mod-hero">
        <div>
          <div class="eyebrow">{esc(mod["category"]).upper()} · MINECRAFT DUNGEONS II MOD</div>
          <h1>{esc(mod["name"])}</h1>
          <p class="lead">{esc(mod["description"])}</p>
          <div class="mod-meta-row">
            <span>v{esc(mod["version"])}</span>
            <span>by {esc(mod["author"])}</span>
            <span>{esc(mod["loader"])}</span>
            <span>Game build {esc(mod["game_build"])}</span>
          </div>
        </div>
        <div class="install-panel">
          <div class="compat {esc(status)}">{esc(status.title())}</div>
          {f'<p class="status-note">{esc(status_note)}</p>' if status_note else ''}
          <button class="install large-install" onclick="location.href='obsidianmods://install/{esc(mod["id"])}'"{disabled}>{button}</button>
          <small>SHA-256 verified by Obsidian Mods Manager.</small>
        </div>
      </div>
      <div class="mod-columns">
        <section>
          <h2>About this Minecraft Dungeons II mod</h2>
          <p>{esc(mod["description"])}</p>
          <div class="tags">{tags}</div>
          <h2>How to install</h2>
          <p>Install Obsidian Mods Manager, then use the button above. The manager downloads the official GitHub release, verifies its SHA-256 hash and installs it into the correct Dungeons II mod location.</p>
        </section>
        <aside>
          <h3>Dependencies</h3>
          <ul>{deps}</ul>
          <h3>Package SHA-256</h3>
          <code class="hash">{esc(mod["sha256"])}</code>
        </aside>
      </div>
      <section class="star-inline">
        <h2>Want more Minecraft Dungeons II mods?</h2>
        <p>⭐ Star Obsidian Mods on GitHub. It genuinely motivates the developer to keep building more mods, compatibility fixes and manager features.</p>
        <a class="primary" href="https://github.com/mixutin/ObsidianMods">Star Obsidian Mods</a>
      </section>
    </section>
  </main>
  <footer>Obsidian Mods · Community project · Not affiliated with Mojang or Microsoft.</footer>
</body>
</html>
"""
def main():
    catalog = json.loads((DOCS / "catalog.json").read_text())
    mods = catalog.get("mods", [])

    for mod in mods:
        out = DOCS / "mods" / mod["id"] / "index.html"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(render_mod(mod))

    urls = [
        BASE + "/",
        BASE + "/discover.html",
        BASE + "/manager.html",
        BASE + "/overlay.html",
        BASE + "/package-format.html",
    ]
    urls.extend(mod_url(mod["id"]) for mod in mods)
    sitemap = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
    ]
    sitemap.extend(f"  <url><loc>{esc(url)}</loc></url>" for url in urls)
    sitemap.append("</urlset>")
    (DOCS / "sitemap.xml").write_text("\n".join(sitemap) + "\n")
    print(f"generated {len(mods)} mod page(s) and sitemap")

if __name__ == "__main__":
    main()
