import { render, fireEvent, screen } from "@testing-library/svelte";
import StepConfig from "./StepConfig.svelte";

const stepMock = {
  name: "My Step",
  step_type: "prompt",
  step_content: "Hello, world!",
  agent_id: 1,
  id: "new",
};

const agents = [
  { id: 1, name: "Agent Smith" },
  { id: 2, name: "Agent Johnson" },
];

const stepTypes = ["prompt", "python"];

const t = {
  render: (stepOverrides = {}) => {
    const step = { ...stepMock, ...stepOverrides };

    return render(StepConfig, {
      props: { step, stepTypes, agents },
    });
  },
};

describe("StepConfig.test.ts - StepConfig Component", () => {
  it("renders step name input with correct value", () => {
    t.render();
    const nameInput = screen.getByLabelText("Step Name") as HTMLInputElement;
    expect(nameInput.value).toBe("My Step");
  });

  it("renders agent select and updates value", async () => {
    t.render();

    const select = screen.getByLabelText(
      "Associated Agent",
    ) as HTMLSelectElement;
    expect(select.value).toBe("1");

    await fireEvent.change(select, { target: { value: "2" } });
    expect(select.value).toBe("2");
  });

  it("updates step type from Prompt to Python", async () => {
    t.render();

    const typeSelect = screen.getByLabelText("Step Type") as HTMLSelectElement;
    expect(typeSelect.value).toBe("prompt");

    await fireEvent.change(typeSelect, { target: { value: "python" } });
    expect(typeSelect.value).toBe("python");
  });

  it("renders textarea for prompt type", () => {
    t.render();
    const textarea = screen.getByLabelText(
      "Prompt Template",
    ) as HTMLTextAreaElement;
    expect(textarea.value).toContain("Hello, world!");
  });

  it("renders code editor container for Python type", () => {
    const { container } = t.render({ step_type: "python" });
    const editorContainer = container.querySelector(".cm-editor");
    expect(editorContainer).toBeTruthy();
  });
});
