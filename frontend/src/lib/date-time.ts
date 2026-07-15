import { Temporal } from "temporal-polyfill";
import { getLocale } from "$lib/paraglide/runtime.js";

/** The calendar year of an ISO date string like `2025-03-15`. */
export function getYear(date: string): number {
  return Temporal.PlainDate.from(date).year;
}

export function formatDate(date: string): string {
  const locale = getLocale();
  const parsed = Temporal.PlainDate.from(date);
  return parsed.toLocaleString(locale);
}
