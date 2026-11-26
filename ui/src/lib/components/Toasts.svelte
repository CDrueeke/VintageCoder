<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { dismissToast, ui } from "../state.svelte";
  import { t } from "../i18n.svelte";
  import Icon from "./Icon.svelte";
</script>

<div class="toasts" role="status" aria-live="polite">
  {#each ui.toasts as m (m.id)}
    <div class="toast" class:err={m.error}>
      <span>{m.text}</span>
      <button type="button" class="ibtn" aria-label={t("toast-close")} onclick={() => dismissToast(m.id)}><Icon name="x" size={14} /></button>
    </div>
  {/each}
</div>
