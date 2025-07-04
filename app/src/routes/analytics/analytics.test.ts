import { render, waitFor, screen } from "@testing-library/svelte";
import Analytics from "./+page.svelte";

vi.mock("./api", () => ({
  getAnalyticsCounts: vi.fn(() =>
    Promise.resolve({
      agentCount: 2,
      runtimeSessionCount: 2,
      stepCount: 8,
    }),
  ),
  getAgentPerformance: vi.fn(() =>
    Promise.resolve([
      {
        agentId: 1,
        totalRuns: 1,
        successRate: 100,
        avgResponseTime: "1.2s",
      },
      {
        agentId: 2,
        totalRuns: 1,
        successRate: 100,
        avgResponseTime: "2.1s",
      },
    ]),
  ),
  getErrorDistribution: vi.fn(() =>
    Promise.resolve({
      completed: 2,
      cancelled: 0,
      running: 0,
      waiting: 0,
    }),
  ),
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
