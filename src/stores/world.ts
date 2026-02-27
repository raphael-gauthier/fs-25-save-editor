import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { Environment, WeatherEvent, EnvironmentChanges } from "@/lib/types";
import { dayTimeToHHMM, dayToSeason } from "@/lib/utils";

export const useWorldStore = defineStore("world", () => {
  const environment = ref<Environment | null>(null);
  const originalEnvironment = ref<Environment | null>(null);

  function hydrate(env: Environment) {
    const serialized = JSON.stringify(env);
    environment.value = JSON.parse(serialized);
    originalEnvironment.value = JSON.parse(serialized);
  }

  // Getters
  const isDirty = computed(() => {
    if (!environment.value || !originalEnvironment.value) return false;
    const e = environment.value;
    const o = originalEnvironment.value;
    if (e.dayTime !== o.dayTime) return true;
    if (e.currentDay !== o.currentDay) return true;
    if (e.snowHeight !== o.snowHeight) return true;
    if (e.groundWetness !== o.groundWetness) return true;
    if (e.weatherForecast.length !== o.weatherForecast.length) return true;
    for (let i = 0; i < e.weatherForecast.length; i++) {
      const a = e.weatherForecast[i];
      const b = o.weatherForecast[i];
      if (
        a.typeName !== b.typeName ||
        a.season !== b.season ||
        a.variationIndex !== b.variationIndex ||
        a.startDay !== b.startDay ||
        a.startDayTime !== b.startDayTime ||
        a.duration !== b.duration
      )
        return true;
    }
    return false;
  });

  const changeCount = computed(() => {
    if (!environment.value || !originalEnvironment.value) return 0;
    let count = 0;
    const e = environment.value;
    const o = originalEnvironment.value;
    if (e.dayTime !== o.dayTime) count++;
    if (e.currentDay !== o.currentDay) count++;
    if (e.snowHeight !== o.snowHeight) count++;
    if (e.groundWetness !== o.groundWetness) count++;
    // Count forecast as a single change if modified
    const forecastChanged =
      e.weatherForecast.length !== o.weatherForecast.length ||
      e.weatherForecast.some((a, i) => {
        const b = o.weatherForecast[i];
        return (
          a.typeName !== b.typeName ||
          a.season !== b.season ||
          a.variationIndex !== b.variationIndex ||
          a.startDay !== b.startDay ||
          a.startDayTime !== b.startDayTime ||
          a.duration !== b.duration
        );
      });
    if (forecastChanged) count++;
    return count;
  });

  const currentTime = computed(() => {
    if (!environment.value) return "00:00";
    return dayTimeToHHMM(environment.value.dayTime);
  });

  const currentSeason = computed(() => {
    if (!environment.value) return "SPRING";
    return dayToSeason(environment.value.currentDay, environment.value.daysPerPeriod);
  });

  // Actions
  function setDayTime(seconds: number) {
    if (environment.value) {
      environment.value.dayTime = seconds;
    }
  }

  function setCurrentDay(day: number) {
    if (environment.value) {
      environment.value.currentDay = day;
    }
  }

  function setSnowHeight(height: number) {
    if (environment.value) {
      environment.value.snowHeight = height;
    }
  }

  function setGroundWetness(wetness: number) {
    if (environment.value) {
      environment.value.groundWetness = wetness;
    }
  }

  function updateWeatherEvent(index: number, changes: Partial<WeatherEvent>) {
    if (environment.value && environment.value.weatherForecast[index]) {
      Object.assign(environment.value.weatherForecast[index], changes);
    }
  }

  function deleteWeatherEvent(index: number) {
    if (environment.value) {
      environment.value.weatherForecast.splice(index, 1);
    }
  }

  function addWeatherEvent(event: WeatherEvent) {
    if (environment.value) {
      environment.value.weatherForecast.push(event);
    }
  }

  function forceSunny() {
    if (environment.value) {
      environment.value.weatherForecast = environment.value.weatherForecast.map((e) => ({
        ...e,
        typeName: "SUN",
      }));
    }
  }

  function removeTwisters() {
    if (environment.value) {
      environment.value.weatherForecast = environment.value.weatherForecast.filter(
        (e) => e.typeName !== "TWISTER",
      );
    }
  }

  function resetChanges() {
    if (originalEnvironment.value) {
      environment.value = JSON.parse(JSON.stringify(originalEnvironment.value));
    }
  }

  function getChanges(): EnvironmentChanges | null {
    if (!environment.value || !originalEnvironment.value || !isDirty.value) return null;
    const e = environment.value;
    const o = originalEnvironment.value;
    const changes: EnvironmentChanges = {};

    if (e.dayTime !== o.dayTime) changes.dayTime = e.dayTime;
    if (e.currentDay !== o.currentDay) changes.currentDay = e.currentDay;
    if (e.snowHeight !== o.snowHeight) changes.snowHeight = e.snowHeight;
    if (e.groundWetness !== o.groundWetness) changes.groundWetness = e.groundWetness;

    const forecastChanged =
      e.weatherForecast.length !== o.weatherForecast.length ||
      e.weatherForecast.some((a, i) => {
        const b = o.weatherForecast[i];
        return (
          a.typeName !== b.typeName ||
          a.season !== b.season ||
          a.variationIndex !== b.variationIndex ||
          a.startDay !== b.startDay ||
          a.startDayTime !== b.startDayTime ||
          a.duration !== b.duration
        );
      });
    if (forecastChanged) {
      changes.weatherForecast = e.weatherForecast;
    }

    return changes;
  }

  function commitChanges() {
    if (environment.value) {
      originalEnvironment.value = JSON.parse(JSON.stringify(environment.value));
    }
  }

  return {
    environment,
    originalEnvironment,
    isDirty,
    changeCount,
    currentTime,
    currentSeason,
    hydrate,
    setDayTime,
    setCurrentDay,
    setSnowHeight,
    setGroundWetness,
    updateWeatherEvent,
    deleteWeatherEvent,
    addWeatherEvent,
    forceSunny,
    removeTwisters,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
