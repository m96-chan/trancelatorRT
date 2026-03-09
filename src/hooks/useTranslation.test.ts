import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { useTranslation } from "./useTranslation";

describe("useTranslation", () => {
  let mockInvoke: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    mockInvoke = vi.fn(() => Promise.resolve());
  });

  it("has correct default state", () => {
    const { result } = renderHook(() => useTranslation(mockInvoke));
    const [state] = result.current;
    expect(state.sourceLanguage).toBe("en");
    expect(state.targetLanguage).toBe("ja");
    expect(state.pipelineState).toBe("Idle");
    expect(state.transcriptionText).toBe("");
    expect(state.translationText).toBe("");
    expect(state.error).toBeNull();
  });

  it("setSourceLanguage updates state and calls invoke", () => {
    const { result } = renderHook(() => useTranslation(mockInvoke));
    act(() => {
      result.current[1].setSourceLanguage("fr");
    });
    expect(result.current[0].sourceLanguage).toBe("fr");
    expect(mockInvoke).toHaveBeenCalledWith("set_source_language", {
      code: "fr",
    });
  });

  it("setTargetLanguage updates state and calls invoke", () => {
    const { result } = renderHook(() => useTranslation(mockInvoke));
    act(() => {
      result.current[1].setTargetLanguage("de");
    });
    expect(result.current[0].targetLanguage).toBe("de");
    expect(mockInvoke).toHaveBeenCalledWith("set_target_language", {
      code: "de",
    });
  });

  it("startRecording calls invoke and sets Recording state", async () => {
    const { result } = renderHook(() => useTranslation(mockInvoke));
    await act(async () => {
      await result.current[1].startRecording();
    });
    expect(mockInvoke).toHaveBeenCalledWith("start_recording");
    expect(result.current[0].pipelineState).toBe("Recording");
  });

  it("stopRecording calls invoke and sets Idle state", async () => {
    const { result } = renderHook(() => useTranslation(mockInvoke));
    await act(async () => {
      await result.current[1].startRecording();
    });
    await act(async () => {
      await result.current[1].stopRecording();
    });
    expect(mockInvoke).toHaveBeenCalledWith("stop_recording");
    expect(result.current[0].pipelineState).toBe("Idle");
  });

  it("startRecording sets error on failure", async () => {
    mockInvoke.mockRejectedValueOnce("Audio device error");
    const { result } = renderHook(() => useTranslation(mockInvoke));
    await act(async () => {
      await result.current[1].startRecording();
    });
    expect(result.current[0].error).toBe("Audio device error");
  });

  it("clearTexts resets text and error", async () => {
    mockInvoke.mockRejectedValueOnce("error");
    const { result } = renderHook(() => useTranslation(mockInvoke));
    await act(async () => {
      await result.current[1].startRecording();
    });
    act(() => {
      result.current[1].clearTexts();
    });
    expect(result.current[0].transcriptionText).toBe("");
    expect(result.current[0].translationText).toBe("");
    expect(result.current[0].error).toBeNull();
  });

  it("swapLanguages swaps source and target", () => {
    const { result } = renderHook(() => useTranslation(mockInvoke));
    act(() => {
      result.current[1].swapLanguages();
    });
    expect(result.current[0].sourceLanguage).toBe("ja");
    expect(result.current[0].targetLanguage).toBe("en");
  });
});
