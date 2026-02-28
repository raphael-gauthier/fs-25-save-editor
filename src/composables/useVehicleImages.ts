import { ref, triggerRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

interface VehicleImageResult {
  filename: string;
  imagePath: string | null;
}

// Global reactive cache: using a ref wrapping a Map so Vue can track updates
const imageCache = ref(new Map<string, string | null>());
const pendingBatch = ref(false);

export function useVehicleImages() {
  async function loadBatch(gamePath: string, filenames: string[]) {
    // Filter out already cached filenames
    const uncached = filenames.filter((f) => !imageCache.value.has(f));
    if (uncached.length === 0) return;

    pendingBatch.value = true;
    try {
      const results = await invoke<VehicleImageResult[]>(
        "get_vehicle_images_batch",
        {
          gamePath,
          vehicleFilenames: uncached,
        },
      );
      for (const r of results) {
        imageCache.value.set(
          r.filename,
          r.imagePath ? convertFileSrc(r.imagePath) : null,
        );
      }
      // Force Vue to detect Map mutations
      triggerRef(imageCache);
    } catch {
      // On error, mark all as null to avoid retrying
      for (const f of uncached) {
        imageCache.value.set(f, null);
      }
      triggerRef(imageCache);
    } finally {
      pendingBatch.value = false;
    }
  }

  function getImageUrl(filename: string): string | null {
    return imageCache.value.get(filename) ?? null;
  }

  async function detectGamePath(): Promise<string | null> {
    return invoke<string | null>("detect_game_path");
  }

  async function clearDiskCache(): Promise<number> {
    const bytes = await invoke<number>("clear_image_cache");
    imageCache.value.clear();
    triggerRef(imageCache);
    return bytes;
  }

  async function getCacheSize(): Promise<number> {
    return invoke<number>("get_image_cache_size");
  }

  return {
    pendingBatch,
    imageCache,
    loadBatch,
    getImageUrl,
    detectGamePath,
    clearDiskCache,
    getCacheSize,
  };
}
