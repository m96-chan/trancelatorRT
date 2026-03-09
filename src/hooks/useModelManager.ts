import { useState, useCallback, useEffect } from "react";
import type { ModelStatusInfo, StorageInfo } from "../types";

interface TauriInvoke {
  (cmd: string, args?: Record<string, unknown>): Promise<unknown>;
}

export interface ModelManagerState {
  models: ModelStatusInfo[];
  storageInfo: StorageInfo | null;
  loading: boolean;
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

  const downloadModel = useCallback(
    async (id: string) => {
      try {
        setError(null);
        await invoke("download_model", { id });
        await refreshModels();
      } catch (e) {
        setError(String(e));
      }
    },
    [invoke, refreshModels],
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
    { models, storageInfo, loading, error },
    { refreshModels, downloadModel, deleteModel },
  ];
}
