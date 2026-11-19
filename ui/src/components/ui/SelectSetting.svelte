<script lang="ts" module>
  export type SelectSettingOptionType = {
    value: string;
    label: string;
  };
</script>

<script lang="ts">
  import { ChevronDown } from "@lucide/svelte";
  import { t } from "../../lib/i18n/i18n.svelte";
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuRadioGroup,
    DropdownMenuRadioItem,
    DropdownMenuTrigger,
  } from "$lib/components/ui/dropdown-menu/index.js";
  import AsyncIcon from "./AsyncIcon.svelte";

  type SelectSettingPropsType = {
    value: string;
    options: SelectSettingOptionType[];
    label: string;
    ariaLabel: string;
    busy?: boolean;
    disabled?: boolean;
    onselect: (value: string) => void;
  };

  let {
    value,
    options,
    label,
    ariaLabel,
    busy = false,
    disabled = busy,
    onselect,
  }: SelectSettingPropsType = $props();
  let open = $state(false);
</script>

<DropdownMenu bind:open>
  <DropdownMenuTrigger class="select-button" aria-label={ariaLabel} aria-busy={busy} {disabled}>
    <span>{busy ? t.common_saving() : label}</span>
    <span class="picker-caret">
      <AsyncIcon icon={ChevronDown} {busy} size={14} />
    </span>
  </DropdownMenuTrigger>
  <DropdownMenuContent>
    <DropdownMenuRadioGroup {value} onValueChange={onselect}>
      {#each options as option (option.value)}
        <DropdownMenuRadioItem value={option.value}>{option.label}</DropdownMenuRadioItem>
      {/each}
    </DropdownMenuRadioGroup>
  </DropdownMenuContent>
</DropdownMenu>
