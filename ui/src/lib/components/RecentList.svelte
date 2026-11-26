<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { RecentProject } from "../api";
  import { t } from "../i18n.svelte";
  import Icon from "./Icon.svelte";
  import Menu from "./Menu.svelte";
  import { popupMenu } from "../cm-lsp";
  // Zuletzt geöffnete Projekte: je Projekt eine kompakte Zeile (System, Name, gekürzter Pfad, Alter, Stern, Menü „…“); angeheftete stehen oben
  let { items, onopen, onpin, oncopy, onreveal, onremove, max = 5 }: { items: RecentProject[]; onopen: (p: RecentProject) => void; onpin: (p: RecentProject) => void; oncopy: (p: RecentProject) => void; onreveal: (p: RecentProject) => void; onremove: (p: RecentProject) => void; max?: number } = $props();
  let all = $state(false);
  const shown = $derived(all ? items : items.slice(0, max));
  const pinned = $derived(shown.filter((x) => x.pinned));
  const others = $derived(shown.filter((x) => !x.pinned));
  /** Pfad in der Mitte kürzen (Anfang und Ende bleiben lesbar) */
  const mid = (s: string, n = 56): string => (s.length <= n ? s : `${s.slice(0, Math.ceil((n - 1) / 2))}…${s.slice(s.length - Math.floor((n - 1) / 2))}`);
  const sys = (r: RecentProject): string => (r.systems.length ? r.systems[0]![0] + (r.systems.length > 1 ? ` +${r.systems.length - 1}` : "") : "–");
  const tip = (r: RecentProject): string => [r.path, r.systems.map((s) => (s[1] ? `${s[0]} ${s[1].toUpperCase()}` : s[0])).join(", "), r.langs.join(", "), r.emulator].filter(Boolean).join("\n");
  const entries = (r: RecentProject) => [{ id: "open", label: t("startui-oeffnen"), disabled: r.missing }, { id: "pin", label: t(r.pinned ? "start-loesen" : "start-anheften"), disabled: r.missing }, { id: "copy", label: t("startui-kopieren"), disabled: r.missing }, { id: "reveal", label: t("tree-im-ordner-zeigen"), disabled: r.missing }, { id: "remove", label: t("start-entfernen") }];
  function menu(r: RecentProject, id: string) { if (id === "open") onopen(r); else if (id === "pin") onpin(r); else if (id === "copy") oncopy(r); else if (id === "reveal") onreveal(r); else onremove(r); }
  /** Rechtsklick (und Ctrl+Klick) auf einen Eintrag: dasselbe Menü wie „…“, an der Mausposition über allem */
  function kontext(e: MouseEvent, r: RecentProject) {
    e.preventDefault(); popupMenu(null, e.clientX, e.clientY, t("start-aktionen", { name: r.name }), entries(r).map((en) => ({ label: en.label, disabled: en.disabled, run: () => menu(r, en.id) })));
  }
  let root: HTMLElement | undefined = $state();
  function key(e: KeyboardEvent) {
    const d = e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : 0; if (!d || !root) return;
    const mains = [...root.querySelectorAll<HTMLButtonElement>(".main:not(:disabled)")]; const i = mains.indexOf(document.activeElement as HTMLButtonElement);
    if (i < 0) return; e.preventDefault(); mains[Math.max(0, Math.min(mains.length - 1, i + d))]?.focus();
  }
</script>

