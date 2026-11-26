<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { tick } from "svelte";
  import { t } from "../i18n.svelte";
  import { places, runCommand } from "../commands";
  import { ed } from "../editor.svelte";
  import { aktiv as spriteAktiv } from "../sprite.svelte";
  import { ui } from "../state.svelte";
  import { call, type RecentProject, type TreeNode } from "../api";
  const files = (ns: TreeNode[]): string[] => ns.flatMap((n) => (n.kind === "dir" ? files(n.children ?? []) : [n.rel]));
  import { isTurbo } from "../turbo.svelte";
  import Kbd from "./Kbd.svelte";
  const DEBUG_IDS = ["debuggen", "debuggen_aktiv", "debug_weiter", "debug_pause", "debug_ueber", "debug_hinein", "debug_raus", "debug_hierher"];
  // Befehlspalette (⌘K / Strg+K): Befehle der Tabelle und Orte der Navigation. Alle Wörter müssen im Namen vorkommen (wie `vc_commands::filtere`).
  interface Entry { id: string; name: string; group: "palette-group-commands" | "palette-group-places" | "palette-dateien" | "palette-projekte"; keys: string; enabled: boolean; reason?: string; source?: string | null }
  let q = $state("");
  let sel = $state(0);
  let input = $state<HTMLInputElement>();
  let listEl = $state<HTMLElement>();
  let before: HTMLElement | null = null;
  let recent = $state<RecentProject[]>([]);

  const entries = $derived.by<Entry[]>(() => {
    const words = q.toLowerCase().split(/\s+/).filter(Boolean);
    const all: Entry[] = [
      ...ui.commands.map((c) => {
        const base = c.needs === "none" || (c.needs === "project" && !!ui.project) || (c.needs === "editor" && (ed.active !== null || spriteAktiv()));
        // Turbo BASIC XL: Debuggen ist ausgegraut (T7), die Diskettenbefehle gehören nur zu Turbo-Projekten
        const noDebug = isTurbo() && DEBUG_IDS.includes(c.id), noTurbo = c.id.startsWith("turbo_") && !isTurbo();
        return { id: c.id, name: c.title ?? t(`cmd-${c.id}`), source: c.source ?? null, group: "palette-group-commands" as const, keys: c.keysDisplay, enabled: base && !noDebug && !noTurbo, reason: noDebug ? t("turboui-debuggen-hinweis") : noTurbo ? t("turboui-nur-turbo") : undefined };
      }),
      ...files(ui.project?.tree ?? []).map((f) => ({ id: `file:${f}`, name: f, group: "palette-dateien" as const, keys: "", enabled: true })),
      ...recent.filter((r) => !r.missing && r.path !== ui.project?.root).map((r) => ({ id: `recent:${r.path}`, name: t("palette-projekt", { name: r.name, path: r.pretty }), group: "palette-projekte" as const, keys: "", enabled: true })),
      ...places().map((p) => ({ id: p.id, name: t(p.label), group: "palette-group-places" as const, keys: "", enabled: true })),
      { id: "ui:gallery", name: t("gallery-title"), group: "palette-group-places", keys: "", enabled: true },
      { id: "ui:measure", name: t("measure-title"), group: "palette-group-places", keys: "", enabled: true },
      ...(["auto", "de", "en"] as const).map((l) => ({ id: `lang:${l}`, name: t("lang-entry", { name: l === "auto" ? t("lang-choice-auto") : l === "de" ? "Deutsch" : "English" }), group: "palette-group-places" as const, keys: "", enabled: true })),
      { id: "lang:xx", name: t("lang-entry", { name: t("lang-choice-pseudo") }), group: "palette-group-places", keys: "", enabled: true },
    ];
    const hit = all.filter((e) => words.every((w) => e.name.toLowerCase().includes(w)));
    const rank = (e: Entry) => (e.enabled ? 0 : 2) + (words.length === 0 || e.name.toLowerCase().startsWith(words[0]!) ? 0 : 1);
    return hit.map((e, i) => ({ e, i })).sort((a, b) => rank(a.e) - rank(b.e) || a.i - b.i).map((x) => x.e).slice(0, 80);
  });

  $effect(() => { void q; sel = 0; });
  $effect(() => { if (ui.palette) { before = document.activeElement as HTMLElement | null; q = ""; sel = 0; tick().then(() => input?.focus()); call("recent_list").then((r) => (recent = r)).catch(() => {}); } });

  function close() { ui.palette = false; before?.focus(); }
  async function run(e: Entry | undefined) { if (!e || !e.enabled) return; close(); await runCommand(e.id); }
  function key(ev: KeyboardEvent) {
    if (ev.key === "Escape") { ev.preventDefault(); close(); }
    else if (ev.key === "ArrowDown") { ev.preventDefault(); sel = Math.min(entries.length - 1, sel + 1); scroll(); }
    else if (ev.key === "ArrowUp") { ev.preventDefault(); sel = Math.max(0, sel - 1); scroll(); }
    else if (ev.key === "Enter") { ev.preventDefault(); void run(entries[sel]); }
    else if (ev.key === "Tab") ev.preventDefault();
  }
  function scroll() { tick().then(() => listEl?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: "nearest" })); }
  const base_disabled = (e: Entry) => !e.enabled;
  const KB = $derived((e: Entry) => (e.keys ? [e.keys] : []));
</script>

{#if ui.palette}
  <div class="pal-back" role="presentation" onpointerdown={(e) => e.target === e.currentTarget && close()}>
    <div class="pal" role="dialog" aria-modal="true" aria-label={t("cmd-palette")} tabindex="-1" onkeydown={key}>
      <input bind:this={input} bind:value={q} role="combobox" aria-expanded="true" aria-controls="pal-list" aria-autocomplete="list" aria-activedescendant={entries[sel] ? `pal-${sel}` : undefined} placeholder={t("cmd-palette")} aria-label={t("cmd-palette")} autocomplete="off" spellcheck="false" />
      <ul id="pal-list" role="listbox" bind:this={listEl} aria-label={t("palette-group-commands")}>
        {#each entries as e, i (e.id)}
          {#if i === 0 || entries[i - 1]!.group !== e.group}<li class="grp lbl" role="presentation">{t(e.group)}</li>{/if}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <li id={`pal-${i}`} role="option" aria-selected={i === sel} aria-disabled={e.enabled ? undefined : "true"} onpointermove={() => (sel = i)} onclick={() => run(e)}>
            <span>{e.name}</span>{#if e.source}<span style="margin-left:8px;padding:1px 7px;border-radius:8px;background:var(--raise);color:var(--dim);font-size:11px" title={t("palette-herkunft", { paket: e.source })}>{e.source}</span>{/if}{#if e.reason && base_disabled(e)}<span class="dim reason"> · {e.reason}</span>{/if}<span class="sp"></span>{#if e.keys}<Kbd parts={KB(e)} />{/if}
          </li>
        {:else}
          <li class="grp" role="presentation" style="color:var(--dim)">{t("palette-empty")}</li>
        {/each}
      </ul>
      <div class="foot">{t("palette-hint")}</div>
      <div class="sr-only" aria-live="polite">{t("palette-count", { n: entries.length })}</div>
    </div>
  </div>
{/if}
