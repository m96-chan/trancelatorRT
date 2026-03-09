import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { TextDisplay } from "./TextDisplay";

describe("TextDisplay", () => {
  it("renders label", () => {
    render(<TextDisplay label="Recognized" text="" language="en" />);
    expect(screen.getByText("Recognized")).toBeInTheDocument();
  });

  it("renders text content", () => {
    render(<TextDisplay label="Recognized" text="Hello world" language="en" />);
    expect(screen.getByText("Hello world")).toBeInTheDocument();
  });

  it("renders placeholder when empty", () => {
    render(<TextDisplay label="Recognized" text="" language="en" />);
    expect(screen.getByText("---")).toBeInTheDocument();
  });

  it("sets lang attribute", () => {
    const { container } = render(
      <TextDisplay label="Translated" text="こんにちは" language="ja" />,
    );
    expect(container.querySelector("[lang='ja']")).toBeInTheDocument();
  });

  it("uses testid based on label", () => {
    render(<TextDisplay label="Recognized" text="test" language="en" />);
    expect(screen.getByTestId("text-recognized")).toHaveTextContent("test");
  });
});
