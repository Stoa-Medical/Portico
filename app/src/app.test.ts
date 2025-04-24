import { render, screen, within } from "@testing-library/svelte";
import Routes from "./routes/+page.svelte";

vi.mock("$lib/supabase", () => ({
  default: {
    from: () => ({
      select: vi.fn().mockReturnThis(),
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

const t = {
  render: () => render(Routes),
  findBreadcrumb: (name) => {
    const breadcrumbNav = screen.getByRole("navigation", {
      name: /breadcrumb/i,
    });
    return within(breadcrumbNav).getByText(name);
  },
};

describe("app.test.ts", () => {
  test("test environment is using jsdom", () => {
    expect(window).toBeDefined();
    expect(document).toBeDefined();
  });

  test("initializes application", async () => {
    t.render();

    const welcomeText = await screen.findByText(
      /Get started on something new/i,
    );
    const systemStatusText = await screen.findByText(/System Status/i);

    expect(welcomeText).toBeInTheDocument();
    expect(systemStatusText).toBeInTheDocument();
  });

  test("shows `Home` page location in breadcrumb", async () => {
    t.render();

    const homeBreadcrumb = t.findBreadcrumb("Home");

    expect(homeBreadcrumb).toBeInTheDocument();
    expect(homeBreadcrumb).toHaveAttribute("href", "/");
  });
});
