import { Temporal } from "temporal-polyfill";
import { getLocale } from "$lib/paraglide/runtime.js";

export function formatDate(date: string): string {
  const locale = getLocale();
  const parsed = Temporal.PlainDate.from(date);
  return parsed.toLocaleString(locale);
}
