import { render, fireEvent, screen } from "@testing-library/svelte";
import PageHeader from "./PageHeader.svelte";
import {
  PlusOutline,
  TrashBinOutline,
  CalendarMonthOutline,
} from "flowbite-svelte-icons";
import { tick } from "svelte";

const breadcrumbs = [
  { label: "Home", url: "/" },
  { label: "Agents", url: "/agents" },
  { label: "Agent Detail", url: "/agents/1" },
];

const actions = [
  {
    type: "button",
    label: "Add Agent",
    onClick: vi.fn(),
    icon: PlusOutline,
    color: "blue",
  },
  {
    type: "button",
    label: "Delete",
    onClick: vi.fn(),
    icon: TrashBinOutline,
    color: "red",
  },
];

const defaultTitle = "Agenti";

const t = {
  render: (customActions = actions) => {
    return render(PageHeader, {
      title: defaultTitle,
      breadcrumbs,
      actionBar: customActions,
    });
  },
};

describe("PageHeader.test.ts - PageHeader Component", () => {
  it("renders title correctly", () => {
    t.render();
    expect(screen.getByText(defaultTitle)).toBeInTheDocument();
  });

  it("renders breadcrumbs correctly", () => {
    t.render();
    breadcrumbs.forEach((breadcrumb) => {
      expect(screen.getByText(breadcrumb.label)).toBeInTheDocument();
    });
  });

  it("renders action buttons correctly", () => {
    t.render();
    actions.forEach((action) => {
      expect(screen.getByText(action.label)).toBeInTheDocument();
    });
  });

  it("calls the correct action when a button is clicked", async () => {
    t.render();
    const addButton = screen.getByText("Add Agent");
    const deleteButton = screen.getByText("Delete");

    await fireEvent.click(addButton);
    await fireEvent.click(deleteButton);

    expect(actions[0].onClick).toHaveBeenCalledTimes(1);
    expect(actions[1].onClick).toHaveBeenCalledTimes(1);
  });

  it("renders buttons with the correct styles", () => {
    t.render();
    const addButton = screen.getByText("Add Agent");
    const deleteButton = screen.getByText("Delete");

    expect(addButton).toHaveClass("bg-sea");
    expect(deleteButton).toHaveClass("bg-[#CE5A5A]");
  });

  it("renders a disabled button correctly", () => {
    const disabledActions = [
      {
        type: "button",
        label: "Disabled Action",
        onClick: vi.fn(),
        icon: PlusOutline,
        color: "blue",
        disabled: true,
      },
    ];

    t.render(disabledActions);

    const disabledButton = screen.getByText("Disabled Action");
    expect(disabledButton).toHaveClass("bg-gray-400");
    expect(disabledButton).toHaveClass("cursor-not-allowed");
    expect(disabledButton).toBeDisabled();
  });

  it("renders a select dropdown with icon", async () => {
    const mockTimePeriods = [
      { value: "7d", name: "Last 7 days" },
      { value: "30d", name: "Last 30 days" },
    ];

    let selectedValue = "30d";
    const selectAction = [
      {
        type: "select",
        icon: CalendarMonthOutline,
        value: selectedValue,
        options: mockTimePeriods,
      },
    ];

    t.render(selectAction);

    // Wait for DOM updates
    await tick();

    // Check dropdown presence
    const select = screen.getByDisplayValue("Last 30 days");
    expect(select).toBeInTheDocument();

    // Check that all options exist
    mockTimePeriods.forEach((period) => {
      expect(screen.getByText(period.name)).toBeInTheDocument();
    });

    // Check for icon by role or class (since it's a component)
    const icon = document.querySelector("svg");
    expect(icon).toBeInTheDocument();
  });
});
