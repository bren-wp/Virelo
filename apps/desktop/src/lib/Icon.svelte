<script lang="ts">
  export let name = 'home';

  export let size = 18;
  export let strokeWidth = 1.8;
</script>

<svg
  class="icon"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#if name === 'menu'}
    <path d="M4 7h16M4 12h16M4 17h16" />
  {:else if name === 'home'}
    <path d="M3 10.5 12 3l9 7.5" />
    <path d="M5 9.5V21h14V9.5" />
    <path d="M9 21v-7h6v7" />
  {:else if name === 'building'}
    <rect x="4" y="3" width="16" height="18" rx="2" />
    <path d="M8 7h2M14 7h2M8 11h2M14 11h2M8 15h2M14 15h2M10 21v-3h4v3" />
  {:else if name === 'users'}
    <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
    <circle cx="9" cy="7" r="4" />
    <path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75" />
  {:else if name === 'contact'}
    <rect x="3" y="4" width="18" height="16" rx="2" />
    <circle cx="9" cy="10" r="2.5" />
    <path d="M5.5 17a4 4 0 0 1 7 0M15 9h3M15 13h3" />
  {:else if name === 'folder'}
    <path d="M3 6h6l2 2h10v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
  {:else if name === 'check'}
    <rect x="3" y="3" width="18" height="18" rx="3" />
    <path d="m8 12 2.7 2.7L16.5 9" />
  {:else if name === 'note'}
    <path d="M5 3h10l4 4v14H5z" />
    <path d="M15 3v5h5M8 12h8M8 16h6" />
  {:else if name === 'file'}
    <path d="M6 2h9l5 5v15H6z" />
    <path d="M15 2v6h5" />
  {:else if name === 'activity'}
    <path d="M3 12h4l2.5-6 4.5 12 2.5-6H21" />
  {:else if name === 'wallet'}
    <path d="M4 6h14a2 2 0 0 1 2 2v11H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h12" />
    <path d="M16 11h6v5h-6a2.5 2.5 0 0 1 0-5z" />
  {:else if name === 'backup'}
    <path d="M4 7v5h5" />
    <path d="M5.5 16a8 8 0 1 0 .5-9L4 9" />
  {:else if name === 'search'}
    <circle cx="11" cy="11" r="7" />
    <path d="m20 20-4-4" />
  {:else if name === 'plus'}
    <path d="M12 5v14M5 12h14" />
  {:else if name === 'edit'}
    <path d="M12 20h9" />
    <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L8 18l-4 1 1-4z" />
  {:else if name === 'trash'}
    <path d="M3 6h18M8 6V4h8v2M7 6l1 14h8l1-14M10 10v6M14 10v6" />
  {:else if name === 'open'}
    <path d="M14 3h7v7M10 14 21 3" />
    <path d="M21 14v5a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5" />
  {:else if name === 'export'}
    <path d="M12 3v12M7 8l5-5 5 5" />
    <path d="M5 13v7h14v-7" />
  {:else if name === 'calendar'}
    <rect x="3" y="5" width="18" height="16" rx="2" />
    <path d="M16 3v4M8 3v4M3 10h18" />
  {:else if name === 'mail'}
    <rect x="3" y="5" width="18" height="14" rx="2" />
    <path d="m4 7 8 6 8-6" />
  {:else if name === 'phone'}
    <path d="M5 3h4l2 5-2.5 1.5a15 15 0 0 0 6 6L16 13l5 2v4a2 2 0 0 1-2 2C10.2 21 3 13.8 3 5a2 2 0 0 1 2-2z" />
  {:else if name === 'briefcase'}
    <rect x="3" y="7" width="18" height="13" rx="2" />
    <path d="M8 7V4h8v3M3 12h18M10 12v2h4v-2" />
  {:else}
    <path d="m12 3 1.7 4.3L18 9l-4.3 1.7L12 15l-1.7-4.3L6 9l4.3-1.7zM19 15l.8 2.2L22 18l-2.2.8L19 21l-.8-2.2L16 18l2.2-.8z" />
  {/if}
</svg>
