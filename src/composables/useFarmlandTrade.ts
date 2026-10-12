import { useFieldStore } from "@/stores/field";
import { useFinanceStore } from "@/stores/finance";

/** The player's farm in single player (FarmManager.SINGLEPLAYER_FARM_ID). */
export const PLAYER_FARM_ID = 1;

/** Why a farmland cannot be bought or sold. */
export type TradeBlock = "notBuyable" | "owned" | "otherFarm" | "notOwned" | "buildings" | "money";

/**
 * Buying and selling farmlands the way the game does: the farmland changes
 * owner and the farm pays, or gets back, the farmland's price.
 */
export function useFarmlandTrade() {
  const fieldStore = useFieldStore();
  const finance = useFinanceStore();

  function ownerOf(id: number): number | undefined {
    return fieldStore.farmlands.find((f) => f.id === id)?.farmId;
  }

  function totalPrice(ids: number[]): number {
    return ids.reduce((sum, id) => sum + (fieldStore.getFarmlandInfo(id)?.price ?? 0), 0);
  }

  /** Why `ids` cannot all be bought by the player, or null when they can. */
  function buyBlock(ids: number[]): TradeBlock | null {
    for (const id of ids) {
      const owner = ownerOf(id);
      if (!fieldStore.getFarmlandInfo(id) || owner === undefined) return "notBuyable";
      if (owner === PLAYER_FARM_ID) return "owned";
      if (owner !== 0) return "otherFarm";
    }
    // The finance store clamps money at 0, so an overdraft would be free
    return totalPrice(ids) > finance.money ? "money" : null;
  }

  /** Why `ids` cannot all be sold by the player, or null when they can. */
  function sellBlock(ids: number[]): TradeBlock | null {
    for (const id of ids) {
      const info = fieldStore.getFarmlandInfo(id);
      if (!info) return "notBuyable";
      if (ownerOf(id) !== PLAYER_FARM_ID) return "notOwned";
      if (info.builtPlaceables > 0) return "buildings";
    }
    return null;
  }

  function buy(ids: number[]): boolean {
    if (ids.length === 0 || buyBlock(ids)) return false;
    const total = totalPrice(ids);
    for (const id of ids) fieldStore.updateFarmland(id, PLAYER_FARM_ID);
    finance.addMoney(-total);
    return true;
  }

  function sell(ids: number[]): boolean {
    if (ids.length === 0 || sellBlock(ids)) return false;
    const total = totalPrice(ids);
    for (const id of ids) fieldStore.updateFarmland(id, 0);
    finance.addMoney(total);
    return true;
  }

  return { ownerOf, totalPrice, buyBlock, sellBlock, buy, sell };
}
