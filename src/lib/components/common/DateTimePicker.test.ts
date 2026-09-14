import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

import DateTimePicker from "$lib/components/common/DateTimePicker.svelte";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    scaleFactor: vi.fn().mockResolvedValue(1),
    innerSize: vi.fn().mockResolvedValue({ toLogical: () => ({ width: 400, height: 900 }) }),
  }),
  LogicalSize: class LogicalSize {},
}));

afterEach(cleanup);

beforeAll(() => {
  Element.prototype.animate = () => ({ cancel: vi.fn(), onfinish: null }) as unknown as Animation;
});

describe("DateTimePicker", () => {
  it("supports required date-only fields with a minimum date", async () => {
    const onDateChange = vi.fn();
    render(DateTimePicker, {
      props: {
        date: "2026-08-15",
        minDate: "2026-08-15",
        showTime: false,
        clearable: false,
        triggerAriaLabel: "Choose event end date",
        onDateChange,
      },
    });

    await fireEvent.click(screen.getByRole("button", { name: /Deadline: Aug 15, 2026/i }));

    const unavailableDay = screen.getByRole("button", { name: /August 14, 2026/i });
    expect(unavailableDay).toHaveProperty("disabled", true);
    expect(screen.queryByLabelText("Deadline hour")).toBeNull();
    expect(screen.queryByRole("button", { name: "Clear" })).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: /August 16, 2026/i }));
    expect(onDateChange).toHaveBeenCalledWith("2026-08-16");
  });
});
