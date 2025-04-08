import { render, within, screen, fireEvent } from "@testing-library/svelte";
import AgentsPage from "./+page.svelte";

// Mocking the `api.js` file
vi.mock("./api", async () => {
  const originalModule = await vi.importActual("./api");
  return {
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
    deleteAgent: originalModule.deleteAgent,
    saveAgent: originalModule.saveAgent,
  };
});

const t = {
  render: () => render(AgentsPage),
};

describe("agents.test.ts - Agents Page", () => {
  it("renders the list of agents", async () => {
    t.render();

    // Then agents are fetched and rendered:
    const agent1 = await screen.findByText("Test Agent 1");
    const agent2 = await screen.findByText("Test Agent 2");

    expect(agent1).toBeInTheDocument();
    expect(agent2).toBeInTheDocument();
  });

  it("allows a user to delete an existing agent from the list", async () => {
    t.render();

    // When the user opens the agent
    const agent1 = await screen.findByText("Test Agent 1");
    fireEvent.click(agent1);

    // Set up confirmation to click OK on the confirm dialog (simulated)
    vi.spyOn(window, "confirm").mockReturnValueOnce(true);

    // Then clicks the delete button and confirms
    const deleteButton = await screen.findByText("Delete");
    await fireEvent.click(deleteButton);

    // Then the agent is removed from the list
    expect(screen.queryByText("Test Agent 1")).not.toBeInTheDocument();
  });

  it("allows a user to create a new agent", async () => {
    t.render();

    // When you click the 'Add Agent' button in the action bar
    const addButton = await screen.findByText("Add Agent");
    fireEvent.click(addButton);

    // Wait for the modal to appear by checking for the title
    const modalTitle = await screen.findByText("Add New Agent");
    expect(modalTitle).toBeInTheDocument();

    // Then fill out the agent name
    fireEvent.input(screen.getByPlaceholderText("Enter agent name"), {
      target: { value: "New Agent Test" },
    });

    // Then submit the modal form
    const submitButton = await screen.findByText("Create");
    fireEvent.click(submitButton);

    // Then the new agent is added to the table
    const table = await screen.findByRole("table");
    const newAgentInTable = within(table).getByText("New Agent Test");

    expect(newAgentInTable).toBeInTheDocument();
  });

  it("allows a user to edit an existing agent", async () => {
    t.render();

    // Open the details of "Test Agent 1"
    const agent1 = await screen.findByText("Test Agent 1");
    fireEvent.click(agent1);

    await screen.findByText("General");

    // Modify the name field
    const nameInput = screen.getByLabelText("Agent Name");
    fireEvent.input(nameInput, {
      target: { value: "Test Updated Agent Name" },
    });

    // Save changes
    const saveButton = screen.getByText("Save Changes");
    fireEvent.click(saveButton);

    // Assert that the updated name appears in the table
    const table = await screen.findByRole("table");
    const updatedAgentInTable = within(table).getByText(
      "Test Updated Agent Name"
    );
    expect(updatedAgentInTable).toBeInTheDocument();
  });
});
