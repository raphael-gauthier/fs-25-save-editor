import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { SaleItem } from "@/lib/types";

export interface SaleChangePayload {
  index: number;
  delete: boolean;
  price?: number;
  damage?: number;
  wear?: number;
  age?: number;
  operatingTime?: number;
  timeLeft?: number;
}

export const useSaleStore = defineStore("sale", () => {
  const items = ref<SaleItem[]>([]);
  const originalItems = ref<SaleItem[]>([]);
  const deletedIndices = ref<Set<number>>(new Set());

  const isDirty = computed(() => {
    if (deletedIndices.value.size > 0) return true;
    if (items.value.length !== originalItems.value.length) return true;
    return items.value.some((item) => {
      const orig = originalItems.value.find((o) => o.index === item.index);
      if (!orig) return true;
      return isItemModified(item, orig);
    });
  });

  const changeCount = computed(() => {
    let count = deletedIndices.value.size;
    const origMap = new Map(originalItems.value.map((i) => [i.index, i]));
    for (const item of items.value) {
      const orig = origMap.get(item.index);
      if (!orig) {
        count++;
        continue;
      }
      if (item.price !== orig.price) count++;
      if (Math.abs(item.damage - orig.damage) > 0.0001) count++;
      if (Math.abs(item.wear - orig.wear) > 0.0001) count++;
      if (item.age !== orig.age) count++;
      if (Math.abs(item.operatingTime - orig.operatingTime) > 0.001) count++;
      if (item.timeLeft !== orig.timeLeft) count++;
    }
    return count;
  });

  function getOriginalItem(index: number): SaleItem | undefined {
    return originalItems.value.find((i) => i.index === index);
  }

  function hydrate(data: SaleItem[]) {
    const serialized = JSON.stringify(data);
    items.value = JSON.parse(serialized);
    originalItems.value = JSON.parse(serialized);
    deletedIndices.value = new Set();
  }

  function updateItem(index: number, changes: Partial<SaleItem>) {
    const item = items.value.find((i) => i.index === index);
    if (item) {
      Object.assign(item, changes);
    }
  }

  function resetToNew(index: number) {
    const item = items.value.find((i) => i.index === index);
    if (item) {
      item.wear = 0;
      item.damage = 0;
      item.age = 0;
      item.operatingTime = 0;
    }
  }

  function setDiscountPrice(index: number) {
    const item = items.value.find((i) => i.index === index);
    if (item) {
      item.price = Math.round(item.price * 0.1);
    }
  }

  function extendSale(index: number, days: number = 30) {
    const item = items.value.find((i) => i.index === index);
    if (item) {
      item.timeLeft += days;
    }
  }

  function deleteItem(index: number) {
    deletedIndices.value.add(index);
    deletedIndices.value = new Set(deletedIndices.value);
    items.value = items.value.filter((i) => i.index !== index);
  }

  function resetChanges() {
    items.value = JSON.parse(JSON.stringify(originalItems.value));
    deletedIndices.value = new Set();
  }

  function getChanges(): SaleChangePayload[] | null {
    const changes: SaleChangePayload[] = [];

    for (const idx of deletedIndices.value) {
      changes.push({ index: idx, delete: true });
    }

    const origMap = new Map(originalItems.value.map((i) => [i.index, i]));
    for (const item of items.value) {
      const orig = origMap.get(item.index);
      if (!orig || !isItemModified(item, orig)) continue;

      const change: SaleChangePayload = { index: item.index, delete: false };
      if (item.price !== orig.price) change.price = item.price;
      if (Math.abs(item.damage - orig.damage) > 0.0001) change.damage = item.damage;
      if (Math.abs(item.wear - orig.wear) > 0.0001) change.wear = item.wear;
      if (item.age !== orig.age) change.age = item.age;
      if (Math.abs(item.operatingTime - orig.operatingTime) > 0.001) change.operatingTime = item.operatingTime;
      if (item.timeLeft !== orig.timeLeft) change.timeLeft = item.timeLeft;

      changes.push(change);
    }

    return changes.length > 0 ? changes : null;
  }

  function commitChanges() {
    originalItems.value = JSON.parse(JSON.stringify(items.value));
    deletedIndices.value = new Set();
  }

  return {
    items,
    isDirty,
    changeCount,
    getOriginalItem,
    hydrate,
    updateItem,
    resetToNew,
    setDiscountPrice,
    extendSale,
    deleteItem,
    resetChanges,
    getChanges,
    commitChanges,
  };
});

function isItemModified(item: SaleItem, orig: SaleItem): boolean {
  return (
    item.price !== orig.price ||
    Math.abs(item.damage - orig.damage) > 0.0001 ||
    Math.abs(item.wear - orig.wear) > 0.0001 ||
    item.age !== orig.age ||
    Math.abs(item.operatingTime - orig.operatingTime) > 0.001 ||
    item.timeLeft !== orig.timeLeft
  );
}
