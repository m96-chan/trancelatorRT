import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ModelCard } from "./ModelCard";
import type { ModelStatusInfo } from "../types";

function makeModel(overrides: Partial<ModelStatusInfo> = {}): ModelStatusInfo {
  return {
    info: {
      model_type: "Whisper",
      id: "whisper-tiny",
      display_name: "Whisper Tiny",
      version: "1.0.0",
      url: "https://example.com/model.bin",
      size_bytes: 75_000_000,
      sha256: "abc123",
      filename: "ggml-tiny.bin",
    },
    status: "NotDownloaded",
    local_path: null,
    ...overrides,
  };
}

describe("ModelCard", () => {
  it("shows model name and type", () => {
    render(
      <ModelCard model={makeModel()} onDownload={() => {}} onDelete={() => {}} />,
    );
    expect(screen.getByText("Whisper Tiny")).toBeInTheDocument();
    expect(screen.getByText("Whisper")).toBeInTheDocument();
  });

  it("shows download button when not downloaded", () => {
    render(
      <ModelCard model={makeModel()} onDownload={() => {}} onDelete={() => {}} />,
    );
    expect(screen.getByLabelText("Download Whisper Tiny")).toBeInTheDocument();
  });

  it("shows delete button when downloaded", () => {
    render(
      <ModelCard
        model={makeModel({ status: "Downloaded" })}
        onDownload={() => {}}
        onDelete={() => {}}
      />,
    );
    expect(screen.getByLabelText("Delete Whisper Tiny")).toBeInTheDocument();
  });

  it("shows progress bar when downloading", () => {
    render(
      <ModelCard
        model={makeModel({ status: { Downloading: { progress_percent: 42 } } })}
        onDownload={() => {}}
        onDelete={() => {}}
      />,
    );
    expect(screen.getByRole("progressbar")).toBeInTheDocument();
  });

  it("calls onDownload when download clicked", () => {
    const onDownload = vi.fn();
    render(
      <ModelCard model={makeModel()} onDownload={onDownload} onDelete={() => {}} />,
    );
    fireEvent.click(screen.getByLabelText("Download Whisper Tiny"));
    expect(onDownload).toHaveBeenCalledWith("whisper-tiny");
  });

  it("calls onDelete when delete clicked", () => {
    const onDelete = vi.fn();
    render(
      <ModelCard
        model={makeModel({ status: "Downloaded" })}
        onDownload={() => {}}
        onDelete={onDelete}
      />,
    );
    fireEvent.click(screen.getByLabelText("Delete Whisper Tiny"));
    expect(onDelete).toHaveBeenCalledWith("whisper-tiny");
  });

  it("shows size formatted", () => {
    render(
      <ModelCard model={makeModel()} onDownload={() => {}} onDelete={() => {}} />,
    );
    expect(screen.getByText("71.5 MB")).toBeInTheDocument();
  });

  it("shows status text", () => {
    render(
      <ModelCard model={makeModel()} onDownload={() => {}} onDelete={() => {}} />,
    );
    expect(screen.getByTestId("model-status-whisper-tiny")).toHaveTextContent(
      "Not Downloaded",
    );
  });
});
