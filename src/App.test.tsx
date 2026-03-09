import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve()),
}));

import { invoke } from "@tauri-apps/api/core";
const mockInvoke = vi.mocked(invoke);

describe("App", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
    mockInvoke.mockResolvedValue(undefined as never);
  });

  it("renders the app title", () => {
    render(<App />);
    expect(screen.getByText("trancelatorRT")).toBeInTheDocument();
  });

  it("renders language selectors", () => {
    render(<App />);
    expect(screen.getByLabelText("Source")).toBeInTheDocument();
    expect(screen.getByLabelText("Target")).toBeInTheDocument();
  });

  it("renders record button", () => {
    render(<App />);
    expect(screen.getByLabelText("Start recording")).toBeInTheDocument();
  });

  it("renders text display panels", () => {
    render(<App />);
    expect(screen.getByTestId("text-recognized")).toBeInTheDocument();
    expect(screen.getByTestId("text-translated")).toBeInTheDocument();
  });

  it("renders pipeline status", () => {
    render(<App />);
    expect(screen.getByTestId("pipeline-status")).toHaveTextContent("Ready");
  });

  it("renders language hint with defaults", () => {
    render(<App />);
    expect(screen.getByText(/English → 日本語/)).toBeInTheDocument();
  });

  it("renders swap button", () => {
    render(<App />);
    expect(screen.getByLabelText("Swap languages")).toBeInTheDocument();
  });

  it("default source language is English", () => {
    render(<App />);
    const source = screen.getByLabelText("Source") as HTMLSelectElement;
    expect(source.value).toBe("en");
  });

  it("default target language is Japanese", () => {
    render(<App />);
    const target = screen.getByLabelText("Target") as HTMLSelectElement;
    expect(target.value).toBe("ja");
  });
});
