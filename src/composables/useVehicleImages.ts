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

const CHUNK_SIZE = 5;

export function useVehicleImages() {
  async function loadBatch(gamePath: string, filenames: string[]) {
    // Filter out already cached filenames
    const uncached = filenames.filter((f) => !imageCache.value.has(f));
    if (uncached.length === 0) return;

    pendingBatch.value = true;

    // Yield to browser so Vue can mount components and paint skeletons
    await new Promise((r) => setTimeout(r, 0));

    // Split into chunks and load progressively
    for (let i = 0; i < uncached.length; i += CHUNK_SIZE) {
      const chunk = uncached.slice(i, i + CHUNK_SIZE);
      try {
        const results = await invoke<VehicleImageResult[]>(
          "get_vehicle_images_batch",
          {
            gamePath,
            vehicleFilenames: chunk,
          },
        );
        for (const r of results) {
          imageCache.value.set(
            r.filename,
            r.imagePath ? convertFileSrc(r.imagePath) : null,
          );
        }
      } catch {
        // On error, mark chunk as null to avoid retrying
        for (const f of chunk) {
          imageCache.value.set(f, null);
        }
      }
      // Force Vue to detect Map mutations and let browser paint between chunks
      triggerRef(imageCache);
      await new Promise((r) => setTimeout(r, 0));
    }

    pendingBatch.value = false;
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
