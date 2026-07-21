import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  AnimalCluster,
  AnimalClusterChangePayload,
  FillStock,
  FillStockChangePayload,
  Husbandry,
  HusbandryChangePayload,
  MeadowStock,
} from "@/lib/types";

type StockKind = "food" | "water" | "meadow";

export const useAnimalStore = defineStore("animal", () => {
  const husbandries = ref<Husbandry[]>([]);
  const originalHusbandries = ref<Husbandry[]>([]);

  const totalAnimals = computed(() =>
    husbandries.value.reduce(
      (sum, h) => sum + h.clusters.reduce((s, c) => s + c.numAnimals, 0),
      0,
    ),
  );

  function findOriginal(placeableIndex: number): Husbandry | undefined {
    return originalHusbandries.value.find((h) => h.placeableIndex === placeableIndex);
  }

  function stocksOf(h: Husbandry, kind: StockKind): (FillStock | MeadowStock)[] {
    if (kind === "food") return h.food;
    if (kind === "water") return h.water;
    return h.meadow;
  }

  /**
   * Clusters are matched by their `index` field rather than array position.
   * Invariant this relies on: the backend parser (`parsers/husbandry.rs`) assigns
   * `index` equal to the array position at push time, and the writer
   * (`writers/husbandry.rs`) re-derives the same counter while walking `<animal>`
   * elements in document order. As long as both sides hold that invariant, keying
   * on `index` and keying on position are equivalent — but keying on `index`
   * doesn't silently break if that ever stops being true.
   */
  function originalCluster(orig: Husbandry, index: number): AnimalCluster | undefined {
    return orig.clusters.find((c) => c.index === index);
  }

  function isHusbandryModified(h: Husbandry, orig: Husbandry): boolean {
    const clusterChanged = h.clusters.some((c) => {
      const o = originalCluster(orig, c.index);
      if (!o) return false;
      return (
        c.numAnimals !== o.numAnimals ||
        c.age !== o.age ||
        c.health !== o.health ||
        c.reproduction !== o.reproduction
      );
    });
    if (clusterChanged) return true;

    // Stocks have no `index` field — only `fillType`, which is not guaranteed unique
    // within a collection — so array position is the only correlation available here.
    // Kept positional deliberately; not the same guarantee as clusters above.

    return (["food", "water", "meadow"] as StockKind[]).some((kind) =>
      stocksOf(h, kind).some((s, i) => s.amount !== stocksOf(orig, kind)[i]?.amount),
    );
  }

  const isDirty = computed(() =>
    husbandries.value.some((h) => {
      const orig = findOriginal(h.placeableIndex);
      return orig ? isHusbandryModified(h, orig) : false;
    }),
  );

  const changeCount = computed(() => {
    let count = 0;
    for (const h of husbandries.value) {
      const orig = findOriginal(h.placeableIndex);
      if (!orig) continue;
      h.clusters.forEach((c) => {
        const o = originalCluster(orig, c.index);
        if (!o) return;
        if (c.numAnimals !== o.numAnimals) count++;
        if (c.age !== o.age) count++;
        if (c.health !== o.health) count++;
        if (c.reproduction !== o.reproduction) count++;
      });
      for (const kind of ["food", "water", "meadow"] as StockKind[]) {
        stocksOf(h, kind).forEach((s, i) => {
          if (s.amount !== stocksOf(orig, kind)[i]?.amount) count++;
        });
      }
    }
    return count;
  });

  function hydrate(data: Husbandry[]) {
    const serialized = JSON.stringify(data ?? []);
    husbandries.value = JSON.parse(serialized);
    originalHusbandries.value = JSON.parse(serialized);
  }

  function updateCluster(
    placeableIndex: number,
    clusterIndex: number,
    field: "numAnimals" | "age" | "health" | "reproduction",
    value: number,
  ) {
    const h = husbandries.value.find((x) => x.placeableIndex === placeableIndex);
    const cluster = h?.clusters.find((c) => c.index === clusterIndex);
    if (cluster) cluster[field] = value;
  }

  function updateStock(
    placeableIndex: number,
    kind: StockKind,
    fillType: string,
    amount: number,
  ) {
    const h = husbandries.value.find((x) => x.placeableIndex === placeableIndex);
    if (!h) return;
    const stock = stocksOf(h, kind).find((s) => s.fillType === fillType);
    if (stock) stock.amount = amount;
  }

  /** Original value of a field, used by the UI to flag suspiciously large edits. */
  function originalClusterValue(
    placeableIndex: number,
    clusterIndex: number,
    field: "numAnimals" | "age" | "health" | "reproduction",
  ): number | undefined {
    return findOriginal(placeableIndex)?.clusters.find((c) => c.index === clusterIndex)?.[field];
  }

  function originalStockValue(
    placeableIndex: number,
    kind: StockKind,
    fillType: string,
  ): number | undefined {
    const orig = findOriginal(placeableIndex);
    if (!orig) return undefined;
    return stocksOf(orig, kind).find((s) => s.fillType === fillType)?.amount;
  }

  function resetChanges() {
    husbandries.value = JSON.parse(JSON.stringify(originalHusbandries.value));
  }

  function getChanges(): HusbandryChangePayload[] | null {
    const changes: HusbandryChangePayload[] = [];

    for (const h of husbandries.value) {
      const orig = findOriginal(h.placeableIndex);
      if (!orig || !isHusbandryModified(h, orig)) continue;

      const change: HusbandryChangePayload = { placeableIndex: h.placeableIndex };

      const clusters: AnimalClusterChangePayload[] = [];
      h.clusters.forEach((c) => {
        const o = originalCluster(orig, c.index);
        if (!o) return;
        const entry: AnimalClusterChangePayload = { index: c.index };
        let touched = false;
        if (c.numAnimals !== o.numAnimals) { entry.numAnimals = c.numAnimals; touched = true; }
        if (c.age !== o.age) { entry.age = c.age; touched = true; }
        if (c.health !== o.health) { entry.health = c.health; touched = true; }
        if (c.reproduction !== o.reproduction) { entry.reproduction = c.reproduction; touched = true; }
        if (touched) clusters.push(entry);
      });
      if (clusters.length > 0) change.clusters = clusters;

      for (const kind of ["food", "water", "meadow"] as StockKind[]) {
        const dirty: FillStockChangePayload[] = stocksOf(h, kind)
          .filter((s, i) => s.amount !== stocksOf(orig, kind)[i]?.amount)
          .map((s) => ({ fillType: s.fillType, amount: s.amount }));
        if (dirty.length > 0) change[kind] = dirty;
      }

      changes.push(change);
    }

    return changes.length > 0 ? changes : null;
  }

  function commitChanges() {
    originalHusbandries.value = JSON.parse(JSON.stringify(husbandries.value));
  }

  return {
    husbandries,
    totalAnimals,
    isDirty,
    changeCount,
    hydrate,
    updateCluster,
    updateStock,
    originalClusterValue,
    originalStockValue,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
