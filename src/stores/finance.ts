import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type {
  SavegameData,
  FarmStatistics,
  DailyFinance,
  FinanceChanges,
} from "@/lib/types";

export const useFinanceStore = defineStore("finance", () => {
  const money = ref(0);
  const loan = ref(0);
  const statistics = ref<FarmStatistics | null>(null);
  const dailyFinances = ref<DailyFinance[]>([]);

  const originalMoney = ref(0);
  const originalLoan = ref(0);

  const isDirty = computed(
    () => money.value !== originalMoney.value || loan.value !== originalLoan.value,
  );

  const changeCount = computed(() => {
    let count = 0;
    if (money.value !== originalMoney.value) count++;
    if (loan.value !== originalLoan.value) count++;
    return count;
  });

  function hydrate(data: SavegameData) {
    const farm = data.farms[0];
    if (farm) {
      money.value = farm.money;
      loan.value = farm.loan;
      statistics.value = farm.statistics;
      dailyFinances.value = farm.dailyFinances;
    } else {
      money.value = data.career.money;
      loan.value = 0;
      statistics.value = null;
      dailyFinances.value = [];
    }
    originalMoney.value = money.value;
    originalLoan.value = loan.value;
  }

  function setMoney(value: number) {
    money.value = Math.max(0, value);
  }

  function setLoan(value: number) {
    loan.value = Math.max(0, value);
  }

  function addMoney(amount: number) {
    money.value = Math.max(0, money.value + amount);
  }

  function repayLoan(deductFromMoney: boolean) {
    if (deductFromMoney) {
      money.value = Math.max(0, money.value - loan.value);
    }
    loan.value = 0;
  }

  function resetChanges() {
    money.value = originalMoney.value;
    loan.value = originalLoan.value;
  }

  function getChanges(): FinanceChanges | null {
    if (!isDirty.value) return null;

    const changes: FinanceChanges = {};
    if (money.value !== originalMoney.value) {
      changes.money = money.value;
    }
    if (loan.value !== originalLoan.value) {
      changes.loan = loan.value;
    }
    return changes;
  }

  function commitChanges() {
    originalMoney.value = money.value;
    originalLoan.value = loan.value;
  }

  return {
    money,
    loan,
    statistics,
    dailyFinances,
    originalMoney,
    originalLoan,
    isDirty,
    changeCount,
    hydrate,
    setMoney,
    setLoan,
    addMoney,
    repayLoan,
    resetChanges,
    getChanges,
    commitChanges,
  };
});
