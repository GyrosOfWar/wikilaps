export type SearchParamValue = string | number | null | undefined;

export function optionalIntegerParameter(
  params: URLSearchParams,
  name: string,
): number | undefined {
  const param = params.get(name);
  if (!param) {
    return undefined;
  }

  const parsed = parseInt(param, 10);
  return isNaN(parsed) ? undefined : parsed;
}

export function requiredIntegerParameter(
  params: URLSearchParams,
  name: string,
  defaultValue: number,
): number {
  return optionalIntegerParameter(params, name) ?? defaultValue;
}

/**
 * Copies `current` and applies `updates` on top of it, dropping any parameter set to
 * null/undefined/"". Returns a query string including the leading "?", or "" if empty.
 */
export function withSearchParams(
  current: URLSearchParams,
  updates: Record<string, SearchParamValue>,
): string {
  const params = new URLSearchParams(current);
  for (const [name, value] of Object.entries(updates)) {
    if (value === null || value === undefined || value === "") {
      params.delete(name);
    } else {
      params.set(name, String(value));
    }
  }

  const query = params.toString();
  return query ? `?${query}` : "";
}
