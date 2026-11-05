/* Shared by the static marketplace and the local preview. */
function serverOrigin(value) {
  try {
    const url = new URL(value);
    return /^(https?:)$/.test(url.protocol) && !url.username && !url.password
      ? url.origin
      : null;
  } catch {
    return null;
  }
}

function installUrl(host, packageId, packageUrl) {
  const origin = serverOrigin(host);
  if (!origin)
    throw new Error("Enter an http:// or https:// Sandstorm server URL.");
  return `${origin}/install/${encodeURIComponent(packageId)}?url=${encodeURIComponent(packageUrl)}`;
}

function rememberedServers() {
  try {
    const saved = JSON.parse(window.localStorage.getItem("sandstormServers") || "[]");
    const latest = window.localStorage.getItem("sandstormServer");
    return [...new Set([latest, ...(Array.isArray(saved) ? saved : [])]
      .map(serverOrigin).filter(Boolean))];
  } catch {
    return [];
  }
}

function rememberServer(value) {
  const origin = serverOrigin(value);
  if (origin) {
    try {
      const servers = [origin, ...rememberedServers().filter(server => server !== origin)];
      window.localStorage.setItem("sandstormServers", JSON.stringify(servers));
      window.localStorage.setItem("sandstormServer", origin);
    } catch {
      // Installation still works when browser storage is unavailable.
    }
  }
  return origin;
}

function preferredServer(value) {
  const origin = rememberServer(value);
  if (origin) return origin;
  return rememberedServers()[0] || null;
}

function matchesApp(searchText, categories, query, tag) {
  return (
    (!tag || categories.includes(tag)) &&
    searchText.toLowerCase().includes(query.toLowerCase())
  );
}

function categoryFromPath(pathname) {
  const match = /^\/genre\/([^/]+)\/?$/.exec(pathname);
  if (!match) return "";
  try {
    return decodeURIComponent(match[1]);
  } catch {
    return "";
  }
}

if (typeof module !== "undefined")
  module.exports = { serverOrigin, installUrl, matchesApp, categoryFromPath };

