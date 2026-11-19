<script lang="ts">
  import { app } from "../../lib/state/state.svelte";
  import AccountDialog from "../accounts/AccountDialog.svelte";
  import Breadcrumb from "./Breadcrumb.svelte";
  import BusyOverlay from "./BusyOverlay.svelte";
  import BackupDialog from "../vault/BackupDialog.svelte";
  import CardGrid from "../ui/CardGrid.svelte";
  import ConfirmDialog from "../dialogs/ConfirmDialog.svelte";
  import FolderDialog from "../folders/FolderDialog.svelte";
  import FolderEditorDialog from "../folders/FolderEditorDialog.svelte";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import ReportDialog from "../playtime/ReportDialog.svelte";
  import SettingsDialog from "../settings/SettingsDialog.svelte";
  import SteamGuardDialog from "../steam-guard/SteamGuardDialog.svelte";
  import VaultDialog from "../vault/VaultDialog.svelte";
  import VaultPasswordOfferDialog from "../vault/VaultPasswordOfferDialog.svelte";
  import Toasts from "./Toasts.svelte";
  import UpdateDialog from "../updates/UpdateDialog.svelte";
</script>

<div class="app" class:idle={app.idle}>
  <div class="app-content" inert={app.busy.active || Boolean(app.switchingId)}>
    <Header />
    <Breadcrumb />
    <main class="content">
      <CardGrid />
    </main>
    <Footer />

    <AccountDialog />
    <SteamGuardDialog />
    <ReportDialog />
    <SettingsDialog />
    <UpdateDialog />
    <VaultDialog />
    <VaultPasswordOfferDialog />
    <BackupDialog />
    <ConfirmDialog />
    <FolderDialog />
    <FolderEditorDialog />
  </div>
  <BusyOverlay />
  <Toasts />
</div>
