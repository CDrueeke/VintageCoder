<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { cfg, emuSuchen, emuTesten, kurz, maschineTesten, pruefen, zielOp, zl } from "../../lib/settings.svelte";
  import EmuInstall from "../../lib/components/EmuInstall.svelte";
  import ArgsEditor from "../../lib/components/ArgsEditor.svelte";
  import Switch from "../../lib/components/Switch.svelte";
  import { t } from "../../lib/i18n.svelte";
  // Zielsysteme und Hardware (E12.2): links die Systeme des Entwurfs, rechts Startziel, Emulatoren und echte Maschinen des gewählten Systems. Alles ist ein Entwurf; „Speichern“ steht unten auf der Seite.
  type Emu = Record<string, any>; type Mach = Record<string, any>; type Ziel = { system: string; emulators: Emu[]; start_on?: string }; // eslint-disable-line @typescript-eslint/no-explicit-any
  const d = $derived(cfg.draft);
  const ziele = $derived((d?.targets ?? []) as Ziel[]);
  const maschinen = $derived((d?.machines ?? []) as Mach[]);
  const ziel = $derived(ziele.find((x) => x.system === zl.sel));
  const info = $derived(zl.info.find((x) => x.id === zl.sel));
  const nameVon = (id: string) => zl.info.find((x) => x.id === id)?.name ?? id;
  const frei = $derived(zl.info.filter((x) => !ziele.some((z) => z.system === x.id)));
  const eigene = $derived(maschinen.filter((m) => (m.target || (m.kind === "mega65" ? "mega65" : "c64")) === zl.sel));
  let neueArt = $state(""); let neueHw = $state(""); let neuesSystem = $state("");
  const hat = (k: string) => /atari800|xemu|mega65/.test(k);
  const status = (e: Emu) => zl.anwesend[e.id];
</script>

