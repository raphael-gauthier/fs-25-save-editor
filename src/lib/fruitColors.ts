/// Swatch colors for crops in lists: a visual cue only, the crop name is
/// always written next to it.
const FRUIT_COLORS: Record<string, string> = {
  WHEAT: "#d8b24c",
  BARLEY: "#c9a458",
  OAT: "#cdbb7a",
  CANOLA: "#d6c23a",
  SUNFLOWER: "#e39a2b",
  SOYBEAN: "#8fa448",
  MAIZE: "#e0b53a",
  POTATO: "#b08a4a",
  SUGARBEET: "#b5546a",
  SUGARCANE: "#7fae5a",
  COTTON: "#b9b4a8",
  SORGHUM: "#a8543a",
  RICE: "#9fc06a",
  RICELONGGRAIN: "#a9c97a",
  ONION: "#c98a52",
  CARROT: "#d9772b",
  PARSNIP: "#d8c79a",
  BEETROOT: "#8e2f4f",
  GRASS: "#5fa05a",
  MEADOW: "#6fae66",
  SPINACH: "#3f8a4a",
  GREENBEAN: "#4e9a4a",
  PEA: "#7cb85a",
  OILSEEDRADISH: "#9a8fc4",
  POPLAR: "#6f8f5e",
  GRAPE: "#7a4a8e",
  OLIVE: "#7f8a3e",
};

/** A stable color for any fruit name, map mods included. */
export function fruitColor(name: string | null | undefined): string {
  if (!name) return "var(--muted)";
  const known = FRUIT_COLORS[name.toUpperCase()];
  if (known) return known;
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + ch.charCodeAt(0)) | 0;
  return `hsl(${Math.abs(hash) % 360} 45% 52%)`;
}
