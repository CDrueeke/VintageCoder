<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  // Emulator portabel installieren (E12.2b): zeigt, was geladen würde (Version, Größe, Quelle, Ziel), und lädt erst nach ausdrücklichem „Jetzt laden“.
  import { call, CoreError, type EmuPakete } from "../api";
  import { errorText, t } from "../i18n.svelte";
  let { kind, onfertig }: { kind: string; onfertig?: (path: string) => void } = $props();
  let info = $state<EmuPakete | null>(null), frage = $state<string | null>(null), laeuft = $state(false), meldung = $state(""), fehler = $state(false);
  const neuLaden = () => { call("emu_pakete", { kind }).then((i) => (info = i)).catch(() => {}); };
  $effect(() => { window.addEventListener("vc-emu-installed", neuLaden); return () => window.removeEventListener("vc-emu-installed", neuLaden); });   // mehrere Karten derselben Sorte bleiben im Takt
  $effect(() => { const k = kind; info = null; frage = null; meldung = ""; call("emu_pakete", { kind: k }).then((i) => { if (k === kind) info = i; }).catch(() => {}); });
  const offen = $derived((info?.packages ?? []).filter((p) => !p.installed));
  async function laden(version: string) {
    frage = null; laeuft = true; meldung = ""; fehler = false;
    try { const p = await call("emu_install", { kind, version }); meldung = t("ziele-installiert", { pfad: p }); onfertig?.(p); window.dispatchEvent(new Event("vc-emu-installed")); }
    catch (e) { fehler = true; meldung = e instanceof CoreError ? errorText(e.code, e.args) : String(e); }
    finally { laeuft = false; }
  }
</script>

{#if info?.known}
  <div class="inst" data-zl="install">
    {#if laeuft}<p class="small" role="status" data-zl="inst-laeuft">{t("ziele-laedt-und-installiert")}</p>
    {:else if frage}
      {@const p = offen.find((x) => x.version === frage)}
      {#if p}<p class="small" data-zl="inst-frage">{t("ziele-laedt-frage", { name: info.name ?? "", version: p.version, mb: p.sizeMb.toFixed(1), host: p.host, nach: p.to ?? "" })}</p>
        <div class="row"><button type="button" class="btn pri sm" data-zl="inst-ja" onclick={() => void laden(p.version)}>{t("ziele-jetzt-laden")}</button><button type="button" class="btn sm" data-zl="inst-nein" onclick={() => (frage = null)}>{t("kopierdlg-abbrechen")}</button></div>{/if}
    {:else if offen.length}
      <button type="button" class="btn sm" data-zl="inst-start" onclick={() => (frage = offen[0]!.version)}>{t("ziele-portabel-installieren", { version: offen[0]!.version })}</button>
    {:else if info.manual && !(info.installed?.length)}<p class="small dim" data-zl="inst-manuell">{info.manual}</p>{/if}
    {#if meldung}<p class="small" class:err={fehler} class:ok={!fehler} role="status" data-zl="inst-meldung">{meldung}</p>{/if}
  </div>
{/if}

<style>
  .inst { margin: 4px 0; } p { margin: 4px 0; } .row { display: flex; gap: 8px; } .small { font-size: 11.5px; } .dim { color: var(--dim); } .ok { color: var(--ok); } .err { color: var(--err); }
</style>
