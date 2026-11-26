<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  let { checked = $bindable(false), label, onchange }: { checked?: boolean; label: string; onchange?: (on: boolean) => void } = $props();
</script>

<label class="row" style="display:inline-flex;align-items:center;gap:10px">
  <button type="button" role="switch" class="sw" aria-checked={checked} aria-label={label} onclick={() => { checked = !checked; onchange?.(checked); }}></button>
  <span>{label}</span>
</label>
