import { writable, derived, type Writable } from "svelte/store";
import { browser } from "$app/environment";

// Type definitions
interface AppConfig {
  enforceAgentOwnership: boolean;
}

// Default config values - default to false to allow viewing all agents
const defaultConfig: AppConfig = {
  enforceAgentOwnership: false, // Default to false to allow viewing all agents
};

// Pure function to load config from storage
const loadStoredConfig = (): Partial<AppConfig> =>
  browser && localStorage.getItem("portico_config")
    ? JSON.parse(localStorage.getItem("portico_config") || "{}")
    : {};

// Create the store with defaults merged with stored values
const configStore: Writable<AppConfig> = writable<AppConfig>({
  ...defaultConfig,
  ...loadStoredConfig(),
});

// Export individual settings as derived stores for easier consumption
export const enforceAgentOwnership = derived(
  configStore,
  ($config) => $config.enforceAgentOwnership,
);

// Side effect isolated in subscription
if (browser) {
  configStore.subscribe((config) => {
    localStorage.setItem("portico_config", JSON.stringify(config));
  });
}

// Pure function to update config
export function updateConfig(partialConfig: Partial<AppConfig>): void {
  configStore.update((config) => ({
    ...config,
    ...partialConfig,
  }));
}

export default configStore;
