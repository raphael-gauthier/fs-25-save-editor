import type { ClassValue } from "clsx"
import { clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatMoney(amount: number): string {
  return new Intl.NumberFormat("fr-FR", {
    style: "decimal",
    maximumFractionDigits: 0,
  }).format(Math.round(amount))
}

export function formatPlayTime(totalMinutes: number): string {
  const hours = Math.floor(totalMinutes / 60)
  const minutes = Math.floor(totalMinutes % 60)
  if (hours > 0) {
    return `${hours}h ${minutes.toString().padStart(2, "0")}min`
  }
  return `${minutes}min`
}

/// Infers a readable name from the vehicle XML filename.
export function vehicleDisplayName(filename: string): string {
  const parts = filename.replace(/\\/g, "/").split("/")
  const last = parts[parts.length - 1] ?? filename
  const name = last.replace(/\.xml$/i, "")
  return name
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
    .replace(/[_-]/g, " ")
    .trim()
}

/// Infers the vehicle type key from the XML filename.
/// Returns an i18n-compatible key like "tractors", "harvesters", etc.
export function vehicleType(filename: string): string {
  const parts = filename.replace(/\\/g, "/").toLowerCase().split("/")
  const knownTypes = new Set([
    "tractors", "harvesters", "trailers", "tools", "cars", "trucks",
    "cutters", "forageharvesters", "loaders", "telehandlers", "wheelloaders",
    "placeables", "sprayers", "mowers", "balers", "spreaders", "cultivators",
    "plows", "seeders", "weeders", "rollers", "levelers", "forklifts",
    "conveyors", "augerwagons", "mixerwagons", "animals", "pallets",
  ])
  // Map from lowercase to proper camelCase keys
  const caseMap: Record<string, string> = {
    forageharvesters: "forageHarvesters",
    telehandlers: "teleHandlers",
    wheelloaders: "wheelLoaders",
    augerwagons: "augerWagons",
    mixerwagons: "mixerWagons",
  }
  for (const part of parts) {
    if (knownTypes.has(part)) {
      return caseMap[part] ?? part
    }
  }
  return "other"
}

/// Converts day_time (seconds since midnight) to "HH:MM" string.
export function dayTimeToHHMM(seconds: number): string {
  const totalMinutes = Math.floor(seconds / 60)
  const hours = Math.floor(totalMinutes / 60) % 24
  const minutes = totalMinutes % 60
  return `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}`
}

/// Converts "HH:MM" string to day_time in seconds.
export function hhmmToDayTime(hhmm: string): number {
  const [h, m] = hhmm.split(":").map(Number)
  return (h || 0) * 3600 + (m || 0) * 60
}

/// Deduces the season from the current day and daysPerPeriod.
/// FS25 has 4 seasons cycling: SPRING (0), SUMMER (1), AUTUMN (2), WINTER (3)
export function dayToSeason(day: number, daysPerPeriod: number): string {
  if (daysPerPeriod <= 0) return "SPRING"
  const period = Math.floor((day - 1) / daysPerPeriod) % 4
  const seasons = ["SPRING", "SUMMER", "AUTUMN", "WINTER"]
  return seasons[period]
}

/// Formats a duration in milliseconds to a human-readable string.
export function formatDuration(ms: number): string {
  const totalMinutes = Math.floor(ms / 60000)
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60
  if (hours > 0 && minutes > 0) return `${hours}h ${minutes.toString().padStart(2, "0")}min`
  if (hours > 0) return `${hours}h`
  return `${minutes}min`
}

/// Formats operating hours.
export function formatOperatingTime(hours: number): string {
  if (hours < 1) {
    const minutes = Math.round(hours * 60)
    return `${minutes}min`
  }
  const h = Math.floor(hours)
  const m = Math.round((hours - h) * 60)
  if (m === 0) return `${h}h`
  return `${h}h ${m.toString().padStart(2, "0")}min`
}
