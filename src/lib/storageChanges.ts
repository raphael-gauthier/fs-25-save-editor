import type {
  ObjectStorageChangePayload,
  SiloStockChangePayload,
  SiloStorage,
  StoredObject,
} from "@/lib/types";

/** Identical stored objects: same kind, model, content, fill level and wrapping. */
export interface ObjectGroup {
  key: string;
  /** A representative of the group */
  object: StoredObject;
  /** Positions of the group's objects in the storage, in file order */
  indices: number[];
}

export function objectKey(o: StoredObject): string {
  return [o.className, o.filename, o.fillType, o.fillLevel, o.wrappingState].join("|");
}

/** Groups identical objects, in order of first appearance. */
export function groupObjects(objects: StoredObject[]): ObjectGroup[] {
  const groups = new Map<string, ObjectGroup>();
  objects.forEach((object, index) => {
    const key = objectKey(object);
    const group = groups.get(key);
    if (group) group.indices.push(index);
    else groups.set(key, { key, object, indices: [index] });
  });
  return [...groups.values()];
}

/**
 * Object storage change turning `original` into `current`, where `current`
 * only added copies of original objects or removed some: groups that grew copy
 * their first original object, groups that shrank drop their last ones.
 */
export function objectStorageChange(
  current: StoredObject[],
  original: StoredObject[],
  finishFermentation: boolean,
): ObjectStorageChangePayload | null {
  const counts = new Map<string, number>();
  for (const object of current) {
    const key = objectKey(object);
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  const change: ObjectStorageChangePayload = { remove: [], copies: [], finishFermentation };
  for (const group of groupObjects(original)) {
    const delta = (counts.get(group.key) ?? 0) - group.indices.length;
    if (delta > 0) change.copies.push({ index: group.indices[0], count: delta });
    if (delta < 0) change.remove.push(...group.indices.slice(delta));
  }
  const changed = change.remove.length > 0 || change.copies.length > 0 || finishFermentation;
  return changed ? change : null;
}

/** Silo fill levels that differ from `original`, 0 for a fill type that was emptied. */
export function siloStockChanges(current: SiloStorage[], original: SiloStorage[]): SiloStockChangePayload[] {
  const changes: SiloStockChangePayload[] = [];
  for (const storage of current) {
    const before = original.find((o) => o.kind === storage.kind && o.index === storage.index);
    const levels = new Map(storage.stocks.map((s) => [s.fillType, s.amount]));
    const previous = new Map((before?.stocks ?? []).map((s) => [s.fillType, s.amount]));
    for (const fillType of new Set([...levels.keys(), ...previous.keys()])) {
      const amount = levels.get(fillType) ?? 0;
      if (amount !== (previous.get(fillType) ?? 0)) {
        changes.push({ kind: storage.kind, index: storage.index, fillType, amount });
      }
    }
  }
  return changes;
}
