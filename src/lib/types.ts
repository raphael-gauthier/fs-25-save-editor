export interface LocalizedMessage {
  code: string;
  params: Record<string, string>;
}

export interface SavegameSummary {
  path: string;
  name: string;
  mapTitle: string;
  money: number;
  playTime: number;
  saveDate: string;
  economicDifficulty: string;
}

export interface CareerSavegame {
  savegameName: string;
  creationDate: string;
  mapId: string;
  mapTitle: string;
  saveDate: string;
  economicDifficulty: string;
  money: number;
  playTime: number;
  growthMode: number;
  plannedDaysPerPeriod: number;
  plowingRequired: boolean;
  stonesEnabled: boolean;
  weedsEnabled: boolean;
  limeRequired: boolean;
  snowEnabled: boolean;
  fuelUsage: number;
  trafficEnabled: boolean;
}

export interface Farm {
  farmId: number;
  name: string;
  color: number;
  loan: number;
  money: number;
  players: FarmPlayer[];
  statistics: FarmStatistics;
  dailyFinances: DailyFinance[];
}

export interface FarmPlayer {
  uniqueUserId: string;
  farmManager: boolean;
  lastNickname: string;
  timeLastConnected: string;
  buyVehicle: boolean;
  sellVehicle: boolean;
  buyPlaceable: boolean;
  sellPlaceable: boolean;
  manageContracts: boolean;
  tradeAnimals: boolean;
  createFields: boolean;
  landscaping: boolean;
  hireAssistant: boolean;
  resetVehicle: boolean;
  manageProductions: boolean;
  cutTrees: boolean;
  manageRights: boolean;
  transferMoney: boolean;
  updateFarm: boolean;
  manageContracting: boolean;
}

export interface FarmStatistics {
  traveledDistance: number;
  fuelUsage: number;
  seedUsage: number;
  sprayUsage: number;
  workedHectares: number;
  cultivatedHectares: number;
  sownHectares: number;
  sprayedHectares: number;
  threshedHectares: number;
  plowedHectares: number;
  baleCount: number;
  missionCount: number;
  playTime: number;
  revenue: number;
  expenses: number;
}

export interface DailyFinance {
  day: number;
  newVehiclesCost: number;
  soldVehicles: number;
  newAnimalsCost: number;
  soldAnimals: number;
  constructionCost: number;
  soldBuildings: number;
  fieldPurchase: number;
  soldFields: number;
  vehicleRunningCost: number;
  vehicleLeasingCost: number;
  propertyMaintenance: number;
  propertyIncome: number;
  productionCosts: number;
  soldProducts: number;
  harvestIncome: number;
  missionIncome: number;
  wagePayment: number;
  loanInterest: number;
  otherIncome: number;
  otherExpenses: number;
}

export interface Position {
  x: number;
  y: number;
  z: number;
}

export interface Rotation {
  x: number;
  y: number;
  z: number;
}

export interface Vehicle {
  uniqueId: string;
  filename: string;
  displayName: string;
  age: number;
  price: number;
  farmId: number;
  propertyState: "None" | "Owned" | "Rented";
  operatingTime: number;
  position: Position | null;
  rotation: Rotation | null;
  configurations: VehicleConfiguration[];
  fillUnits: FillUnit[];
  attachedImplements: AttachedImplement[];
}

export interface VehicleConfiguration {
  name: string;
  id: string;
}

export interface FillUnit {
  index: number;
  fillType: string;
  fillLevel: number;
  capacity: number | null;
}

export interface AttachedImplement {
  jointIndex: number;
  attachedVehicleUniqueId: string;
  moveDown: boolean;
}

export interface SaleItem {
  index: number;
  xmlFilename: string;
  displayName: string;
  age: number;
  price: number;
  damage: number;
  wear: number;
  operatingTime: number;
  timeLeft: number;
  isGenerated: boolean;
  boughtConfigurations: BoughtConfiguration[];
}

export interface BoughtConfiguration {
  name: string;
  id: string;
}

export interface Field {
  id: number;
  plannedFruit: string;
  fruitType: string;
  growthState: number;
  lastGrowthState: number;
  weedState: number;
  stoneLevel: number;
  sprayLevel: number;
  sprayType: string;
  limeLevel: number;
  plowLevel: number;
  rollerLevel: number;
  stubbleShredLevel: number;
  waterLevel: number;
  groundType: string;
}

