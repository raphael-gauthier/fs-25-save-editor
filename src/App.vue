<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";
import { Toaster } from "@/components/ui/sonner";
import { useSettingsStore } from "@/stores/settings";
import { useUnsavedChanges } from "@/composables/useUnsavedChanges";
import ConfirmDiscardDialog from "@/components/ConfirmDiscardDialog.vue";
import DisclaimerDialog from "@/components/DisclaimerDialog.vue";

const settings = useSettingsStore();
const router = useRouter();
const { isDirty, confirmDiscardIfDirty } = useUnsavedChanges();

// Navigation guard: confirm discard when leaving editor to home
router.beforeEach(async (to, from) => {
  if (from.path.startsWith("/editor") && to.path === "/") {
    const confirmed = await confirmDiscardIfDirty();
    if (!confirmed) return false;
  }
  return true;
});

// Window close guard
function onBeforeUnload(e: BeforeUnloadEvent) {
  if (isDirty.value) {
    e.preventDefault();
    e.returnValue = "";
  }
}

onMounted(() => {
  settings.loadSettings();
  window.addEventListener("beforeunload", onBeforeUnload);
});

onUnmounted(() => {
  window.removeEventListener("beforeunload", onBeforeUnload);
});
</script>

<template>
  <router-view />
  <DisclaimerDialog />
  <ConfirmDiscardDialog />
  <Toaster rich-colors />
</template>
