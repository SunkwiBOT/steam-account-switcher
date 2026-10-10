<script lang="ts">
  import { errorMessage, resolveErrorCode } from "../../lib/errors/errors.svelte";

  import { t } from "../../lib/i18n/i18n.svelte";

  import ActionButton from "../ui/ActionButton.svelte";
  import { onMount } from "svelte";
  import { AlertTriangle, FileText, FolderOpen, RefreshCw } from "@lucide/svelte";
  import { invoke } from "../../lib/api/tauri";
  import { showToast } from "../../lib/state/state.svelte";
  import type { LogSnapshotType } from "../../lib/types";
  import CopyButton from "../ui/CopyButton.svelte";

  let log = $state<LogSnapshotType | null>(null);
  let previous = $state(false);
  let loading = $state(false);
  let opening = $state(false);
  let error = $state("");
  let generation = 0;

  const load = async (selectPrevious = previous): Promise<void> => {
    if (loading) return;
    previous = selectPrevious;
    loading = true;
    error = "";
    const current = ++generation;
    try {
      const result = await invoke("logs_snapshot", { previous });
      if (current === generation) log = result;
    } catch (cause) {
      if (current === generation) error = resolveErrorCode(cause);
    } finally {
      if (current === generation) loading = false;
    }
  };

  const openDirectory = async (): Promise<void> => {
    if (opening) return;
    opening = true;
    try {
      await invoke("logs_open_directory");
    } catch (cause) {
      showToast(errorMessage(cause));
    } finally {
      opening = false;
    }
  };

  onMount(() => {
    void load();
    return () => {
      generation += 1;
    };
  });
</script>

<section class="settings-card logs-card">
  <h3 class="settings-card-title"><FileText size={15} /> {t.settings_logs_title()}</h3>
  <p class="settings-note">
    {t.logs_description()}
  </p>
  <div class="logs-path">
    <label class="field-label" for="log-path">{t.logs_file_label()}</label>
    <div class="logs-path-row">
      <input id="log-path" class="input" readonly value={log?.path ?? ""} />
      <CopyButton
        valueLabel={t.logs_file_label()}
        copyKey={`log-path:${log?.path ?? ""}`}
        label={t.logs_path_copy()}
        read={() => log?.path ?? ""}
        disabled={!log}
      />
    </div>
  </div>
  <div class="logs-actions">
    <ActionButton
      icon={FolderOpen}
      label={t.folder_open()}
      busyLabel={t.common_opening()}
      busy={opening}
      disabled={!log}
      className="button"
      onclick={openDirectory}
    />
    <ActionButton
      icon={RefreshCw}
      label={t.common_refresh()}
      busyLabel={t.common_refreshing()}
      busy={loading}
      disabled={false}
      className="button"
      onclick={() => load()}
    />
  </div>
  <div class="logs-selector" aria-label={t.logs_file_selector()}>
    <button
      class="button"
      type="button"
      class:primary={!previous}
      aria-pressed={!previous}
      disabled={loading}
      onclick={() => load(false)}>{t.logs_current()}</button
    >
    <button
      class="button"
      type="button"
      class:primary={previous}
      aria-pressed={previous}
      disabled={loading || !log?.previous_available}
      onclick={() => load(true)}>{t.logs_previous()}</button
    >
  </div>
  {#if error}<p class="steamguard-error" role="alert">
      <AlertTriangle size={15} />
      {errorMessage(error)}
    </p>{/if}
  <textarea
    class="logs-preview"
    readonly
    rows="10"
    aria-label={t.logs_content_label()}
    aria-busy={loading}
    value={log?.content || (loading ? t.logs_loading() : t.logs_empty())}></textarea>
</section>
