<script lang="ts">
  import { AlertTriangle } from "@lucide/svelte";
  import MessageText from "./MessageText.svelte";

  type FormErrorPropsType = { message: string; id?: string };
  let { message, id }: FormErrorPropsType = $props();
</script>

{#if message}
  <p {id} class="steamguard-error form-error" role="alert">
    <AlertTriangle size={15} />
    <span><MessageText content={message} /></span>
  </p>
{/if}
