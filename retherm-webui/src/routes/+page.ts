import type { RethermSettings } from '$lib/retherm';

export async function load({ fetch }) {
   const response = await fetch('/api/settings');

   if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
   }

   const settings: RethermSettings = await response.json();
   console.log(settings);

   return { settings };
}
