<script lang="ts">
  import { onMount } from "svelte";

  import { wireMenuDismissal } from "../lib/menus.svelte";
  import { app } from "../lib/state.svelte";
  import AccountDialog from "./AccountDialog.svelte";
  import Breadcrumb from "./Breadcrumb.svelte";
  import BusyOverlay from "./BusyOverlay.svelte";
  import CardGrid from "./CardGrid.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import Dropdown from "./Dropdown.svelte";
  import FolderDialog from "./FolderDialog.svelte";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import ReportDialog from "./ReportDialog.svelte";
  import SettingsDialog from "./SettingsDialog.svelte";
  import Toast from "./Toast.svelte";
  import UpdateDialog from "./UpdateDialog.svelte";

  onMount(wireMenuDismissal);
</script>

<div class="app" class:idle={app.idle}>
  <Header />
  <Breadcrumb />
  <main class="content">
    <CardGrid />
  </main>
  <Footer />

  <AccountDialog />
  <ReportDialog />
  <SettingsDialog />
  <UpdateDialog />
  <ConfirmDialog />
  <FolderDialog />
  <ContextMenu />
  <Dropdown />
  <BusyOverlay />
  <Toast />
</div>
