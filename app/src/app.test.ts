import { render, screen } from "@testing-library/svelte";
import Routes from "./routes/+page.svelte";

test("test environment is using jsdom", () => {
  expect(window).toBeDefined();
  expect(document).toBeDefined();
});

test("renders component correctly", async () => {
  render(Routes);

  const welcomeText = await screen.findByText(/Welcome to the application/i);
  expect(welcomeText).toBeInTheDocument();
});