if (typeof document !== "undefined")
  (() => {
    const params = new URLSearchParams(location.search);
    const experimental = params.get("experimental") === "true";
    const variant = document.getElementById("experimental-page");
    if (experimental && variant) {
      const content = variant.content.cloneNode(true);
      // This script is already running; inserted script tags would bind handlers twice.
      content.querySelectorAll("script").forEach((script) => script.remove());
      document.body.replaceChildren(content);
    }
    const host = preferredServer(params.get("host"));
    const chooser = document.getElementById("server-chooser");
    if (chooser && host) {
      chooser.hidden = false;
      const label = `Install on: ${host}`;
      chooser.querySelector("summary").textContent = label;
      chooser.querySelector("p").textContent = label;
      const details = chooser.querySelector("details");
      const list = chooser.querySelector("ul");
      const updateChooser = () => {
        const hasAlternates = list.children.length > 0;
        details.hidden = !hasAlternates;
        chooser.querySelector("p").hidden = hasAlternates;
      };
      rememberedServers().filter(server => server !== host).forEach(server => {
        const item = document.createElement("li");
        const name = document.createElement("a");
        name.textContent = server;
        const url = new URL(location.href);
        url.searchParams.set("host", server);
        name.href = url.href;
        const remove = document.createElement("button");
        remove.type = "button";
        remove.textContent = "×";
        remove.setAttribute("aria-label", `Forget ${server}`);
        remove.addEventListener("click", () => {
          try {
            window.localStorage.setItem("sandstormServers", JSON.stringify(
              rememberedServers().filter(saved => saved !== server),
            ));
          } catch {
            window.alert("Unable to forget this server because browser storage is unavailable.");
            return;
          }
          item.remove();
          updateChooser();
          if (!details.hidden) chooser.querySelector("summary").focus();
        });
        item.append(name, remove);
        list.append(item);
      });
      updateChooser();
    }
    const query = params.get("q") || "";
    const tag = categoryFromPath(location.pathname);
    const authorPage = location.pathname.startsWith("/author/");
    const welcome = document.getElementById("welcome");
    if (welcome) welcome.hidden = Boolean(authorPage || tag || query);
    const title = document.getElementById("catalog-title");
    if (title)
      title.textContent = query
        ? `Search results for “${query}”`
        : title.hasAttribute("data-author-title")
          ? title.textContent
          : tag || "All Apps";
    const cards = document.querySelectorAll("article[data-search]");
    let visible = 0;
    cards.forEach((card) => {
      card.classList.toggle("featured", card.hasAttribute("data-featured") && !query && !tag && !authorPage);
      card.hidden = (card.hasAttribute("data-homepage-hidden") && !query && !tag && !authorPage) || !matchesApp(
        card.dataset.search,
        JSON.parse(card.dataset.categories),
        query,
        tag,
      );
      if (!card.hidden) visible++;
    });
    const empty = document.getElementById("empty-results");
    if (empty) empty.hidden = visible !== 0;
    const search = document.querySelector('input[name="q"]');
    if (search) search.value = query;
    document.querySelectorAll("[data-tag]").forEach((link) => {
      const selected = link.dataset.tag === (tag || "All");
      link.classList.toggle("selected", selected);
      const url = new URL(link.href);
      if (query) url.searchParams.set("q", query);
      link.href = url.href;
    });
    document.querySelectorAll("form.search-form").forEach((form) => {
      if (!experimental) return;
      const field = document.createElement("input");
      field.type = "hidden";
      field.name = "experimental";
      field.value = "true";
      form.append(field);
    });
    // Carry experimental mode through category links, app pages, and back links.
    document.querySelectorAll("a[href]").forEach((link) => {
      if (link.closest("#server-chooser")) return;
      const url = new URL(link.href);
      if (
        url.origin === location.origin &&
        (url.pathname === "/" ||
          url.pathname.startsWith("/app/") ||
          url.pathname.startsWith("/genre/") ||
          url.pathname.startsWith("/author/"))
      ) {
        url.searchParams.delete("host");
        if (experimental) url.searchParams.set("experimental", "true");
        link.href = url.href;
      }
    });
    document.querySelectorAll("[data-install]").forEach((link) => {
      const packageUrl = new URL(link.getAttribute("href"), location.href).href;
      if (host)
        link.href = installUrl(host, link.dataset.packageId, packageUrl);
      link.addEventListener("click", (event) => {
        if (host) return;
        event.preventDefault();
        const value = window.prompt(
          "Your Sandstorm server URL (for example https://sandstorm.example.com):",
        );
        if (value === null) return;
        const origin = serverOrigin(value);
        if (!origin) {
          window.alert("Enter an http:// or https:// Sandstorm server URL.");
          return;
        }
        rememberServer(origin);
        location.href = installUrl(origin, link.dataset.packageId, packageUrl);
      });
    });

    (() => {
      const lightbox = document.getElementById("lightbox");
      const lightboxImage = document.getElementById("lightbox-image");
      const closeButton = document.getElementById("lightbox-close");
      if (!lightbox || !lightboxImage || !closeButton) return;

      const open = (src, alt) => {
        lightboxImage.src = src;
        lightboxImage.alt = alt || "";
        lightbox.classList.add("open");
        lightbox.setAttribute("aria-hidden", "false");
        document.body.style.overflow = "hidden";
      };

      const close = () => {
        lightbox.classList.remove("open");
        lightbox.setAttribute("aria-hidden", "true");
        lightboxImage.removeAttribute("src");
        document.body.style.overflow = "";
      };

      document.querySelectorAll("[data-lightbox-src]").forEach((button) => {
        button.addEventListener("click", () => {
          open(button.dataset.lightboxSrc, button.dataset.lightboxAlt);
        });
      });

      closeButton.addEventListener("click", close);
      lightbox.addEventListener("click", (event) => {
        if (event.target === lightbox) close();
      });
      document.addEventListener("keydown", (event) => {
        if (event.key === "Escape" && lightbox.classList.contains("open")) {
          close();
        }
      });
    })();
  })();
