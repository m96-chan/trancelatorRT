import { renderHook, act, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useModelManager } from "./useModelManager";
import type { ModelStatusInfo, StorageInfo } from "../types";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const mockModels: ModelStatusInfo[] = [
  {
    info: {
      model_type: "Whisper",
      id: "whisper-tiny",
      display_name: "Whisper Tiny",
      version: "1.0.0",
      url: "https://example.com",
      size_bytes: 75_000_000,
      sha256: "abc",
      filename: "ggml-tiny.bin",
    },
    status: "NotDownloaded",
    local_path: null,
  },
];

const mockStorage: StorageInfo = {
  total_bytes: 64_000_000_000,
  available_bytes: 10_000_000_000,
  models_bytes: 0,
};

describe("useModelManager", () => {
  let mockInvoke: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockInvoke = vi.fn((cmd: string) => {
      if (cmd === "get_model_list") return Promise.resolve(mockModels);
      if (cmd === "get_storage_info") return Promise.resolve(mockStorage);
      return Promise.resolve();
    });
  });

  it("fetches models on mount", async () => {
    const { result } = renderHook(() => useModelManager(mockInvoke));
    await waitFor(() => {
      expect(result.current[0].models.length).toBe(1);
    });
    expect(mockInvoke).toHaveBeenCalledWith("get_model_list");
  });

  it("fetches storage info on mount", async () => {
    const { result } = renderHook(() => useModelManager(mockInvoke));
    await waitFor(() => {
      expect(result.current[0].storageInfo).toEqual(mockStorage);
    });
  });

  it("downloadModel calls invoke and refreshes", async () => {
    const { result } = renderHook(() => useModelManager(mockInvoke));
    await waitFor(() => expect(result.current[0].loading).toBe(false));

    await act(async () => {
      await result.current[1].downloadModel("whisper-tiny");
    });

    expect(mockInvoke).toHaveBeenCalledWith("download_model", {
      id: "whisper-tiny",
    });
  });

  it("deleteModel calls invoke and refreshes", async () => {
    const { result } = renderHook(() => useModelManager(mockInvoke));
    await waitFor(() => expect(result.current[0].loading).toBe(false));

    await act(async () => {
      await result.current[1].deleteModel("whisper-tiny");
    });

    expect(mockInvoke).toHaveBeenCalledWith("delete_model", {
      id: "whisper-tiny",
    });
  });

  it("sets error on failure", async () => {
    mockInvoke = vi.fn(() => Promise.reject("Network error"));
    const { result } = renderHook(() => useModelManager(mockInvoke));
    await waitFor(() => {
      expect(result.current[0].error).toBe("Network error");
    });
  });

  it("initializes downloading state as empty", async () => {
    const { result } = renderHook(() => useModelManager(mockInvoke));
    expect(result.current[0].downloading).toEqual({});
  });
});