{#snippet row(r: RecentProject)}
  <li class="proj" class:missing={r.missing} oncontextmenu={(e) => kontext(e, r)}>
    {#if r.image}<img class="pic" src={r.image} alt="" loading="lazy" />{:else}<span class="pic ph" aria-hidden="true" style={`--h:${[...r.name].reduce((a, c) => (a * 31 + c.charCodeAt(0)) % 360, 7)}`}>{r.name.replace(/^[^:]*:\s*/, "").slice(0, 2).toUpperCase()}</span>{/if}
    <span class="sys" aria-hidden="true">{sys(r)}</span>
    <button type="button" class="main" disabled={r.missing} title={tip(r)} onclick={() => onopen(r)}>
      <span class="name">{r.name}</span>
      <span class="path mono" title={r.path}>{mid(r.pretty)}</span>
    </button>
    {#if r.missing}<span class="gone">{t("startui-ordner-fehlt")}</span>{:else}<span class="ago">{r.ago}</span>{/if}
    {#if !r.missing}<button type="button" class="ibtn sm star" class:on={r.pinned} aria-pressed={r.pinned} aria-label={t(r.pinned ? "start-loesen" : "start-anheften")} title={t(r.pinned ? "start-loesen" : "start-anheften")} onclick={() => onpin(r)}>{r.pinned ? "★" : "☆"}</button>{:else}<span class="ph"></span>{/if}
    <span class="mn"><Menu label={t("start-aktionen", { name: r.name })} items={entries(r)} onselect={(id) => menu(r, id)} align="right">{#snippet button()}<Icon name="more" size={16} />{/snippet}</Menu></span>
  </li>
{/snippet}

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div bind:this={root} class="rl" onkeydown={key} role="presentation">
  {#if pinned.length}<div class="kopf">{t("start-angeheftet-kopf")}</div><ul class="plain">{#each pinned as r (r.path)}{@render row(r)}{/each}</ul>{/if}
  {#if others.length}{#if pinned.length}<div class="kopf">{t("start-zuletzt-kopf")}</div>{/if}<ul class="plain">{#each others as r (r.path)}{@render row(r)}{/each}</ul>{/if}
  {#if items.length > max}<button type="button" class="btn sm more" onclick={() => (all = !all)}>{all ? t("start-weniger") : t("start-weitere", { n: items.length - max })}</button>{/if}
</div>

<style>
  .rl { display: flex; flex-direction: column; gap: 4px; }
  .kopf { font-size: 10.5px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--dim); margin: 6px 2px 0; }
  .plain { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 12px; }
  .proj { display: grid; grid-template-columns: auto minmax(0, 1fr) 28px 28px; grid-template-areas: 'pic pic pic pic' 'main main star mn' 'sys ago ago ago'; align-items: center; gap: 4px 8px; padding: 0 8px 10px; background: var(--card); border: 1px solid var(--line2); border-radius: 12px; overflow: hidden; }
  .pic { grid-area: pic; margin: 0 -8px; width: calc(100% + 16px); height: 150px; object-fit: contain; background: #0b0c10; image-rendering: pixelated; display: block; } .pic.ph { display: flex; align-items: center; justify-content: center; font: 700 44px var(--font-mono, monospace); color: rgba(255, 255, 255, 0.85); background: hsl(var(--h) 38% 30%); image-rendering: auto; }
  .proj .main { grid-area: main; } .proj .star { grid-area: star; } .proj .mn { grid-area: mn; } .proj .sys { grid-area: sys; justify-self: start; } .proj .ago, .proj .gone { grid-area: ago; justify-self: end; }
  .proj:hover, .proj:focus-within { background: var(--hover); border-color: var(--line2); }
  .sys { font-size: 11px; font-weight: 600; color: var(--accent); border: 1px solid var(--line2); border-radius: 8px; padding: 2px 6px; text-align: center; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .main { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; min-width: 0; border: 0; background: none; color: var(--text); font: inherit; padding: 2px 0; cursor: pointer; text-align: left; }
  .main:disabled { cursor: default; }
  .name { font-weight: 600; font-size: 13.5px; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .path { font-size: 11px; color: var(--dim); max-width: 100%; white-space: nowrap; }
  .ago, .gone { font-size: 11.5px; color: var(--dim); white-space: nowrap; } .gone { color: var(--err, #c33); }
  .missing .name, .missing .path, .missing .sys { opacity: 0.55; }
  .proj .main { padding-top: 6px; } .star { color: var(--dim); opacity: 0; } .star.on { opacity: 1; color: var(--accent); }
  .mn { opacity: 0; }
  .proj:hover .star, .proj:focus-within .star, .proj:hover .mn, .proj:focus-within .mn, .missing .mn { opacity: 1; }
  .more { align-self: flex-start; margin-top: 4px; }

</style>
