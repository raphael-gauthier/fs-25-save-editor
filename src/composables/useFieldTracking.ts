import { computed, type Ref } from "vue";

export function useFieldTracking<T>(
  current: Ref<T>,
  original: Ref<T>,
  tolerance: number = 0,
) {
  const isModified = computed(() => {
    if (tolerance > 0) {
      return Math.abs(Number(current.value) - Number(original.value)) > tolerance;
    }
    return current.value !== original.value;
  });

  const fieldClass = computed(() =>
    isModified.value ? "border-l-2 border-amber-500 pl-2" : "",
  );

  return {
    isModified,
    fieldClass,
  };
}
