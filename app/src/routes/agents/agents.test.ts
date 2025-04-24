import {
  render,
  within,
  screen,
  fireEvent,
  waitForElementToBeRemoved,
} from "@testing-library/svelte";
import AgentsPage from "./+page.svelte";

beforeEach(() => {
  window.history.pushState({}, "", "/agents"); // Reset to base state
});

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
      step_type: "prompt",
      agent_id: 1,
      content: "Collect all relevant data",
    },
  ];
}

function getMockRuntimeSessions() {
  return [
    {
      id: 201,
      created_at: new Date("2025-04-08T10:00:00Z").toISOString(),
      updated_at: new Date("2025-04-08T10:15:00Z").toISOString(),
      rts_status: "completed",
      initial_data: { input: "Example input" },
      latest_step_idx: 3,
      latest_result: { output: "Final result" },
    },
  ];
}

export const deleteEqMock = vi.fn(() => ({ data: null, error: null }));
export const deleteMock = vi.fn(() => ({ eq: deleteEqMock }));
export const insertAgentMock = vi.fn(() => ({ data: null, error: null }));
export const updateEqMock = vi.fn(() => ({ data: null, error: null }));
export const updateAgentMock = vi.fn(() => ({
  eq: updateEqMock,
  data: null,
  error: null,
}));

vi.mock("$lib/supabase", () => {
  return {
    default: {
      from: vi.fn(() => ({
        select: vi.fn(() => ({ data: [], error: null })),
        insert: insertAgentMock,
        update: updateAgentMock,
        delete: deleteMock,
      })),
      auth: {
        getUser: vi.fn(() => ({
          data: { user: { id: "test-user-id", email: "test@example.com" } },
          error: null,
        })),
      },
    },
  };
});

vi.mock("./api", async () => {
  const originalModule = await vi.importActual("./api");
  return {
    getAgents: vi.fn(() => Promise.resolve(getMockAgents())),
    getSteps: vi.fn(() => Promise.resolve(getMockSteps())),
    getRuntimeSessions: vi.fn(() => Promise.resolve(getMockRuntimeSessions())),
    deleteAgent: originalModule.deleteAgent,
    saveAgent: originalModule.saveAgent,
    updateAgent: originalModule.updateAgent,
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
    const agentRow = await screen.findByTestId(`agent-row-${agentName}`);
    fireEvent.click(agentRow);

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

    // Then the agent remove API is called
    expect(deleteMock).toHaveBeenCalled();
    expect(deleteEqMock).toHaveBeenCalledWith("agent_id", 1);
  });

  it("allows a user to create a new agent", async () => {
    t.render();

    // When you click the 'New Agent' button in the action bar
    const addButton = await screen.findByText("New Agent");
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
    await fireEvent.click(submitButton);

    // Then the create agent API is called
    expect(insertAgentMock).toHaveBeenCalled();
    expect(insertAgentMock.mock.calls[0]).toMatchObject([
      [
        {
          agent_state: "inactive",
          description: "",
          name: "New Agent Test",
          type: "Assistant",
          owner_id: "test-user-id",
        },
      ],
    ]);
  });

  it("resets and closes the create agent modal when cancelled", async () => {
    t.render();

    // When you open the New Agent modal
    const addButton = await screen.findByText("New Agent");
    await fireEvent.click(addButton);

    // Then you enter a name
    const nameInput = await screen.findByPlaceholderText("Enter agent name");
    fireEvent.input(nameInput, { target: { value: "Temporary Agent" } });

    // When you click the Cancel button
    const cancelButton = await screen.findByText("Cancel");
    await fireEvent.click(cancelButton);

    // Then the modal should close
    expect(screen.queryByText("Add New Agent")).not.toBeInTheDocument();

    // When you open the modal again
    await fireEvent.click(addButton);

    // Then the form should be reset
    const reopenedInput =
      await screen.findByPlaceholderText("Enter agent name");
    expect((reopenedInput as HTMLInputElement).value).toBe("");
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

    // Then the update agent api should be called with the new agent name
    expect(updateAgentMock).toHaveBeenCalled();
    const updateArgs = updateAgentMock.mock.calls[0];

    expect(updateArgs).toMatchObject([
      expect.objectContaining({
        name: "Test Updated Agent Name",
      }),
    ]);
  });

  it("allows selecting and deselecting an agent by clicking the same row", async () => {
    t.render();

    // When you click on "Test Agent 1"
    await t.clickOnAgent();

    // Then the agent detail panel will display
    const generalTabEl = await screen.getByRole("tab", { name: /general/i });
    expect(generalTabEl).toBeInTheDocument();

    // When you click on "Test Agent 1" again
    await t.clickOnAgent();

    // The agent should no longer be selected
    expect(
      screen.queryByRole("tab", { name: /general/i }),
    ).not.toBeInTheDocument();
  });

  it("allows a user to view an existing agents step configurations", async () => {
    vi.mocked(await import("./api")).getSteps.mockReturnValue([
      {
        id: 101,
        name: "My First Step",
        step_type: "prompt",
        agent_id: 1,
        step_content: "Step content here",
      },
    ] as any);

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
      "Step Type",
    )) as HTMLSelectElement;
    expect(typeSelect).toBeInTheDocument();
    expect(typeSelect.value).toBe("prompt");

    const contentArea = await screen.findByLabelText("Prompt Template");
    expect(contentArea).toBeInTheDocument();
    expect((contentArea as HTMLTextAreaElement).value).toContain(
      "Step content here",
    );
  });

  it("allows a user to add steps an existing agent", async () => {
    t.render();

    // When you click on "Test Agent 1"
    await t.clickOnAgent();

    // Then open "Steps" tab
    await t.clickAgentTab("Steps");

    // Then click "Add step"
    const addStepButton = await screen.findByText("Add Step");
    fireEvent.click(addStepButton);
  });

  it("allows a user to view runtime sessions and see details", async () => {
    t.render();

    // When you click on an Agents "Runtime Sessions" tab
    await t.clickOnAgent();
    await t.clickAgentTab("Runtime Sessions");

    // Then the list of sessions should be displayed
    const sessionIdCell = await screen.findByText("201");
    expect(sessionIdCell).toBeInTheDocument();

    // When you click on an individual runtime session
    const viewButton = await screen.findByText("View");
    await fireEvent.click(viewButton);

    // Then you should see the details for that session
    expect(
      await screen.findByText((text) => text.includes("Example input")),
    ).toBeInTheDocument();

    const latestResultEl = await screen.findByText((text) =>
      text.includes("Final result"),
    );
    expect(latestResultEl).toBeInTheDocument();

    // When you click close
    const closeButton = await screen.findByText("Close");
    fireEvent.click(closeButton);

    // Them the detail view should disappear
    await waitForElementToBeRemoved(latestResultEl);
  });
});
