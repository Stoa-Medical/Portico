import { render, fireEvent, screen } from "@testing-library/svelte";
import PageHeader from "./PageHeader.svelte";
import { PlusOutline, TrashBinOutline } from "flowbite-svelte-icons";

const breadcrumbs = [
  { label: "Home", url: "/" },
  { label: "Agents", url: "/agents" },
  { label: "Agent Detail", url: "/agents/1" },
];

const actions = [
  {
    label: "Add Agent",
    onClick: vi.fn(),
    icon: PlusOutline,
    color: "blue",
  },
  {
    label: "Delete",
    onClick: vi.fn(),
    icon: TrashBinOutline,
    color: "red",
  },
];

const defaultTitle = "Agenti";

const t = {
  render: () => {
    return render(PageHeader, {
      title: defaultTitle,
      breadcrumbs,
      actionBar: actions,
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

    const [addAgentAction, deleteAction] = actions;
    expect(addAgentAction.onClick).toHaveBeenCalledTimes(1);
    expect(deleteAction.onClick).toHaveBeenCalledTimes(1);
  });

  it("renders buttons with the correct styles", () => {
    t.render();

    const addButton = screen.getByText("Add Agent");
    const deleteButton = screen.getByText("Delete");

    expect(addButton).toHaveClass("bg-blue-500");
    expect(deleteButton).toHaveClass("bg-red-500");
  });

  it("renders a disabled button correctly", () => {
    const disabledActions = [
      {
        label: "Disabled Action",
        onClick: vi.fn(),
        icon: PlusOutline,
        color: "blue",
        disabled: true,
      },
    ];

    render(PageHeader, {
      title: defaultTitle,
      breadcrumbs,
      actionBar: disabledActions,
    });

    const disabledButton = screen.getByText("Disabled Action");

    // Button is visually styled and disabled
    expect(disabledButton).toHaveClass("bg-gray-400");
    expect(disabledButton).toHaveClass("cursor-not-allowed");
    expect(disabledButton).toBeDisabled();
  });
});
