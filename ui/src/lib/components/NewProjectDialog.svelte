<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { cards, closeNewProject, createProject, nameOk, np, pickDir, setBasic, setCard, setField, setSystem, toggleExtra } from "../neuprojekt.svelte";
  import { t } from "../i18n.svelte";
  import Dialog from "./Dialog.svelte";
  import Segment from "./Segment.svelte";
  // Neues Projekt: Zielsystem, Vorlage, Details; Vorschau der Dateistruktur. Dieselben Eingaben wie in der alten Oberfläche.
  const f = $derived(np.form);
  const o = $derived(np.opt);
  const card = $derived(f.card === null || !np.info ? null : np.info.cards[f.card] ?? null);
  const others = $derived(card && np.info ? np.info.systems.filter((s) => s.id !== f.system && card.systems.includes(s.id)) : []);
  const ok = $derived(nameOk(f.name) && f.card !== null && !np.busy);
  const files = $derived(np.files.filter((x) => !x.endsWith("/")).length);
  const norm = $derived(o?.norm ? ` ${f.norm.toUpperCase()}` : "");
  const sys = $derived(np.info?.systems.find((s) => s.id === f.system)?.name ?? f.system);
  function key(e: KeyboardEvent) { if (e.key === "Enter" && (e.target as HTMLElement).tagName === "INPUT" && ok) { e.preventDefault(); void createProject(); } }
</script>

