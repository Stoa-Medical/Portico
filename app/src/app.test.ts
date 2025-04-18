import { render, screen, within } from "@testing-library/svelte";
import Routes from "./routes/+page.svelte";

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

    const welcomeText = await screen.findByText(/Hi there, Howard!/i);

    expect(welcomeText).toBeInTheDocument();
  });

  test("shows `Home` page location in breadcrumb", async () => {
    t.render();

    const homeBreadcrumb = t.findBreadcrumb("Home");

    expect(homeBreadcrumb).toBeInTheDocument();
    expect(homeBreadcrumb).toHaveAttribute("href", "/");
  });
});
