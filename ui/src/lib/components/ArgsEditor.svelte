<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { call, type ArgsInfo, type ArgParam } from "../api";
  import { t } from "../i18n.svelte";
  import { fail } from "../state.svelte";
  // Parametereditor (E12.2b): Startparameter aus dem Katalog für einen Emulator (`kind`) oder den Assembler (`asm`). `owner` trägt `opts` (Kennung → Wert) und `args` (freies Feld); beides wird im Entwurf geändert.
  let { kind, owner, ziel = "c64", feld = "args", titel }: { kind: string; owner: { opts?: Record<string, string>; args?: string; [k: string]: unknown }; ziel?: string; feld?: string; titel: string } = $props();
  let info = $state<ArgsInfo | null>(null); let offen = $state(false); let gruppen = $state<Record<string, boolean>>({});
  const opts = $derived(owner.opts ?? {}); const frei = $derived((owner[feld] as string | undefined) ?? "");
  let timer: ReturnType<typeof setTimeout> | null = null;
  function laden() { if (timer) clearTimeout(timer); timer = setTimeout(async () => { try { info = await call("args_info", { kind, opts: { ...opts }, free: frei, target: ziel }); } catch (e) { fail(e); } }, 120); }
  $effect(() => { void JSON.stringify(opts); void frei; void kind; if (offen) laden(); });
  $effect(() => { if (!info && !offen) { void call("args_info", { kind, opts: { ...opts }, free: frei, target: ziel }).then((r) => (info = r)).catch(() => {}); } });
  function setze(id: string, v: string) { const o = { ...(owner.opts ?? {}) }; if (v === "") delete o[id]; else o[id] = v; owner.opts = o; }
  const markiert = (p: ArgParam) => (p.off_flag ? `${p.flag} / ${p.off_flag}` : p.flag);
  const tip = (p: ArgParam) => [`${p.hint} (${markiert(p)})`, p.default ? t("args-standard-des-programms", { d: p.default }) : "", p.example ? t("args-beispiel", { x: p.example }) : "", p.min != null || p.max != null ? t("args-bereich", { a: p.min ?? "–", b: p.max ?? "–" }) : "", p.conflicts.length ? t("args-nicht-zusammen", { x: p.conflicts.join(", ") }) : "", p.requires.length ? t("args-braucht", { x: p.requires.join(", ") }) : ""].filter(Boolean).join("\n");
  async function migrieren() { try { const r = await call("args_migrate", { kind, opts: { ...opts }, free: frei }); owner.opts = r.opts; owner[feld] = r.free; } catch (e) { fail(e); } }
  const gesetzt = (id: string) => (opts[id] ?? "").trim() !== "";
  const nGruppe = (g: string) => info?.catalog?.params.filter((p) => p.group === g && gesetzt(p.id)).length ?? 0;
  const symbol = (l: string) => (l === "error" ? "✖" : l === "warn" ? "⚠" : "ℹ");
  const fokus = $state({ key: "" });
</script>

