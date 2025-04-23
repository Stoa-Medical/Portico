import { render, waitFor, screen } from "@testing-library/svelte";
import Analytics from "./+page.svelte";

vi.mock("$lib/supabase", () => ({
  default: {
    from: (table: string) => ({
      select: () => {
        switch (table) {
          case "agents":
            return { count: 2, data: [{ id: 1 }, { id: 2 }], error: null };
          case "runtime_sessions":
            return {
              count: 2,
              data: [
                {
                  requested_by_agent_id: 1,
                  rts_status: "completed",
                  total_execution_time: "1.2",
                },
                {
                  requested_by_agent_id: 2,
                  rts_status: "completed",
                  total_execution_time: "2.1",
                },
              ],
              error: null,
            };
          case "steps":
            return { count: 8, data: [], error: null };
          default:
            return { count: 0, data: [], error: null };
        }
      },
      eq: vi.fn().mockReturnThis(),
      order: vi.fn().mockReturnThis(),
      limit: vi.fn().mockReturnThis(),
      insert: vi.fn().mockReturnThis(),
      update: vi.fn().mockReturnThis(),
      delete: vi.fn().mockReturnThis(),
    }),
    auth: {
      getUser: vi.fn(() =>
        Promise.resolve({
          data: { user: { id: "mock-user" } },
          error: null,
        }),
      ),
    },
  },
}));

describe("Analytics Dashboard", () => {
  it("renders charts on mount", async () => {
    const { container } = render(Analytics);
    await waitFor(() =>
      expect(container.querySelector("#success-rate-chart")).toBeTruthy(),
    );
    expect(container.querySelector("#execution-time-chart")).toBeTruthy();
    expect(container.querySelector("#usage-chart")).toBeTruthy();
  });

  it("renders summary cards with correct values", async () => {
    render(Analytics);

    await waitFor(() => {
      expect(screen.getByTestId("total-agents")).toHaveTextContent("2");
      expect(screen.getByTestId("total-steps")).toHaveTextContent("8");
      expect(screen.getByTestId("avg-success-rate")).toHaveTextContent("100%");
      expect(screen.getByTestId("total-executions")).toHaveTextContent("2");
    });
  });
});
