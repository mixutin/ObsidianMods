const state = { mods: [] };
const grid = document.querySelector("#mods");
const empty = document.querySelector("#empty");
const search = document.querySelector("#search");

function escapeHtml(value = "") {
  return value.replace(/[&<>"']/g, ch => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#039;"
  }[ch]));
}

function render() {
  const q = search.value.trim().toLowerCase();
  const mods = state.mods.filter(mod =>
    [mod.name, mod.summary, mod.category, ...(mod.tags || [])]
      .join(" ").toLowerCase().includes(q)
  );

  grid.innerHTML = mods.map(mod => `
    <article class="card ${mod.featured ? "featured" : ""}">
      <div class="card-top">
        <h3>${escapeHtml(mod.name)}</h3>
        <span class="pill">${escapeHtml(mod.category.toUpperCase())}</span>
      </div>
      <div class="meta">v${escapeHtml(mod.version)} · ${escapeHtml(mod.author)} · ${escapeHtml(mod.loader)}</div>
      <p>${escapeHtml(mod.summary)}</p>
      <button class="install" data-id="${escapeHtml(mod.id)}">Install with Obsidian</button>
    </article>
  `).join("");

  empty.hidden = mods.length !== 0;
  document.querySelectorAll(".install").forEach(button => {
    button.addEventListener("click", () => {
      window.location.href = `obsidianmods://install/${encodeURIComponent(button.dataset.id)}`;
    });
  });
}

search.addEventListener("input", render);

fetch("catalog.json", { cache: "no-store" })
  .then(response => {
    if (!response.ok) throw new Error(`catalog HTTP ${response.status}`);
    return response.json();
  })
  .then(catalog => {
    state.mods = catalog.mods || [];
    render();
  })
  .catch(error => {
    console.error(error);
    grid.innerHTML = "";
    empty.textContent = "Catalog temporarily unavailable.";
    empty.hidden = false;
  });
