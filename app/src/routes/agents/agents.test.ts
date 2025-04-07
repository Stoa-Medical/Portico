import { render, screen } from "@testing-library/svelte";
import AgentsPage from "./+page.svelte";

// Mocking the `api.js` file
vi.mock("./api", () => ({
  getAgents: vi.fn().mockResolvedValue([
    {
      id: 1,
      name: "Test Agent 1",
      type: "Assistant",
      status: "Active",
      lastActive: "2025-04-07",
    },
    {
      id: 2,
      name: "Test Agent 2",
      type: "Researcher",
      status: "Inactive",
      lastActive: "2025-04-06",
    },
  ]),
  getAgentSteps: vi.fn().mockReturnValue([]),
}));

const t = {
  render: () => render(AgentsPage),
};

describe("Agents Component", () => {
  it("renders the list of agents", async () => {
    t.render();

    // Then ensure agents are fetched and rendered:
    const agent1 = await screen.findByText("Test Agent 1");
    const agent2 = await screen.findByText("Test Agent 2");

    expect(agent1).toBeInTheDocument();
    expect(agent2).toBeInTheDocument();
  });
});