<div class="ae" data-args={kind}>
  <button type="button" class="kopf" aria-expanded={offen} data-args="toggle" onclick={() => (offen = !offen)}>{offen ? "▾" : "▸"} {titel}{#if info?.changed}<span class="dim n"> · {t("args-n-gesetzt", { n: info.changed })}</span>{/if}</button>
  {#if offen && info}
    {#if info.catalog}
      <p class="small dim">{t("args-katalog", { l: info.catalog.label })}{info.catalog.verified.startsWith("nicht geprüft") ? " " + t("args-nicht-geprueft") : ""}</p>
      {#each info.catalog.groups as g (g.id)}{#if info.catalog.params.some((p) => p.group === g.id)}
        <button type="button" class="gk" aria-expanded={gruppen[g.id] ?? nGruppe(g.id) > 0} data-args="gruppe" onclick={() => (gruppen[g.id] = !(gruppen[g.id] ?? nGruppe(g.id) > 0))}>{(gruppen[g.id] ?? nGruppe(g.id) > 0) ? "▾" : "▸"} {g.label}{#if nGruppe(g.id)}<span class="dim"> · {t("args-n-gesetzt", { n: nGruppe(g.id) })}</span>{/if}</button>
        {#if gruppen[g.id] ?? nGruppe(g.id) > 0}
          {#each info.catalog.params.filter((p) => p.group === g.id) as p (p.id)}
            <div class="pz" title={tip(p)} data-args="param" data-p={p.id}>
              {#if p.type === "switch" && !p.off_flag}<label class="r"><input type="checkbox" checked={opts[p.id] === "on"} onchange={(e) => setze(p.id, e.currentTarget.checked ? "on" : "")} />{p.label}</label>
              {:else if p.type === "switch"}<span>{p.label}</span><select class="inp s" aria-label={p.label} value={opts[p.id] ?? ""} onchange={(e) => setze(p.id, e.currentTarget.value)}><option value="">{t("args-standard")}</option><option value="on">{t("args-an")}</option><option value="off">{t("args-aus")}</option></select>
              {:else if p.type === "choice"}<span>{p.label}</span><select class="inp s" aria-label={p.label} value={opts[p.id] ?? ""} onchange={(e) => setze(p.id, e.currentTarget.value)}><option value="">{t("args-standard")}</option>{#each p.choices as c (c.value)}<option value={c.value}>{c.label} ({c.value})</option>{/each}</select>
              {:else}<span>{p.label}</span><input class="inp s" aria-label={p.label} type={p.type === "number" ? "number" : "text"} placeholder={p.example ?? ""} value={opts[p.id] ?? ""} onfocus={() => (fokus.key = p.id)} oninput={(e) => setze(p.id, e.currentTarget.value)} />{/if}
              <span class="mono small dim">{markiert(p)}</span>{#if gesetzt(p.id)}<button type="button" class="ibtn sm" aria-label={t("args-zuruecksetzen-eins")} onclick={() => setze(p.id, "")}>×</button>{/if}
            </div>
          {/each}
        {/if}
      {/if}{/each}
    {:else}<p class="small dim">{t("args-kein-katalog")}</p>{/if}
    <label class="small dim">{t("args-weitere")}<input class="inp mono" data-args="free" value={frei} oninput={(e) => { owner[feld] = e.currentTarget.value; }} /></label>
    <div class="row small"><span class="dim">{t("args-variable-einfuegen")}</span>{#each info.placeholders as p (p.name)}<button type="button" class="btn sm" title={`${p.hint}${p.example ? ` — ${p.example}` : ""}`} data-args="ph" onclick={() => { if (fokus.key && opts[fokus.key] !== undefined) setze(fokus.key, (opts[fokus.key] ?? "") + `{${p.name}}`); else owner[feld] = (frei && !frei.endsWith(" ") ? frei + " " : frei) + `{${p.name}}`; }}>{`{${p.name}}`}</button>{/each}</div>
    {#if info.migratable > 0}<p class="small"><span class="dim">{t("args-migrierbar", { n: info.migratable })}</span> <button type="button" class="btn sm" data-args="migrieren" onclick={() => void migrieren()}>{t("args-uebernehmen")}</button></p>{/if}
    {#each info.issues as i (i.text)}<p class={`small is ${i.level}`} data-args="issue"><span aria-hidden="true">{symbol(i.level)}</span> {i.text}</p>{/each}
    <div class="vor"><span class="small dim">{t("args-vorschau")}</span><code data-args="vorschau">{info.preview}</code></div>
    <div class="row"><button type="button" class="btn sm" data-args="zurueck" disabled={!Object.keys(opts).length && !frei.trim()} onclick={() => { owner.opts = {}; owner[feld] = ""; }}>{t("args-auf-standard")}</button></div>
  {/if}
</div>

<style>
  .ae { display: flex; flex-direction: column; gap: 6px; } .kopf, .gk { text-align: left; font-weight: 600; padding: 4px 6px; border-radius: 6px; } .gk { font-size: 12.5px; padding-left: 14px; } .kopf:hover, .gk:hover { background: var(--hover); }
  .pz { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; padding: 2px 6px 2px 24px; font-size: 12.5px; } .r { display: flex; align-items: center; gap: 6px; flex-direction: row; } .inp.s { width: 220px; flex: none; }
  .row { display: flex; gap: 6px; flex-wrap: wrap; align-items: center; } .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .n { font-weight: 400; } label { display: flex; flex-direction: column; gap: 4px; } .inp { width: 100%; }
  .is.error { color: var(--err); } .is.warn { color: var(--warn); } .is.info { color: var(--dim); } .vor { background: var(--card); border-radius: 8px; padding: 8px; display: flex; flex-direction: column; gap: 3px; } code { font-family: var(--font-mono, monospace); font-size: 12px; word-break: break-all; }
</style>
