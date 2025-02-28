<script>
  import { page } from '$app/stores';
  import { Breadcrumb, BreadcrumbItem } from 'flowbite-svelte';
  import { onNavigate } from '$app/navigation';
  import { onMount, onDestroy } from 'svelte';
  
  let history = [];
  let currentPath = '';
  
  $effect(() => {
    currentPath = $page.url.pathname;
    if (!history.includes(currentPath)) {
      history = [...history, currentPath];
    }
  });
  
  // Handle browser back/forward buttons
  onMount(() => {
    const handlePopState = () => {
      const path = window.location.pathname;
      // Update history array to reflect navigation
      const index = history.indexOf(path);
      if (index >= 0) {
        history = history.slice(0, index + 1);
      } else {
        history = [...history, path];
      }
    };
    
    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  });
  
  // Format path for display
  function formatPathSegment(segment) {
    if (!segment) return 'Home';
    return segment.charAt(0).toUpperCase() + segment.slice(1).replace(/-/g, ' ');
  }
  
  // Generate breadcrumb items from path
  function getPathSegments(path) {
    const segments = path.split('/').filter(Boolean);
    return segments.map((segment, index) => {
      const url = '/' + segments.slice(0, index + 1).join('/');
      return { name: formatPathSegment(segment), href: url };
    });
  }
</script>

<Breadcrumb aria-label="Navigation history">
  <BreadcrumbItem href="/">Home</BreadcrumbItem>
  {#each getPathSegments(currentPath) as segment}
    <BreadcrumbItem href={segment.href}>{segment.name}</BreadcrumbItem>
  {/each}
</Breadcrumb> 