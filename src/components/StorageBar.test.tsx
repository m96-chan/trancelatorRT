import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { StorageBar } from "./StorageBar";

describe("StorageBar", () => {
  it("shows loading when no storage info", () => {
    render(<StorageBar storageInfo={null} />);
    expect(screen.getByTestId("storage-bar")).toHaveTextContent("Loading...");
  });

  it("shows storage info when available", () => {
    render(
      <StorageBar
        storageInfo={{
          total_bytes: 64_000_000_000,
          available_bytes: 10_000_000_000,
          models_bytes: 150_000_000,
        }}
      />,
    );
    expect(screen.getByText(/Models:/)).toBeInTheDocument();
    expect(screen.getByText(/Free:/)).toBeInTheDocument();
  });

  it("renders storage fill bar", () => {
    render(
      <StorageBar
        storageInfo={{
          total_bytes: 64_000_000_000,
          available_bytes: 10_000_000_000,
          models_bytes: 150_000_000,
        }}
      />,
    );
    expect(screen.getByTestId("storage-fill")).toBeInTheDocument();
  });
});
