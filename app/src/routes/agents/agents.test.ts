import { render, within, screen, fireEvent } from "@testing-library/svelte";
import AgentsPage from "./+page.svelte";

// Mocking the `api.js` file for data access
function getMockAgents() {
  return [
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
  ];
}

function getMockSteps() {
  return [
    {
      id: 101,
      name: "Data Collection",
      type: "Prompt",
      agentId: 1,
      content: "Collect all relevant data",
      isActive: true,
      lastEdited: "1 hour ago",
      createdAt: "2025-04-01",
    },
  ];
}

vi.mock("./api", async () => {
  const originalModule = await vi.importActual("./api");
  return {
    getAgents: vi.fn(() => Promise.resolve(getMockAgents())),
    getSteps: vi.fn(() => Promise.resolve(getMockSteps())),
    deleteAgent: originalModule.deleteAgent,
    saveAgent: originalModule.saveAgent,
  };
});

const t = {
  render: () => render(AgentsPage),
  findInTable: async ({
    text,
    tableId = "agents-table",
  }: {
    text: string;
    tableId?: string;
  }) => {
    const table = await screen.getByTestId(tableId);
    return within(table).getByText(text);
  },
  clickOnAgent: async (agentName: string = "Test Agent 1") => {
    const agent = await screen.findByText(agentName);
    fireEvent.click(agent);

    // Wait for agent panel to open (Check for "General" tab)
    await screen.findByText("General");
  },
  clickAgentTab: async (tabName: string) => {
    const tab = await screen.findByText(tabName);
    fireEvent.click(tab);
  },
};

describe("agents.test.ts - Agents Page", () => {
  it("renders the list of agents", async () => {
    t.render();

    // Then agents are fetched and rendered
    const agent1 = await screen.findByText("Test Agent 1");
    const agent2 = await screen.findByText("Test Agent 2");

    expect(agent1).toBeInTheDocument();
    expect(agent2).toBeInTheDocument();
  });

  it("allows a user to delete an existing agent from the list", async () => {
    t.render();

    // When you click on "Test Agent 1"
    await t.clickOnAgent();

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
    const newAgentInTable = await t.findInTable({ text: "New Agent Test" });

    expect(newAgentInTable).toBeInTheDocument();
  });

  it("allows a user to edit an existing agent", async () => {
    t.render();

    // When you click on "Test Agent 1"
    await t.clickOnAgent();

    // Then update the agent name
    const nameInput = screen.getByLabelText("Agent Name");
    fireEvent.input(nameInput, {
      target: { value: "Test Updated Agent Name" },
    });

    // After saving the changes
    const saveButton = screen.getByText("Save Changes");
    fireEvent.click(saveButton);

    // Then the updated name will appear in the table
    const updatedAgentInTable = await t.findInTable({
      text: "Test Updated Agent Name",
    });
    expect(updatedAgentInTable).toBeInTheDocument();
  });

  it("allows a user to view an existing agents step configurations", async () => {
    vi.mocked(await import("./api")).getSteps.mockReturnValue([
      {
        id: 101,
        name: "My First Step",
        type: "Prompt",
        agentId: 1,
        content: "Step content here",
        lastEdited: "1 hour ago",
      },
    ]);

    t.render();

    // When clicking on the steps tab for an existing agent
    await t.clickOnAgent();
    await t.clickAgentTab("Steps");

    // Then clicking on a specific step
    const viewButton = await screen.findByText("View");
    await fireEvent.click(viewButton);

    // Then the step configuration is shown with the correct values
    const stepNameInput = await screen.findByLabelText("Step Name");
    expect(stepNameInput).toBeInTheDocument();
    expect((stepNameInput as HTMLInputElement).value).toBe("My First Step");

    const typeSelect = (await screen.findByLabelText(
      "Step Type"
    )) as HTMLSelectElement;
    expect(typeSelect).toBeInTheDocument();
    expect(typeSelect.value).toBe("Prompt");

    const contentArea = await screen.findByLabelText("Prompt Template");
    expect(contentArea).toBeInTheDocument();
    expect((contentArea as HTMLTextAreaElement).value).toContain(
      "Step content here"
    );
  });

  it.skip("allows a user to add steps an existing agent", async () => {
    t.render();

    // When you click on "Test Agent 1"
    await t.clickOnAgent();

    // Then open "Steps" tab
    await t.clickAgentTab("Steps");

    // Then click "Add step"
    const addStepButton = await screen.findByText("Add Step");
    fireEvent.click(addStepButton);

    // Then blick "Save"
    const saveButton = await screen.findByText("Save Step");
    fireEvent.click(saveButton);

    // Then the new step should show inside of the agents step list
    const updatedAgentInTable = await t.findInTable({
      text: "Test Updated Agent Name",
      tableId: "steps-table",
    });
    expect(updatedAgentInTable).toBeInTheDocument();
  });
});
