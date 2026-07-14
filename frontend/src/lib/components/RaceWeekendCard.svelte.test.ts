import { describe, expect, it, vi } from "vitest";
import { render } from "vitest-browser-svelte";
import { page } from "vitest/browser";
import RaceWeekendCard from "./RaceWeekendCard.svelte";
import { finishedRaceWeekend, makeSession, makeWeekend, upcomingWeekend } from "$lib/fixtures";
import * as m from "$lib/paraglide/messages";

describe("RaceWeekendCard", () => {
  it("renders an upcoming weekend as disabled with no sessions", async () => {
    render(RaceWeekendCard, { weekend: upcomingWeekend, onSubmitVote: vi.fn() });

    await expect.element(page.getByText(m.race_voting_not_yet())).toBeVisible();
    // Sessions are hidden entirely until the weekend gets underway.
    await expect
      .element(page.getByRole("heading", { name: m.session_type_race() }))
      .not.toBeInTheDocument();
    await expect
      .element(page.getByRole("button", { name: m.vote_submit() }))
      .not.toBeInTheDocument();
  });

  it("renders a votable finished race with vote controls", async () => {
    render(RaceWeekendCard, { weekend: finishedRaceWeekend, onSubmitVote: vi.fn() });

    await expect.element(page.getByText(m.race_voting_not_yet())).not.toBeInTheDocument();
    await expect.element(page.getByRole("heading", { name: m.session_type_race() })).toBeVisible();
    // Race sessions offer the three-way choice.
    await expect.element(page.getByRole("radio", { name: m.vote_type_full_race() })).toBeVisible();
    await expect.element(page.getByRole("radio", { name: m.vote_type_race_in_30() })).toBeVisible();
    await expect.element(page.getByRole("radio", { name: m.vote_type_highlights() })).toBeVisible();
    await expect.element(page.getByRole("button", { name: m.vote_submit() })).toBeVisible();
  });

  it("submits the picked vote for the right session", async () => {
    const onSubmitVote = vi.fn();
    render(RaceWeekendCard, { weekend: finishedRaceWeekend, onSubmitVote });

    await page.getByRole("radio", { name: m.vote_type_full_race() }).click();
    await page.getByRole("button", { name: m.vote_submit() }).click();

    expect(onSubmitVote).toHaveBeenCalledWith(finishedRaceWeekend.sessions[0].id, "FullRace");
  });

  it("shows results instead of the form once the user has voted", async () => {
    const weekend = makeWeekend({
      upcoming: false,
      sessions: [
        makeSession({
          id: 10,
          sessionType: "race",
          votingAllowed: true,
          userVote: "FullRace",
          votes: { full: 7, raceIn30: 3, highlights: 2 },
        }),
      ],
    });
    render(RaceWeekendCard, { weekend, onSubmitVote: vi.fn() });

    await expect
      .element(page.getByRole("button", { name: m.vote_submit() }))
      .not.toBeInTheDocument();
    await expect.element(page.getByText(m.poll_total_votes({ count: 12 }))).toBeVisible();
  });
});
