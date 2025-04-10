import { writable } from "svelte/store";

// Initialize from session storage if available
const storedPaths =
  typeof sessionStorage !== "undefined"
    ? JSON.parse(sessionStorage.getItem("navigationHistory") || "[]")
    : [];

// Create the store
const navigationHistory = writable(storedPaths);

// Subscribe to changes and update session storage
if (typeof sessionStorage !== "undefined") {
  navigationHistory.subscribe((value) => {
    sessionStorage.setItem("navigationHistory", JSON.stringify(value));
  });
}

// Helper functions
function addPath(path, title) {
  navigationHistory.update((paths) => {
    // Don't add duplicate consecutive paths
    if (paths.length > 0 && paths[paths.length - 1].path === path) {
      return paths;
    }

    // Add the new path
    return [...paths, { path, title }];
  });
}

function clearHistory() {
  navigationHistory.set([]);
}

function goBack() {
  navigationHistory.update((paths) => {
    if (paths.length <= 1) return paths;
    return paths.slice(0, -1);
  });
  return navigationHistory;
}

export { navigationHistory, addPath, clearHistory, goBack };
