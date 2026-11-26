<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { untrack } from "svelte";
  import { ui } from "../state.svelte";
  import { sess, scratchAnswer } from "../sitzung.svelte";
  import { artOfName, closeDecision, ed, leaveDecision, tabName } from "../editor.svelte";
  import { t } from "../i18n.svelte";
  import Dialog from "./Dialog.svelte";
  import BasicConvertDialog from "./BasicConvertDialog.svelte";
  // Zwei Dialoge des Editors: Name eingeben (Neu, Umbenennen, Löschen-Nachfrage) und „Ungespeicherte Änderungen“ beim Schließen eines Reiters
  let nameOpen = $state(false);
  let value = $state("");
  let err = $state<string | null>(null);
  let art = $state("asm");
  let input = $state<HTMLInputElement | undefined>();
  $effect(() => { if (ed.prompt) { value = ed.prompt.value; art = ed.prompt.art ?? "asm"; err = null; nameOpen = true; queueMicrotask(() => { input?.focus(); input?.select(); }); } });
  $effect(() => { if (!nameOpen && ed.prompt) { const p = ed.prompt; ed.prompt = null; p.oncancel?.(); } });
  // Gibt man selbst eine Endung ein, passt sich die Art an
  // (nur der Name löst es aus, nicht die Auswahl selbst; sonst ließe sich die Art nie von Hand ändern)
  $effect(() => { const v = value; untrack(() => { if (ed.prompt?.arts) { const a = artOfName(v); if (a && a !== art && ed.prompt.arts.some((x) => x.id === a)) art = a; } }); });
  async function ok() {
    const p = ed.prompt; if (!p) return;
    if (p.validate) { err = p.validate(value); if (err) return; }
    nameOpen = false; ed.prompt = null; await p.onok(value, art);
  }
  const closeOpen = $derived(ed.closing !== null);
  const leaveOpen = $derived(ed.leaving !== null);
</script>

{#if ed.prompt}
  <Dialog bind:open={nameOpen} title={ed.prompt.title}>
    {#snippet children()}
      <form onsubmit={(e) => { e.preventDefault(); void ok(); }} id="namefrm">
        {#if ed.prompt?.validate || ed.prompt?.value !== ""}
          <label for="nameinp" style="display:block;margin-bottom:6px;color:var(--dim)">{ed.prompt!.label}</label>
          <input id="nameinp" bind:this={input} bind:value class="inp" autocomplete="off" spellcheck="false" aria-invalid={err ? "true" : undefined} aria-describedby={err ? "nameerr" : undefined} />
          {#if ed.prompt?.arts}<label for="nameart" style="display:block;margin:10px 0 6px;color:var(--dim)">{t("tree-art-label")}</label>
            <select id="nameart" bind:value={art} class="inp">{#each ed.prompt.arts as a (a.id)}<option value={a.id}>{a.label}</option>{/each}</select>{/if}
          {#if err}<p id="nameerr" role="alert" style="color:var(--err);margin-top:6px">{err}</p>{/if}
        {:else}<p>{ed.prompt.label}</p>{/if}
      </form>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => (nameOpen = false)}>{t("dialog-abbrechen")}</button>
      <button type="submit" form="namefrm" class="btn pri">{ed.prompt!.ok}</button>
    {/snippet}
  </Dialog>
{/if}

<Dialog open={closeOpen} title={t("close-title")}>
  {#snippet children()}<p>{t("close-frage", { name: ed.closing ? tabName(ed.closing) : "" })}</p>{/snippet}
  {#snippet footer()}
    <button type="button" class="btn" onclick={() => closeDecision("cancel")}>{t("dialog-abbrechen")}</button>
    <button type="button" class="btn" onclick={() => closeDecision("discard")}>{t("close-verwerfen")}</button>
    <button type="button" class="btn pri" onclick={() => closeDecision("save")}>{t("close-speichern")}</button>
  {/snippet}
</Dialog>

<Dialog open={leaveOpen} title={t("leave-title")}>
  {#snippet children()}
    <p>{t("leave-frage", { n: ed.leaving?.items.length ?? 0 })}</p>
    <ul class="leave" aria-label={t("leave-title")}>{#each ed.leaving?.items ?? [] as it (it.name)}<li>{it.name}{#if it.scratch}&nbsp;<span class="dim">{t("leave-unbenannt")}</span>{/if}</li>{/each}</ul>
  {/snippet}
  {#snippet footer()}
    <button type="button" class="btn" onclick={() => leaveDecision("cancel")}>{t("dialog-abbrechen")}</button>
    <button type="button" class="btn" onclick={() => leaveDecision("discard")}>{t("close-verwerfen")}</button>
    <button type="button" class="btn pri" onclick={() => leaveDecision("save")}>{t("leave-alle-speichern")}</button>
  {/snippet}
</Dialog>

<Dialog open={sess.offer !== null && !!ui.project} title={t("scratch-title")}>
  {#snippet children()}
    <p>{t("scratch-frage", { n: sess.offer?.length ?? 0 })}</p>
    <ul class="leave">{#each sess.offer ?? [] as e (e.name)}<li>{e.name} <span class="dim">{t("scratch-zeichen", { n: e.text.length })}</span></li>{/each}</ul>
    <p class="dim" style="font-size:12px">{t("scratch-hinweis")}</p>
  {/snippet}
  {#snippet footer()}
    <button type="button" class="btn" onclick={() => void scratchAnswer(false)}>{t("close-verwerfen")}</button>
    <button type="button" class="btn pri" onclick={() => void scratchAnswer(true)}>{t("scratch-wiederherstellen")}</button>
  {/snippet}
</Dialog>


<BasicConvertDialog />

<style>
  .leave { margin: 6px 0 0; padding-left: 20px; max-height: 220px; overflow: auto; }
  .leave .dim { color: var(--dim); font-size: 12px; }
  .inp { width: 100%; height: 32px; padding: 0 10px; border-radius: var(--r-ctl); border: 1px solid var(--line2); background: var(--raise); color: var(--text); font: inherit; }
  .inp:focus-visible { box-shadow: var(--focus); outline: none; }
</style>
