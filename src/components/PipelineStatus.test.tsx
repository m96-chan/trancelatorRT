import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { PipelineStatus } from "./PipelineStatus";

describe("PipelineStatus", () => {
  it("shows Ready when idle", () => {
    render(<PipelineStatus state="Idle" error={null} />);
    expect(screen.getByTestId("pipeline-status")).toHaveTextContent("Ready");
  });

  it("shows Listening when recording", () => {
    render(<PipelineStatus state="Recording" error={null} />);
    expect(screen.getByTestId("pipeline-status")).toHaveTextContent("Listening...");
  });

  it("shows Paused when paused", () => {
    render(<PipelineStatus state="Paused" error={null} />);
    expect(screen.getByTestId("pipeline-status")).toHaveTextContent("Paused");
  });

  it("shows Processing when processing", () => {
    render(<PipelineStatus state="Processing" error={null} />);
    expect(screen.getByTestId("pipeline-status")).toHaveTextContent("Processing...");
  });

  it("shows error message", () => {
    render(<PipelineStatus state="Idle" error="Something went wrong" />);
    expect(screen.getByRole("alert")).toHaveTextContent("Something went wrong");
  });

  it("does not show error when null", () => {
    render(<PipelineStatus state="Idle" error={null} />);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("has correct class for recording", () => {
    render(<PipelineStatus state="Recording" error={null} />);
    expect(screen.getByTestId("pipeline-status")).toHaveClass("status-recording");
  });
});
