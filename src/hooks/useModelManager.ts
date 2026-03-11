import { useState, useCallback, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ModelStatusInfo, StorageInfo } from "../types";

interface TauriInvoke {
  (cmd: string, args?: Record<string, unknown>): Promise<unknown>;
}

interface DownloadProgress {
  id: string;
  downloaded: number;
  total: number;
}

interface DownloadComplete {
  id: string;
  success: boolean;
  error: string | null;
}

export interface ModelManagerState {
  models: ModelStatusInfo[];
  storageInfo: StorageInfo | null;
  loading: boolean;
  downloading: Record<string, number>;
  error: string | null;
}

export interface ModelManagerActions {
  refreshModels: () => Promise<void>;
  downloadModel: (id: string) => Promise<void>;
  deleteModel: (id: string) => Promise<void>;
}

export function useModelManager(
  invoke: TauriInvoke,
): [ModelManagerState, ModelManagerActions] {
  const [models, setModels] = useState<ModelStatusInfo[]>([]);
  const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [downloading, setDownloading] = useState<Record<string, number>>({});
  const [error, setError] = useState<string | null>(null);

  const refreshModels = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const modelList = (await invoke("get_model_list")) as ModelStatusInfo[];
      const storage = (await invoke("get_storage_info")) as StorageInfo;
      setModels(modelList);
      setStorageInfo(storage);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [invoke]);

  // Listen for download progress and completion events
  useEffect(() => {
    const unlistenProgress = listen<DownloadProgress>(
      "download-progress",
      (event) => {
        const { id, downloaded, total } = event.payload;
        const percent = total > 0 ? Math.round((downloaded / total) * 100) : 0;
        setDownloading((prev) => ({ ...prev, [id]: percent }));
      },
    );
    const unlistenComplete = listen<DownloadComplete>(
      "download-complete",
      (event) => {
        const { id, success, error: err } = event.payload;
        setDownloading((prev) => {
          const next = { ...prev };
          delete next[id];
          return next;
        });
        if (!success && err) {
          setError(err);
        }
        refreshModels();
      },
    );
    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, [refreshModels]);

  const downloadModel = useCallback(
    async (id: string) => {
      try {
        setError(null);
        setDownloading((prev) => ({ ...prev, [id]: 0 }));
        // Returns immediately — download runs in background thread
        await invoke("download_model", { id });
      } catch (e) {
        setDownloading((prev) => {
          const next = { ...prev };
          delete next[id];
          return next;
        });
        setError(String(e));
      }
    },
    [invoke],
  );

  const deleteModel = useCallback(
    async (id: string) => {
      try {
        setError(null);
        await invoke("delete_model", { id });
        await refreshModels();
      } catch (e) {
        setError(String(e));
      }
    },
    [invoke, refreshModels],
  );

  useEffect(() => {
    refreshModels();
  }, [refreshModels]);

  return [
    { models, storageInfo, loading, downloading, error },
    { refreshModels, downloadModel, deleteModel },
  ];
}
