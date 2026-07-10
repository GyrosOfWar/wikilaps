// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    interface Locals {
      // Raw `Set-Cookie` headers captured from backend responses during SSR,
      // relayed to the browser in the `handle` hook (see hooks.server.ts).
      backendSetCookies?: string[];
    }
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
