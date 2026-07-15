import * as m from "$lib/paraglide/messages";
import type { SessionType } from "$lib/api";

const sessionTypeLabels: Record<SessionType, () => string> = {
  sprint_qualifying: m.session_type_sprint_qualification,
  sprint_race: m.session_type_sprint_race,
  qualifying: m.session_type_qualifying,
  race: m.session_type_race,
};

/** Localized, human-readable label for a session type. */
export const sessionTypeLabel = (type: SessionType): string => sessionTypeLabels[type]();

// translate a GP based on its ID like `las-vegas` to a message key like `gp_las_vegas`
export function grandPrixName(grandPrixId: string) {
  const id = `gp_${grandPrixId.replace("-", "_")}`;
  // @ts-expect-error dynamic key but it's generally fine
  const fn = m[id];
  if (fn) {
    return fn();
  } else {
    console.warn(`No translation key found for input '${id}', falling back to ID`);
    return grandPrixId;
  }
}
