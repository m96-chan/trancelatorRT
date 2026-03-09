import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RecordButton } from "./RecordButton";

describe("RecordButton", () => {
  it("shows Start when idle", () => {
    render(
      <RecordButton pipelineState="Idle" onStart={() => {}} onStop={() => {}} />,
    );
    expect(screen.getByText("Start")).toBeInTheDocument();
  });

  it("shows Stop when recording", () => {
    render(
      <RecordButton pipelineState="Recording" onStart={() => {}} onStop={() => {}} />,
    );
    expect(screen.getByText("Stop")).toBeInTheDocument();
  });

  it("shows Processing when processing", () => {
    render(
      <RecordButton pipelineState="Processing" onStart={() => {}} onStop={() => {}} />,
    );
    expect(screen.getByText("Processing...")).toBeInTheDocument();
  });

  it("calls onStart when idle and clicked", () => {
    const onStart = vi.fn();
    render(
      <RecordButton pipelineState="Idle" onStart={onStart} onStop={() => {}} />,
    );
    fireEvent.click(screen.getByLabelText("Start recording"));
    expect(onStart).toHaveBeenCalled();
  });

  it("calls onStop when recording and clicked", () => {
    const onStop = vi.fn();
    render(
      <RecordButton pipelineState="Recording" onStart={() => {}} onStop={onStop} />,
    );
    fireEvent.click(screen.getByLabelText("Stop recording"));
    expect(onStop).toHaveBeenCalled();
  });

  it("is disabled when processing", () => {
    render(
      <RecordButton pipelineState="Processing" onStart={() => {}} onStop={() => {}} />,
    );
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("has recording class when recording", () => {
    render(
      <RecordButton pipelineState="Recording" onStart={() => {}} onStop={() => {}} />,
    );
    expect(screen.getByRole("button")).toHaveClass("recording");
  });
});