<Dialog wide bind:open={np.open} title={t("neuprojekt-neues-projekt-erstellen")}>
  {#snippet children()}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="np" onkeydown={key}>
      <p class="sub">{t("neuprojekt-zielsystem-vorlage-und-details-waehlen")}</p>
      <div class="cols">
        <section aria-label={t("np-zielsystem")}>
          <h3>{t("np-zielsystem")}</h3>
          <div role="radiogroup" aria-label={t("np-zielsystem")} class="list">
            {#each np.info?.systems ?? [] as s (s.id)}
              <button type="button" role="radio" aria-checked={f.system === s.id} class="opt" class:on={f.system === s.id} onclick={() => void setSystem(s.id)}><b>{s.name}</b><small>{s.cpu}</small></button>
            {/each}
          </div>
          <div class="hw"><span>{t("neuprojekt-hardware-profil")}</span><span class="chip" class:bad={!o?.valid}>{o?.valid ? t("neuprojekt-gueltig") : t("neuprojekt-fehlt")}</span></div>
          {#if o?.valid}<p class="mono small">{t("neuprojekt-cpu-p1-p2", { p1: o.cpu, p2: o.chips })}</p>{/if}
        </section>

        <section aria-label={t("np-vorlage")}>
          <div class="hd"><h3>{t("np-vorlage")}</h3>
            <Segment small label={t("np-sprache")} items={[{ id: "asm", label: t("neuprojekt-assembler") }, { id: "basic", label: "BASIC" }]} value={f.basic ? "basic" : "asm"} onselect={(id) => setBasic(id === "basic")} /></div>
          {#if cards().length === 0}<p class="dim">{t("neuprojekt-fuer-diese-kombination-aus-zielsystem-und")}</p>{/if}
          <div role="radiogroup" aria-label={t("np-vorlage")} class="list">
            {#each cards() as c (c.i)}
              <button type="button" role="radio" aria-checked={f.card === c.i} class="opt card" class:on={f.card === c.i} onclick={() => setCard(c.i)}>
                <span class="top"><b>{c.name}</b><span class="chip">{c.basic ? "BASIC" : "vcasm"}</span></span>
                <small>{c.description}</small>
                <small class="mono">{c.files.map((d) => d.replace("{system}", f.system)).join("  ")}</small>
              </button>
            {/each}
          </div>
          {#if f.basic}<p class="dim small">{t("neuprojekt-basic-wird-im-dialekt-des-zielsystems")}</p>{/if}
        </section>

        <section aria-label={t("neuprojekt-projekt-details")}>
          <h3>{t("neuprojekt-projekt-details")}</h3>
          <label class="fld"><span>{t("neuprojekt-projektname")}</span><input class="inp mono" value={f.name} aria-invalid={!nameOk(f.name)} oninput={(e) => setField("name", e.currentTarget.value)} /></label>
          {#if !nameOk(f.name)}<p class="err small" role="alert">{t("neuprojekt-nur-buchstaben-ziffern-und")}</p>{/if}
          <div class="fld"><span>{t("neuprojekt-speicherort")}</span><div class="row"><span class="mono small grow" title={f.dir}>{np.dirPretty}</span><button type="button" class="btn" onclick={() => void pickDir()}>{t("neuprojekt-waehlen")}</button></div></div>
          {#if f.basic && o}<div class="fld"><span>{t("neuprojekt-basic-dialekt")}</span><Segment small label={t("neuprojekt-basic-dialekt")} items={o.dialects.map((d) => ({ id: d.id, label: d.label }))} value={f.dialect} onselect={(v) => setField("dialect", v)} /></div>{/if}
          {#if o?.norm}<div class="fld"><span>{t("neuprojekt-videonorm")}</span><Segment small label={t("neuprojekt-videonorm")} items={[{ id: "pal", label: "PAL" }, { id: "ntsc", label: "NTSC" }]} value={f.norm} onselect={(v) => setField("norm", v)} /></div>{/if}
          {#if o && o.memory.length}<div class="fld"><span>{t("neuprojekt-speicherausbau")}</span><Segment small label={t("neuprojekt-speicherausbau")} items={o.memory} value={f.memory} onselect={(v) => setField("memory", v)} /></div>{/if}
          {#if o}
            <div class="fld"><span>{t("neuprojekt-standard-emulator")}</span>
              {#if o.emulators.length}<Segment small label={t("neuprojekt-standard-emulator")} items={o.emulators.map((e) => ({ id: e.id, label: e.global ? t("neuprojekt-p1-einstellungen", { p1: e.name }) : e.name }))} value={f.emulator} onselect={(v) => setField("emulator", v)} />
              {:else}<span class="small"><span class="chip">{o.globalEmulatorKind}</span> {t("neuprojekt-aus-den-einstellungen")}</span>{/if}</div>
            <div class="fld"><span>{t("neuprojekt-startziel-f5")}</span><Segment small label={t("neuprojekt-startziel-f5")} items={[{ id: "", label: t("neuprojekt-emulator") }, ...o.machines.map((m) => ({ id: m.id, label: t("neuprojekt-p1-hardware", { p1: m.name }) }))]} value={f.startOn} onselect={(v) => setField("startOn", v)} />
              {#if f.startOn !== o.globalStart}<p class="warn small">{t("neuprojekt-weicht-von-den-einstellungen-ab-wird")}</p>{/if}</div>
            <div class="fld"><span>{t("neuprojekt-diskette")}</span><Segment small label={t("neuprojekt-diskette")} items={[{ id: "", label: t("neuprojekt-keine") }, ...o.disks.map((d) => ({ id: d, label: d.toUpperCase() }))]} value={f.disk} onselect={(v) => setField("disk", v)} /></div>
          {/if}
          <div class="fld"><span>{t("neuprojekt-zeichensatz-des-projekts")}</span><Segment small label={t("neuprojekt-zeichensatz-des-projekts")} items={[{ id: "auto", label: t("neuprojekt-automatisch") }, { id: "own", label: t("neuprojekt-nur-standard-eigen") }, { id: "none", label: t("neuprojekt-keinen") }]} value={f.charset} onselect={(v) => setField("charset", v)} />
            <p class="dim small">{f.charset === "auto" ? t("neuprojekt-dein-rom-hat-vorrang-nur-ein") : f.charset === "own" ? t("neuprojekt-immer-der-eigene-satz-vc-standard") : t("np-kein-zeichensatz")}</p></div>
          {#if (np.info?.structures.length ?? 0) > 1}<div class="fld"><span>{t("np-struktur")}</span><select class="inp" aria-label={t("np-struktur")} value={String(f.structure)} onchange={(e) => setField("structure", Number(e.currentTarget.value))}>{#each np.info?.structures ?? [] as st, i (st.id)}<option value={String(i)}>{st.name}</option>{/each}</select>
            <p class="dim small">{np.info?.structures[f.structure]?.description ?? ""}</p></div>{/if}
          <label class="chk"><input type="checkbox" checked={f.readme} onchange={(e) => setField("readme", e.currentTarget.checked)} />{t("neuprojekt-readme-md-mit-bauhinweisen-anlegen")}</label>
          <label class="chk"><input type="checkbox" checked={f.git} onchange={(e) => setField("git", e.currentTarget.checked)} />{t("neuprojekt-git-repository-anlegen-gitignore-fuer-build")}</label>
          {#if others.length}<div class="fld"><span>{t("neuprojekt-weitere-zielsysteme")}</span>{#each others as s (s.id)}<label class="chk"><input type="checkbox" checked={f.extra.includes(s.id)} onchange={(e) => toggleExtra(s.id, e.currentTarget.checked)} />{s.name}</label>{/each}</div>{/if}
          <div class="fld tree"><span>{t("neuprojekt-dateistruktur")} <span class="chip">{t("neuprojekt-n-dateien", { n: files })}</span></span>
            <ul class="mono small" aria-label={t("neuprojekt-dateistruktur")}><li><b>{f.name}/</b></li>{#each np.files as p (p)}<li style={`padding-left:${(p.replace(/\/$/, "").split("/").length) * 14}px`}>{p.replace(/\/$/, "").split("/").pop()}{p.endsWith("/") ? "/" : ""}{#if p === "build/"} <small class="dim">{t("neuprojekt-entsteht-beim-bauen")}</small>{/if}</li>{/each}</ul></div>
        </section>
      </div>
    </div>
  {/snippet}
  {#snippet footer()}
    <span class="sum">{t("neuprojekt-ziel-p1-norm-p3-p4", { p1: sys, norm, p3: f.basic ? (o?.dialects.find((d) => d.id === f.dialect)?.label ?? "BASIC") : "Assembler (vcasm)", p4: card ? ` · ${card.name}` : "" })}</span>
    {#if np.error}<span class="err" role="alert">{np.error}</span>{/if}
    <button type="button" class="btn" onclick={closeNewProject}>{t("neuprojekt-abbrechen")}</button>
    <button type="button" class="btn pri" disabled={!ok} onclick={() => void createProject()}>{t("neuprojekt-projekt-erstellen-und-oeffnen")}</button>
  {/snippet}
</Dialog>

<style>
  .np { color: var(--text); }
  .sub { margin: 0 0 10px; color: var(--dim); }
  .cols { display: grid; grid-template-columns: minmax(180px, 2fr) minmax(220px, 3fr) minmax(280px, 4fr); gap: 18px; align-items: start; }
  @media (max-width: 1000px) { .cols { grid-template-columns: 1fr; } }
  h3 { margin: 0 0 8px; font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--dim); }
  .hd { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 8px; } .hd h3 { margin: 0; }
  .list { display: flex; flex-direction: column; gap: 6px; }
  .opt { display: flex; flex-direction: column; gap: 2px; text-align: left; padding: 8px 10px; border-radius: var(--r-ctl); border: 1px solid var(--line); background: var(--raise); color: var(--text); font: inherit; cursor: pointer; }
  .opt:hover { background: var(--hover); } .opt.on { border-color: var(--accent); background: var(--accentbg); }
  .opt small { color: var(--dim); font-size: 11.5px; } .opt .top { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .hw { display: flex; justify-content: space-between; align-items: center; margin-top: 12px; font-size: 12px; color: var(--dim); }
  .chip { display: inline-block; padding: 1px 8px; border-radius: 10px; border: 1px solid var(--line2); font-size: 11px; color: var(--dim); } .chip.bad { color: var(--err, #c33); border-color: var(--err, #c33); }
  .fld :global(.seg) { display: flex; flex-wrap: wrap; max-width: 100%; } .fld :global(.seg button) { white-space: normal; text-align: left; }
  .fld { display: flex; flex-direction: column; gap: 4px; margin: 0 0 10px; } .fld > span { font-size: 11.5px; color: var(--dim); }
  .inp { height: 30px; padding: 0 8px; border-radius: var(--r-ctl); border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; }
  .row { display: flex; align-items: center; gap: 8px; } .grow { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chk { display: flex; gap: 8px; align-items: center; margin: 4px 0; font-size: 13px; color: var(--text); }
  .small { font-size: 11.5px; margin: 0; } .dim { color: var(--dim); } .err { color: var(--err, #c33); } .warn { color: var(--warn, #b80); }
  .tree ul { list-style: none; margin: 0; padding: 8px; border: 1px solid var(--line); border-radius: var(--r-ctl); background: var(--bg); max-height: 200px; overflow: auto; }
  .sum { flex: 1; text-align: left; color: var(--dim); font-size: 12px; align-self: center; }
</style>
