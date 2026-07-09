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
