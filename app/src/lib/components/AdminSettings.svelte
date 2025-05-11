<script lang="ts">
  import { enforceAgentOwnership, updateConfig } from "$lib/stores/configStore";

  function toggleAgentOwnership(): void {
    updateConfig({ enforceAgentOwnership: !$enforceAgentOwnership });
  }
</script>

<div class="settings-container">
  <h2 class="text-xl font-semibold mb-4">Admin Settings</h2>

  <div class="setting-group">
    <div class="flex items-center justify-between p-4 border rounded">
      <div>
        <h3 class="font-medium">Enforce Agent Ownership</h3>
        <p class="text-sm text-gray-500">
          When enabled, users can only access their own agents and related data.
          Disable for admin or debugging purposes.
        </p>
      </div>
      <label class="relative inline-flex items-center cursor-pointer">
        <input
          type="checkbox"
          class="sr-only peer"
          checked={$enforceAgentOwnership}
          on:change={toggleAgentOwnership}
        />
        <div
          class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"
        ></div>
      </label>
    </div>
  </div>
</div>
