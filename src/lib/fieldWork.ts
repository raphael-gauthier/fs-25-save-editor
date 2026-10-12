import { GROUND_TYPES } from "@/lib/constants";
import type { DensityEditPayload } from "@/lib/types";

/// Field work the editor offers as one-click presets.
export type SoilWork = "plow" | "cultivate" | "roll" | "mulch";

/** Value of a ground type in the ground density map (GROUND_TYPES omits NONE = 0). */
export function groundValue(name: (typeof GROUND_TYPES)[number]): number {
  return GROUND_TYPES.indexOf(name) + 1;
}

/**
 * Density edit growing `fruit` at growth `stage`, or clearing the crop when
 * `fruit` is "NONE". It replaces earlier crop actions on the field, and the
 * backend sets the ground the stage shows.
 */
export function cropEdit(fruit: string, stage: number): Partial<DensityEditPayload> {
  const crop: Partial<DensityEditPayload> =
    fruit === "NONE"
      ? { setFruitName: "NONE", setGrowthState: undefined }
      : { setFruitName: fruit, setGrowthState: stage, setGroundType: undefined };
  return { ...crop, setHarvestReady: false };
}

/** Density edit doing `work` on a field. */
export function soilWorkEdit(work: SoilWork): Partial<DensityEditPayload> {
  // Plowing and cultivating turn the crop and its weeds under
  const turnUnder = { ...cropEdit("NONE", 0), clearWeeds: true };
  switch (work) {
    case "plow":
      return { ...turnUnder, setGroundType: groundValue("PLOWED"), setPlowLevel: 1 };
    case "cultivate":
      return { ...turnUnder, setGroundType: groundValue("CULTIVATED") };
    case "roll":
      // Roller level 1 means "needs rolling" (MapOverlayGenerator.lua)
      return { setRollerLevel: 0 };
    case "mulch":
      return { setStubbleShredLevel: 1 };
  }
}
