import { sequence } from "@sveltejs/kit/hooks";
import type { Handle, HandleFetch } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { getTextDirection } from "$lib/paraglide/runtime";
import { paraglideMiddleware } from "$lib/paraglide/server";

const backendUrl = env.BACKEND_URL || "http://localhost:13252";

const handleParaglide: Handle = ({ event, resolve }) =>
  paraglideMiddleware(event.request, ({ request, locale }) => {
    event.request = request;

    return resolve(event, {
      filterSerializedResponseHeaders: (name) => name === "content-type",
      transformPageChunk: ({ html }) =>
        html
          .replace("%paraglide.lang%", locale)
          .replace("%paraglide.dir%", getTextDirection(locale)),
    });
  });

const forwardBackendCookies: Handle = async ({ event, resolve }) => {
  const response = await resolve(event);
  for (const cookie of event.locals.backendSetCookies ?? []) {
    response.headers.append("set-cookie", cookie);
  }
  return response;
};

export const handle: Handle = sequence(forwardBackendCookies, handleParaglide);

// Handle requests to the backend (anything starting with /api) when the happen server-side
// including proxying the cookie/set-cookie headers when they are present.
export const handleFetch: HandleFetch = async ({ event, request, fetch }) => {
  const url = new URL(request.url);
  if (!url.pathname.startsWith("/api")) {
    return fetch(request);
  }

  const backend = new URL(backendUrl);
  url.protocol = backend.protocol;
  url.host = backend.host;

  const proxied = new Request(url, request);
  const cookie = event.request.headers.get("cookie");
  if (cookie) {
    proxied.headers.set("cookie", cookie);
  }
  // Forward the real user-agent so the backend sees the browser, not "node".
  const userAgent = event.request.headers.get("user-agent");
  if (userAgent) {
    proxied.headers.set("user-agent", userAgent);
  }

  const response = await fetch(proxied);

  const setCookie = response.headers.getSetCookie();
  if (setCookie.length > 0) {
    event.locals.backendSetCookies = [...(event.locals.backendSetCookies ?? []), ...setCookie];
  }

  return response;
};
