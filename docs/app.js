const state = { mods: [] };

function escapeHtml(value = "") {
  return String(value).replace(/[&<>"']/g, ch => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;",
    '"': "&quot;", "'": "&#039;"
  }[ch]));
}

function statusOf(mod) {
  return mod.status || "stable";
}

function statusLabel(status) {
  if (status === "blocked") return "Temporarily blocked";
  if (status === "experimental") return "Experimental";
  return "Stable";
}

function installMod(id) {
  window.location.href = `obsidianmods://install/${encodeURIComponent(id)}`;
}
function modCard(mod) {
  const status = statusOf(mod);
  const blocked = status === "blocked";
  const tags = (mod.tags || []).slice(0, 4)
    .map(tag => `<span class="tag">${escapeHtml(tag)}</span>`).join("");
  return `
    <article class="card ${mod.featured ? "featured" : ""}">
      <div class="card-top">
        <a class="card-title" href="mods/${encodeURIComponent(mod.id)}/">
          <h3>${escapeHtml(mod.name)}</h3>
        </a>
        <span class="pill">${escapeHtml(mod.category.toUpperCase())}</span>
      </div>
      <div class="meta">v${escapeHtml(mod.version)} · ${escapeHtml(mod.author)} · ${escapeHtml(mod.loader)}</div>
      <div class="compat ${status}">${escapeHtml(statusLabel(status))}</div>
      <p>${escapeHtml(mod.summary)}</p>
      <div class="tags">${tags}</div>
      ${mod.status_note ? `<div class="status-note">${escapeHtml(mod.status_note)}</div>` : ""}
      <div class="card-actions">
        <a class="secondary small" href="mods/${encodeURIComponent(mod.id)}/">Details</a>
        <button class="install" data-id="${escapeHtml(mod.id)}" ${blocked ? "disabled" : ""}>
          ${blocked ? "Unavailable" : "Install with Obsidian"}
        </button>
      </div>
    </article>`;
}
function bindInstallButtons(root = document) {
  root.querySelectorAll(".install:not([disabled])").forEach(button => {
    button.addEventListener("click", () => installMod(button.dataset.id));
  });
}

function setupDirectory() {
  const grid = document.querySelector("#mods");
  if (!grid) return;

  const empty = document.querySelector("#empty");
  const search = document.querySelector("#search");
  const category = document.querySelector("#category");
  const status = document.querySelector("#status-filter");
  const stats = document.querySelector("#catalog-stats");

  if (category) {
    const categories = [...new Set(state.mods.map(mod => mod.category))].sort();
    categories.forEach(value => {
      const option = document.createElement("option");
      option.value = value.toLowerCase();
      option.textContent = value;
      category.appendChild(option);
    });
  }
  const render = () => {
    const q = (search?.value || "").trim().toLowerCase();
    const wantedCategory = category?.value || "";
    const wantedStatus = status?.value || "";

    const mods = state.mods.filter(mod => {
      const haystack = [mod.name, mod.author, mod.summary, mod.category, ...(mod.tags || [])]
        .join(" ").toLowerCase();
      return (!q || haystack.includes(q))
        && (!wantedCategory || mod.category.toLowerCase() === wantedCategory)
        && (!wantedStatus || statusOf(mod) === wantedStatus);
    });

    grid.innerHTML = mods.map(modCard).join("");
    if (empty) empty.hidden = mods.length !== 0;
    if (stats) stats.textContent = `${mods.length} of ${state.mods.length} mods`;
    bindInstallButtons(grid);
  };

  search?.addEventListener("input", render);
  category?.addEventListener("change", render);
  status?.addEventListener("change", render);
  render();
}
function renderModDetail() {
  const host = document.querySelector("#mod-detail");
  if (!host) return;

  const id = new URLSearchParams(location.search).get("id");
  const mod = state.mods.find(item => item.id === id);
  if (!mod) {
    host.innerHTML = '<div class="empty">Mod not found. <a href="discover.html">Back to Discover</a></div>';
    return;
  }

  document.title = `${mod.name} · Obsidian Mods`;
  const currentStatus = statusOf(mod);
  const blocked = currentStatus === "blocked";
  const dependencies = (mod.dependencies || []).length
    ? mod.dependencies.map(item => `<li>${escapeHtml(item)}</li>`).join("")
    : "<li>None</li>";
  const shots = (mod.screenshots || []).map(path =>
    `<img src="${escapeHtml(path)}" alt="${escapeHtml(mod.name)} screenshot" loading="lazy">`
  ).join("");
  host.innerHTML = `
    <a class="back-link" href="discover.html">← Back to Discover</a>
    <div class="mod-hero">
      <div>
        <div class="eyebrow">${escapeHtml(mod.category.toUpperCase())}</div>
        <h1>${escapeHtml(mod.name)}</h1>
        <p class="lead">${escapeHtml(mod.description)}</p>
        <div class="mod-meta-row">
          <span>v${escapeHtml(mod.version)}</span>
          <span>by ${escapeHtml(mod.author)}</span>
          <span>${escapeHtml(mod.loader)}</span>
          <span>Game build ${escapeHtml(mod.game_build)}</span>
        </div>
      </div>
      <div class="install-panel">
        <div class="compat ${currentStatus}">${escapeHtml(statusLabel(currentStatus))}</div>
        ${mod.status_note ? `<p class="status-note">${escapeHtml(mod.status_note)}</p>` : ""}
        <button class="install large-install" data-id="${escapeHtml(mod.id)}" ${blocked ? "disabled" : ""}>
          ${blocked ? "Install temporarily disabled" : "Install with Obsidian"}
        </button>
        <small>SHA-256 verified by Obsidian Mods Manager.</small>
      </div>
    </div>
    <div class="mod-columns">
      <section>
        <h2>About</h2>
        <p>${escapeHtml(mod.description)}</p>
        <div class="tags">${(mod.tags || []).map(tag => `<span class="tag">${escapeHtml(tag)}</span>`).join("")}</div>
      </section>
      <aside>
        <h3>Dependencies</h3>
        <ul>${dependencies}</ul>
        <h3>Package</h3>
        <code class="hash">${escapeHtml(mod.sha256)}</code>
      </aside>
    </div>
    ${shots ? `<section class="gallery"><h2>Screenshots</h2><div class="gallery-grid">${shots}</div></section>` : ""}
  `;
  bindInstallButtons(host);
}

fetch("catalog.json", { cache: "no-store" })
  .then(response => {
    if (!response.ok) throw new Error(`catalog HTTP ${response.status}`);
    return response.json();
  })
  .then(catalog => {
    state.mods = catalog.mods || [];
    setupDirectory();
    renderModDetail();
  })
  .catch(error => {
    console.error(error);
    const grid = document.querySelector("#mods");
    const empty = document.querySelector("#empty");
    if (grid) grid.innerHTML = "";
    if (empty) {
      empty.textContent = "Catalog temporarily unavailable.";
      empty.hidden = false;
    }
    const detail = document.querySelector("#mod-detail");
    if (detail) detail.innerHTML = '<div class="empty">Catalog temporarily unavailable.</div>';
  });