export interface Farmland {
  id: number;
  farmId: number;
}

export interface FieldChangePayload {
  id: number;
  fruitType?: string;
  plannedFruit?: string;
  growthState?: number;
  groundType?: string;
  weedState?: number;
  stoneLevel?: number;
  sprayLevel?: number;
  sprayType?: string;
  limeLevel?: number;
  plowLevel?: number;
  rollerLevel?: number;
  stubbleShredLevel?: number;
  waterLevel?: number;
}

export interface FarmlandChangePayload {
  id: number;
  farmId: number;
}

export interface Environment {
  dayTime: number;
  currentDay: number;
  currentMonotonicDay: number;
  daysPerPeriod: number;
  weatherForecast: WeatherEvent[];
  snowHeight: number;
  groundWetness: number;
}

export interface WeatherEvent {
  typeName: string;
  season: string;
  variationIndex: number;
  startDay: number;
  startDayTime: number;
  duration: number;
}

export interface EnvironmentChanges {
  dayTime?: number;
  currentDay?: number;
  snowHeight?: number;
  groundWetness?: number;
  weatherForecast?: WeatherEvent[];
}

export interface Placeable {
  index: number;
  filename: string;
  displayName: string;
  farmId: number;
  price: number;
  age: number;
  position: Position | null;
  isPrePlaced: boolean;
  isUnderConstruction: boolean;
  constructionSteps: ConstructionStep[];
  productionInputs: ProductionStock[];
  productionOutputs: ProductionStock[];
}

export interface ConstructionStep {
  stepIndex: number;
  materials: ConstructionMaterial[];
}

export interface ConstructionMaterial {
  fillType: string;
  amountRemaining: number;
  amountTotal: number;
}

export interface ProductionStock {
  fillType: string;
  amount: number;
  capacity: number;
}

export interface PlaceableChangePayload {
  index: number;
  farmId?: number;
  price?: number;
  completeConstruction: boolean;
  productionInputs?: ProductionStockChangePayload[];
  productionOutputs?: ProductionStockChangePayload[];
}

export interface ProductionStockChangePayload {
  fillType: string;
  amount: number;
}

export interface Mission {
  uniqueId: string;
  missionType: string;
  status: "Created" | "Running" | "Completed";
  reward: number;
  reimbursement: number;
  completion: number;
  fieldId: number | null;
  fruitType: string | null;
  expectedLiters: number | null;
  depositedLiters: number | null;
}

export interface Collectible {
  index: number;
  collected: boolean;
}

export interface ContractSettings {
  leaseVehicle: number;
  missionPerFarm: number;
  allowClearAdd: number;
}

export interface MissionChangePayload {
  uniqueId: string;
  reward?: number;
  completion?: number;
  status?: string;
  reimbursement?: number;
}

export interface CollectibleChangePayload {
  index: number;
  collected: boolean;
}

export interface ContractSettingsChangePayload {
  leaseVehicle?: number;
  missionPerFarm?: number;
  allowClearAdd?: number;
}

export interface SavegameData {
  path: string;
  career: CareerSavegame;
  farms: Farm[];
  vehicles: Vehicle[];
  sales: SaleItem[];
  fields: Field[];
  farmlands: Farmland[];
  placeables: Placeable[];
  missions: Mission[];
  collectibles: Collectible[];
  contractSettings: ContractSettings | null;
  environment: Environment | null;
  warnings: LocalizedMessage[];
}

export interface BackupInfo {
  name: string;
  path: string;
  createdAt: string;
  sizeBytes: number;
}

export interface FinanceChanges {
  money?: number;
  loan?: number;
}

export interface SavegameChanges {
  finance?: FinanceChanges;
  vehicles?: unknown[];
  sales?: unknown[];
  fields?: FieldChangePayload[];
  farmlands?: FarmlandChangePayload[];
  placeables?: PlaceableChangePayload[];
  missions?: MissionChangePayload[];
  collectibles?: CollectibleChangePayload[];
  contractSettings?: ContractSettingsChangePayload;
  environment?: EnvironmentChanges;
}

export interface SaveResult {
  success: boolean;
  backupPath: string | null;
  filesModified: string[];
  errors: LocalizedMessage[];
}
