import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { LanguageSelector } from "./LanguageSelector";

describe("LanguageSelector", () => {
  it("renders with label", () => {
    render(
      <LanguageSelector label="Source" value="en" onChange={() => {}} />,
    );
    expect(screen.getByLabelText("Source")).toBeInTheDocument();
  });

  it("shows selected value", () => {
    render(
      <LanguageSelector label="Source" value="ja" onChange={() => {}} />,
    );
    const select = screen.getByLabelText("Source") as HTMLSelectElement;
    expect(select.value).toBe("ja");
  });

  it("renders all 8 languages", () => {
    render(
      <LanguageSelector label="Source" value="en" onChange={() => {}} />,
    );
    const select = screen.getByLabelText("Source") as HTMLSelectElement;
    expect(select.options.length).toBe(8);
  });

  it("calls onChange when selection changes", () => {
    const onChange = vi.fn();
    render(
      <LanguageSelector label="Source" value="en" onChange={onChange} />,
    );
    fireEvent.change(screen.getByLabelText("Source"), {
      target: { value: "ja" },
    });
    expect(onChange).toHaveBeenCalledWith("ja");
  });

  it("can be disabled", () => {
    render(
      <LanguageSelector
        label="Source"
        value="en"
        onChange={() => {}}
        disabled
      />,
    );
    expect(screen.getByLabelText("Source")).toBeDisabled();
  });

  it("shows native language names", () => {
    render(
      <LanguageSelector label="Source" value="en" onChange={() => {}} />,
    );
    expect(screen.getByText(/日本語/)).toBeInTheDocument();
    expect(screen.getByText(/English/)).toBeInTheDocument();
  });
});
