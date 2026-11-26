<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { cfg, kurz, ri, rm, romOrdner, romsFestnageln, romsFremdSuchen, romsHaken, romsImportieren, romsLaden, romsListeKopieren, romsWaehlen } from "../../lib/settings.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import { t } from "../../lib/i18n.svelte";
  // ROMs (E12.2): Stand je System und Rolle (erkannt, Ersatz, fehlt) mit Prüfsummen, die durchsuchten Orte, ROM-Ordner öffnen, ROMs anderer Programme mitbenutzen. ROMs werden nie mitgeliefert.
  const sys = $derived(rm.info?.systems.find((s) => s.id === rm.sel));
  const farbe: Record<string, string> = { recognized: "ok", replacement: "warn", unknown: "warn", missing: "err", unusable: "err" };
  const ampel = (s: NonNullable<typeof rm.info>["systems"][number]) => (s.missingRequired.length ? "err" : s.roles.every((r) => !r.required || r.status === "recognized") ? "ok" : "warn");
</script>

<div class="rm" data-es="roms">
  <div class="card"><div class="row"><button type="button" class="btn sm" data-rm="folder" onclick={() => void romOrdner()}>{t("roms-ordner-oeffnen")}</button><button type="button" class="btn sm" data-rm="recheck" disabled={rm.busy} onclick={() => void romsLaden()}>{t("roms-neu-pruefen")}</button></div>
    <Switch checked={!!cfg.draft?.roms?.use_foreign} label={t("roms-fremde")} onchange={(on) => { if (cfg.draft) { cfg.draft.roms = { ...(cfg.draft.roms ?? {}), use_foreign: on }; void romsLaden(); } }} />
    <p class="small dim">{t("roms-fremde-hinweis")}</p><p class="small dim">{t("roms-woher")}</p></div>
  <div class="card" data-rm="import"><h3>{t("roms-import")}</h3>
    <div class="row"><button type="button" class="btn sm" data-rm="pick" disabled={ri.busy} onclick={() => void romsWaehlen()}>{t("roms-datei-waehlen")}</button><button type="button" class="btn sm" data-rm="foreign" disabled={ri.busy} onclick={() => void romsFremdSuchen()}>{t("roms-fremde-suchen")}</button><button type="button" class="btn sm" data-rm="copylist" disabled={!rm.info} onclick={() => void romsListeKopieren()}>{t("roms-liste-kopieren")}</button></div>
    <p class="small dim">{t("roms-import-hinweis")}</p>
    {#if ri.planned && ri.items.length}
      <ul class="orte" data-rm="plan">{#each ri.items as it (it.i)}
        <li class="row"><input type="checkbox" data-rm="haken" aria-label={t("roms-import-haken", { name: it.name })} disabled={it.action !== "copy"} checked={ri.pick.includes(it.i)} onchange={(e) => romsHaken(it.i, e.currentTarget.checked)} />
          <span class="mono small">{it.name}</span><span class="small dim">{it.system ?? "–"}{it.role ? ` · ${it.role}` : ""}{it.label ? ` · ${it.label}` : ""}{it.source ? ` · ${it.source}` : ""}</span>{#if it.system && rm.sel && it.system !== rm.sel}<span class="small" data-rm="anderes-system">{t("roms-import-nach", { system: rm.info?.systems.find((x) => x.id === it.system)?.label ?? it.system })}</span>{/if}
          <span class={`small ${it.action === "copy" ? "ok" : it.action === "present" ? "dim" : "warn"}`} data-rm="aktion">{it.action === "copy" ? t("roms-import-kopieren") : it.action === "present" ? t("roms-import-da") : it.reason}</span></li>
      {/each}</ul>
      <div class="row"><button type="button" class="btn pri sm" data-rm="run" disabled={ri.busy || !ri.pick.length} onclick={() => void romsImportieren()}>{t("roms-import-ausfuehren", { n: ri.pick.length })}</button><button type="button" class="btn sm" data-rm="cancel" onclick={() => { ri.items = []; ri.pick = []; ri.planned = false; }}>{t("kopierdlg-abbrechen")}</button></div>
    {/if}
    {#if ri.msg}<p class="small" role="status" data-rm="msg">{ri.msg}</p>{/if}
  </div>
  {#if rm.info}
    <div class="row sysleiste" role="tablist" aria-label={t("roms-systeme")}>{#each rm.info.systems as s (s.id)}<button type="button" role="tab" class={`tab ${ampel(s)}`} class:on={s.id === rm.sel} aria-selected={s.id === rm.sel} data-rm="system" data-s={s.id} onclick={() => (rm.sel = s.id)}>● {s.label}</button>{/each}</div>
    {#if sys}
      <div class="card" data-rm="rollen">
        {#each sys.roles as r (r.id)}
          <div class="rolle" data-rm="rolle" data-r={r.id}><div class="row"><b class={farbe[r.status]} data-rm="status">{r.status === "missing" && !r.required ? t("roms-fehlt-optional") : r.statusLabel}</b><span>{r.label}</span><span class="mono small dim">{r.path ? r.path.split("/").pop() : r.file}</span>{#if r.others.length}<select class="inp" data-rm="pin" aria-label={t("roms-fest-label", { rolle: r.label })} onchange={(e) => void romsFestnageln(sys!.id, r.id, e.currentTarget.value)}><option value="">{t("roms-fest-auto")}</option>{#each [r.path ? r.path.split("/").pop()! : "", ...r.others].filter(Boolean) as o (o)}<option value={o}>{o}</option>{/each}</select>{/if}</div>
            {#if r.path}<p class="small dim mono">{[r.place, r.revision, r.size ? `${r.size} ${t("roms-byte")}` : "", r.crc32 ? `CRC-32 ${r.crc32}` : "", r.sha1 ? `SHA-1 ${r.sha1.slice(0, 12)}…` : ""].filter(Boolean).join(" · ")}</p>{:else if r.expected.length}<p class="small dim">{t("roms-erwartet", { n: r.expected.join(" / ") })}</p>{/if}</div>
        {/each}
      </div>
    {/if}
    <div class="card"><h3>{t("roms-orte")}</h3><ul class="orte" data-rm="orte">{#each rm.info.searched as o (o.path)}<li><span class="small dim">{o.stage}</span> <span class="mono small">{kurz(o.path)}</span> <span class="small" class:ok={o.files > 0}>{o.exists ? t("roms-dateien", { n: o.files }) : t("roms-fehlt-ort")}</span></li>{/each}</ul></div>
  {:else}<p class="dim">{t("einst-laedt")}</p>{/if}
</div>

<style>
  .card { flex: none; background: var(--raise); border-radius: 10px; padding: 12px; display: flex; flex-direction: column; gap: 8px; max-width: 760px; } .card p { margin: 0; } .rm { display: flex; flex-direction: column; gap: 10px; }
  h3 { margin: 0 0 6px; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); } .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .small { font-size: 12px; } .dim { color: var(--dim); }
  .tab { padding: 5px 12px; border-radius: 8px; border: 1px solid var(--line2); } .tab.on { background: var(--accentbg); border-color: var(--accent); } .tab.ok { color: var(--ok); } .tab.warn { color: var(--warn); } .tab.err { color: var(--err); }
  .ok { color: var(--ok); } .warn { color: var(--warn); } .err { color: var(--err); } .rolle { padding: 6px 0; border-bottom: 1px solid var(--line2); } .rolle b { min-width: 150px; } .orte { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 3px; }
  [data-rm=plan] { max-height: 300px; overflow: auto; padding-right: 6px; }
</style>