<div class="zl" data-es="ziele">
  <aside class="liste" aria-label={t("ziele-systeme")}>
    <h3>{t("ziele-systeme")}</h3>
    <ul role="list">{#each ziele as z (z.system)}<li class="sys" class:on={z.system === zl.sel}><button type="button" class="nm" aria-current={z.system === zl.sel ? "true" : undefined} data-zl="system" data-s={z.system} onclick={() => (zl.sel = z.system)}>{nameVon(z.system)}</button></li>{/each}</ul>
    {#if frei.length}<div class="row"><select class="inp" data-zl="neues-system" aria-label={t("ziele-system-hinzu")} bind:value={neuesSystem}><option value="">{t("ziele-system-hinzu")}</option>{#each frei as x (x.id)}<option value={x.id}>{x.name}</option>{/each}</select><button type="button" class="btn sm" data-zl="system-hinzu" disabled={!neuesSystem} onclick={() => { void zielOp({ kind: "add_target", system: neuesSystem }); neuesSystem = ""; }}>{t("ziele-hinzu")}</button></div>{/if}
  </aside>

  <div class="inhalt">
    {#if !ziel}<p class="dim">{t("ziele-kein-system")}</p>
    {:else}
      <div class="kopf"><h2>{nameVon(ziel.system)}</h2><span class="dim small">{info?.group} · {info?.cpu}</span><button type="button" class="btn sm" data-zl="system-weg" onclick={() => void zielOp({ kind: "remove_target", system: ziel.system })}>{t("ziele-system-weg")}</button></div>
      <div class="card"><h3>{t("ziele-startziel")}</h3>
        <div class="row"><label class="r"><input type="radio" name="start" data-zl="start-emu" checked={!ziel.start_on} onchange={() => { ziel.start_on = ""; }} />{t("ziele-start-emulator")}</label>
          {#each eigene as m (m.id)}<label class="r"><input type="radio" name="start" data-zl="start-maschine" checked={ziel.start_on === m.id} onchange={() => { ziel.start_on = m.id; }} />{m.name || m.id}</label>{/each}</div>
        <p class="small dim">{t("ziele-start-hinweis")}</p></div>

      <h3>{t("ziele-emulatoren")}</h3>
      {#each ziel.emulators as e (e.id)}
        <div class="card" data-zl="emu" data-id={e.id}>
          <div class="row"><input class="inp nm" aria-label={t("ziele-name")} bind:value={e.name} /><select class="inp art" aria-label={t("ziele-art")} bind:value={e.kind} onchange={() => void pruefen(e)}>{#each info?.kinds ?? [] as k (k.id)}<option value={k.id}>{k.label}</option>{/each}</select>
            <label class="r"><input type="radio" name={`std-${ziel.system}`} data-zl="standard" checked={!!e.standard} onchange={() => void zielOp({ kind: "set_standard", system: ziel.system, id: e.id })} />{t("ziele-standard")}</label></div>
          <label class="small dim">{t("ziele-pfad")}<span class="row"><input class="inp" data-zl="pfad" placeholder={t("ziele-pfad-auto")} bind:value={e.path} onchange={() => void pruefen(e)} /><button type="button" class="btn sm" data-zl="suchen" onclick={() => void emuSuchen(e)}>{t("ziele-suchen")}</button><button type="button" class="btn sm" data-zl="testen" disabled={zl.proben[e.id]?.busy} onclick={() => void emuTesten(e)}>{t("ziele-testen")}</button></span></label>
          {#if status(e)}{@const st = status(e)!}<p class="small" class:ok={st.found} class:warn={!st.found} data-zl="anwesend" role="status">{st.found ? "✓ " : "✗ "}{st.auto ? t("ziele-gefunden-auto", { p: kurz(st.text) }) : st.text}</p>{/if}
          <EmuInstall kind={e.kind} onfertig={() => void pruefen(e)} />
          {#if hat(e.kind)}<label class="small dim">{t("ziele-rom")}<input class="inp" data-zl="rom" bind:value={e.rom} /></label>{/if}
          <ArgsEditor kind={e.kind} owner={e} ziel={ziel.system} titel={t("ziele-args-titel")} />
          {#if zl.proben[e.id]}<p class="small" class:ok={zl.proben[e.id]!.ok} class:warn={!zl.proben[e.id]!.ok} data-zl="probe" role="status">{zl.proben[e.id]!.text}</p>{/if}
          <div class="row"><button type="button" class="btn sm" data-zl="emu-weg" onclick={() => void zielOp({ kind: "remove_emulator", system: ziel.system, id: e.id })}>{t("ziele-entfernen")}</button></div>
        </div>
      {:else}<p class="small dim">{t("ziele-keine-emulatoren")}</p>{/each}
      <div class="row"><select class="inp art" data-zl="neue-art" aria-label={t("ziele-art")} bind:value={neueArt}><option value="">{t("ziele-emu-hinzu")}</option>{#each info?.kinds ?? [] as k (k.id)}<option value={k.id}>{k.label}</option>{/each}</select><button type="button" class="btn sm" data-zl="emu-hinzu" disabled={!neueArt} onclick={() => { void zielOp({ kind: "add_emulator", system: ziel.system, ekind: neueArt }); neueArt = ""; }}>{t("ziele-hinzu")}</button></div>

      {#if info?.hardware.length}
        <h3>{t("ziele-hardware")}</h3>
        {#each eigene as m (m.id)}
          <div class="card" data-zl="maschine" data-id={m.id}>
            <div class="row"><input class="inp nm" aria-label={t("ziele-name")} bind:value={m.name} /><span class="dim small">{info.hardware.find((h) => h.id === m.kind)?.label ?? m.kind}</span></div>
            {#if m.kind === "ultimate64"}
              <label class="small dim">{t("ziele-host")}<input class="inp" data-zl="host" placeholder="192.168.1.50" bind:value={m.host} /></label>
              <label class="small dim">{t("ziele-passwort")}<span class="row"><input class="inp" data-zl="passwort" type={zl.zeigen[m.id] ? "text" : "password"} bind:value={m.password} autocomplete="off" /><button type="button" class="btn sm" aria-pressed={!!zl.zeigen[m.id]} data-zl="pw-zeigen" onclick={() => (zl.zeigen[m.id] = !zl.zeigen[m.id])}>{zl.zeigen[m.id] ? t("ziele-pw-verbergen") : t("ziele-pw-zeigen")}</button></span></label>
              <Switch bind:checked={m.reset_before} label={t("ziele-reset-vor")} />
            {:else}
              <label class="small dim">{t("ziele-verbindung")}<select class="inp" data-zl="verbindung" bind:value={m.link}><option value="">{t("ziele-usb")}</option><option value="eth">{t("ziele-ethernet")}</option></select></label>
              <label class="small dim">{t("ziele-geraet")}<span class="row"><input class="inp" data-zl="geraet" list="ports" bind:value={m.device} /><datalist id="ports">{#each zl.ports as p (p)}<option value={p}></option>{/each}</datalist></span></label>
              <label class="small dim">{t("ziele-baud")}<input class="inp" bind:value={m.baud} placeholder="2000000" /></label>
              {#if m.link === "eth"}<p class="small warn" data-zl="eth-hinweis">{t("ziele-eth-ungeprueft")}</p>
                <label class="small dim">{t("ziele-schnittstelle")}<input class="inp" bind:value={m.eth_iface} placeholder="en0" /></label>
                <label class="small dim">etherload<input class="inp" bind:value={m.etherload} /></label><label class="small dim">mega65_ftp<input class="inp" bind:value={m.ftp_tool} /></label><label class="small dim">{t("ziele-sd-ordner")}<input class="inp" bind:value={m.sd_dir} /></label>{/if}
            {/if}
            <Switch bind:checked={m.run_after} label={t("ziele-run-nach")} />
            {#if zl.proben["hw:" + m.id]}<p class="small" class:ok={zl.proben["hw:" + m.id]!.ok} class:warn={!zl.proben["hw:" + m.id]!.ok} data-zl="hw-probe" role="status">{zl.proben["hw:" + m.id]!.text}</p>{/if}
            <div class="row"><button type="button" class="btn sm" data-zl="hw-testen" disabled={zl.proben["hw:" + m.id]?.busy} onclick={() => void maschineTesten(m, m.kind === "mega65" ? (m.link || "usb") : undefined)}>{t("ziele-verbindung-testen")}</button><button type="button" class="btn sm" data-zl="maschine-weg" onclick={() => void zielOp({ kind: "remove_machine", id: m.id })}>{t("ziele-entfernen")}</button></div>
          </div>
        {/each}
        <div class="row">{#each info.hardware as h (h.id)}<button type="button" class="btn sm" data-zl="hw-hinzu" data-h={h.id} onclick={() => void zielOp({ kind: "add_machine", system: ziel.system, mkind: h.id })}>{t("ziele-hw-hinzu", { name: h.label })}</button>{/each}</div>
      {/if}
    {/if}
  </div>
</div>

<style>
  .zl { display: grid; grid-template-columns: 200px minmax(0, 1fr); gap: 14px; } @media (max-width: 900px) { .zl { grid-template-columns: 1fr; } }
  h2 { margin: 0; font-size: 16px; } h3 { margin: 4px 0 4px; font-size: 11px; letter-spacing: 0.07em; text-transform: uppercase; color: var(--dim); }
  .liste ul { list-style: none; margin: 0 0 8px; padding: 0; display: flex; flex-direction: column; gap: 2px; } .sys .nm { width: 100%; text-align: left; padding: 6px 10px; border-radius: 8px; color: var(--text); } .sys .nm:hover { background: var(--hover); } .sys.on .nm { background: var(--accentbg); color: var(--accent2); }
  .inhalt { display: flex; flex-direction: column; gap: 10px; min-width: 0; } .kopf { display: flex; align-items: baseline; gap: 10px; flex-wrap: wrap; } .kopf .btn { margin-left: auto; }
  .card { background: var(--raise); border-radius: 10px; padding: 12px; display: flex; flex-direction: column; gap: 8px; max-width: 720px; } .card p { margin: 0; } label { display: flex; flex-direction: column; gap: 4px; }
  .row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; } .row .inp { flex: 1; min-width: 160px; } .inp.nm { flex: 1; } .inp.art { flex: 0 1 220px; } label.r { flex-direction: row; align-items: center; gap: 6px; font-size: 13px; }
  .small { font-size: 12px; } .dim { color: var(--dim); } .warn { color: var(--warn); } .ok { color: var(--ok); } .inp { width: 100%; }
</style>
