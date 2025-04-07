import { render, screen, within, fireEvent } from "@testing-library/svelte";
import Routes from "./routes/+page.svelte";

test("test environment is using jsdom", () => {
  expect(window).toBeDefined();
  expect(document).toBeDefined();
});

test("initializes application", async () => {
  render(Routes);

  const welcomeText = await screen.findByText(/Welcome to the application/i);

  expect(welcomeText).toBeInTheDocument();
});

test("shows `Home` page location in breadcrumb", async () => {
  render(Routes);

  const breadcrumbNav = screen.getByRole("navigation", { name: /breadcrumb/i });
  const homeBreadcrumb = within(breadcrumbNav).getByText("Home");

  expect(homeBreadcrumb).toBeInTheDocument();
  expect(homeBreadcrumb).toHaveAttribute("href", "/");
});
