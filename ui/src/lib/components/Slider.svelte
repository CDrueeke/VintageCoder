<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  let { value = $bindable(0), min = 0, max = 100, step = 1, label }: { value?: number; min?: number; max?: number; step?: number; label: string } = $props();
</script>

<label style="display:inline-flex;align-items:center;gap:10px">
  <span>{label}</span>
  <input type="range" {min} {max} {step} bind:value aria-valuetext={String(value)} />
  <span class="mono" aria-hidden="true" style="min-width:3ch;color:var(--dim)">{value}</span>
</label>
