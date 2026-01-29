import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { StateCreator } from 'zustand';
import { api } from '@/lib/api';
import type {
  FileOperation,
  OperationResult,
  MergeOptions,
  MergePreview,
  FileOperationStatus,
  TrackedFileOperation,
  BulkOperationResult,
} from '@/types/file-ops';

// Default merge options
const DEFAULT_MERGE_OPTIONS: MergeOptions = {
  strategy: 'append',
  preserveHeaders: true,
  addSeparator: true,
  separator: '\n---\n',
};

interface FileOpsState {
  // Pending operations
  pendingOperations: TrackedFileOperation[];

  // Selection state
  selectedFiles: string[];
  selectionMode: 'single' | 'multi';

  // Merge preview
  mergePreview: MergePreview | null;
  mergeSource: string | null;
  mergeTarget: string | null;
  mergeOptions: MergeOptions;

  // UI State
  isExecuting: boolean;
  isPreviewLoading: boolean;
  showMergeDialog: boolean;
  error: string | null;

  // Actions - File Selection
  selectFile: (path: string) => void;
  deselectFile: (path: string) => void;
  toggleFileSelection: (path: string) => void;
  clearSelection: () => void;
  setSelectionMode: (mode: 'single' | 'multi') => void;

  // Actions - Operations
  queueOperation: (operation: FileOperation) => string;
  removeOperation: (operationId: string) => void;
  clearOperations: () => void;
  executeOperation: (operationId: string) => Promise<OperationResult>;
  executeAllOperations: (stopOnError?: boolean) => Promise<BulkOperationResult>;

  // Actions - Copy/Move/Delete
  copyFile: (source: string, destination: string, overwrite?: boolean) => Promise<OperationResult>;
  moveFile: (source: string, destination: string, overwrite?: boolean) => Promise<OperationResult>;
  deleteFile: (path: string, createBackup?: boolean) => Promise<OperationResult>;

  // Actions - Merge
  previewMerge: (source: string, target: string, options?: Partial<MergeOptions>) => Promise<void>;
  executeMerge: () => Promise<OperationResult | null>;
  setMergeOptions: (options: Partial<MergeOptions>) => void;
  openMergeDialog: (source: string, target: string) => void;
  closeMergeDialog: () => void;

  // Actions - Clone
  cloneSkill: (skillName: string, newName: string) => Promise<OperationResult>;
  cloneAgent: (agentId: number, newName: string) => Promise<OperationResult>;

  // Actions - Utilities
  updateOperationStatus: (operationId: string, status: FileOperationStatus, result?: OperationResult, error?: string) => void;
  clearError: () => void;
}

// Generate unique operation ID
const generateOperationId = (): string => {
  return `op_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
};

const fileOpsStore: StateCreator<
  FileOpsState,
  [],
  [['zustand/subscribeWithSelector', never]],
  FileOpsState
> = (set, get) => ({
  // Initial state
  pendingOperations: [],
  selectedFiles: [],
  selectionMode: 'single',
  mergePreview: null,
  mergeSource: null,
  mergeTarget: null,
  mergeOptions: DEFAULT_MERGE_OPTIONS,
  isExecuting: false,
  isPreviewLoading: false,
  showMergeDialog: false,
  error: null,

  // File Selection Actions
  selectFile: (path: string) => {
    const { selectionMode, selectedFiles } = get();

    if (selectionMode === 'single') {
      set({ selectedFiles: [path] });
    } else {
      if (!selectedFiles.includes(path)) {
        set({ selectedFiles: [...selectedFiles, path] });
      }
    }
  },

  deselectFile: (path: string) => {
    set((state) => ({
      selectedFiles: state.selectedFiles.filter((f) => f !== path),
    }));
  },

  toggleFileSelection: (path: string) => {
    const { selectedFiles, selectFile, deselectFile } = get();

    if (selectedFiles.includes(path)) {
      deselectFile(path);
    } else {
      selectFile(path);
    }
  },

  clearSelection: () => {
    set({ selectedFiles: [] });
  },

  setSelectionMode: (mode: 'single' | 'multi') => {
    set({ selectionMode: mode });

    // Clear selection when switching to single mode with multiple selections
    if (mode === 'single') {
      const { selectedFiles } = get();
      if (selectedFiles.length > 1) {
        set({ selectedFiles: selectedFiles.slice(0, 1) });
      }
    }
  },

  // Operation Queue Actions
  queueOperation: (operation: FileOperation): string => {
    const id = generateOperationId();
    const trackedOp: TrackedFileOperation = {
      id,
      operation,
      status: 'pending',
    };

    set((state) => ({
      pendingOperations: [...state.pendingOperations, trackedOp],
    }));

    return id;
  },

  removeOperation: (operationId: string) => {
    set((state) => ({
      pendingOperations: state.pendingOperations.filter((op) => op.id !== operationId),
    }));
  },

  clearOperations: () => {
    set({ pendingOperations: [] });
  },

  updateOperationStatus: (operationId: string, status: FileOperationStatus, result?: OperationResult, error?: string) => {
    set((state) => ({
      pendingOperations: state.pendingOperations.map((op) =>
        op.id === operationId
          ? {
              ...op,
              status,
              result,
              error,
              ...(status === 'running' ? { startedAt: new Date() } : {}),
              ...(status === 'success' || status === 'error' ? { completedAt: new Date() } : {}),
            }
          : op
      ),
    }));
  },

  executeOperation: async (operationId: string): Promise<OperationResult> => {
    const { pendingOperations, updateOperationStatus } = get();
    const trackedOp = pendingOperations.find((op) => op.id === operationId);

    if (!trackedOp) {
      return {
        success: false,
        message: 'Operation not found',
        affectedFiles: [],
      };
    }

    updateOperationStatus(operationId, 'running');

    try {
      const { operation } = trackedOp;
      let result: OperationResult;

      switch (operation.operationType) {
        case 'copy':
          result = await api.fileOpsCopy(operation.source, operation.destination, operation.overwrite);
          break;
        case 'move':
          result = await api.fileOpsMove(operation.source, operation.destination, operation.overwrite);
          break;
        case 'delete':
          result = await api.fileOpsDelete(operation.source);
          break;
        case 'merge':
          result = await api.fileOpsMergeMarkdown(operation.source, operation.destination);
          break;
        default:
          result = {
            success: false,
            message: `Unknown operation type: ${operation.operationType}`,
            affectedFiles: [],
          };
      }

      updateOperationStatus(operationId, result.success ? 'success' : 'error', result);
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Operation failed';
      updateOperationStatus(operationId, 'error', undefined, errorMessage);
      return {
        success: false,
        message: errorMessage,
        affectedFiles: [],
      };
    }
  },

  executeAllOperations: async (stopOnError = false): Promise<BulkOperationResult> => {
    const { pendingOperations, executeOperation } = get();
    const pendingOps = pendingOperations.filter((op) => op.status === 'pending');

    set({ isExecuting: true, error: null });

    const results: OperationResult[] = [];
    let successCount = 0;
    let failureCount = 0;

    for (const op of pendingOps) {
      const result = await executeOperation(op.id);
      results.push(result);

      if (result.success) {
        successCount++;
      } else {
        failureCount++;
        if (stopOnError) {
          break;
        }
      }
    }

    set({ isExecuting: false });

    return {
      results,
      totalOperations: pendingOps.length,
      successCount,
      failureCount,
    };
  },

  // Direct Operations
  copyFile: async (source: string, destination: string, overwrite = false): Promise<OperationResult> => {
    set({ isExecuting: true, error: null });

    try {
      const result = await api.fileOpsCopy(source, destination, overwrite);
      set({ isExecuting: false });
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Copy failed';
      set({ isExecuting: false, error: errorMessage });
      return {
        success: false,
        message: errorMessage,
        affectedFiles: [],
      };
    }
  },

  moveFile: async (source: string, destination: string, overwrite = false): Promise<OperationResult> => {
    set({ isExecuting: true, error: null });

    try {
      const result = await api.fileOpsMove(source, destination, overwrite);
      set({ isExecuting: false });
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Move failed';
      set({ isExecuting: false, error: errorMessage });
      return {
        success: false,
        message: errorMessage,
        affectedFiles: [],
      };
    }
  },

  deleteFile: async (path: string, createBackup = true): Promise<OperationResult> => {
    set({ isExecuting: true, error: null });

    try {
      const result = await api.fileOpsDelete(path, createBackup);
      set({ isExecuting: false });
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Delete failed';
      set({ isExecuting: false, error: errorMessage });
      return {
        success: false,
        message: errorMessage,
        affectedFiles: [],
      };
    }
  },

  // Merge Operations
  previewMerge: async (source: string, target: string, options?: Partial<MergeOptions>) => {
    const mergeOpts = { ...get().mergeOptions, ...options };

    set({
      isPreviewLoading: true,
      error: null,
      mergeSource: source,
      mergeTarget: target,
      mergeOptions: mergeOpts,
    });

    try {
      const preview = await api.fileOpsPreviewMerge(source, target, mergeOpts);
      set({
        mergePreview: preview,
        isPreviewLoading: false,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Preview failed';
      set({
        isPreviewLoading: false,
        error: errorMessage,
        mergePreview: null,
      });
    }
  },

  executeMerge: async (): Promise<OperationResult | null> => {
    const { mergeSource, mergeTarget, mergeOptions } = get();

    if (!mergeSource || !mergeTarget) {
      return null;
    }

    set({ isExecuting: true, error: null });

    try {
      const result = await api.fileOpsMergeMarkdown(mergeSource, mergeTarget, mergeOptions);
      set({
        isExecuting: false,
        showMergeDialog: false,
        mergePreview: null,
        mergeSource: null,
        mergeTarget: null,
      });
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Merge failed';
      set({
        isExecuting: false,
        error: errorMessage,
      });
      return {
        success: false,
        message: errorMessage,
        affectedFiles: [],
      };
    }
  },

  setMergeOptions: (options: Partial<MergeOptions>) => {
    set((state) => ({
      mergeOptions: { ...state.mergeOptions, ...options },
    }));
  },

  openMergeDialog: (source: string, target: string) => {
    set({
      showMergeDialog: true,
      mergeSource: source,
      mergeTarget: target,
    });
    // Auto-trigger preview
    get().previewMerge(source, target);
  },

  closeMergeDialog: () => {
    set({
      showMergeDialog: false,
      mergePreview: null,
      mergeSource: null,
      mergeTarget: null,
      error: null,
    });
  },

  // Clone Operations
  cloneSkill: async (skillName: string, newName: string): Promise<OperationResult> => {
    set({ isExecuting: true, error: null });

    try {
      const result = await api.fileOpsCloneSkill(skillName, newName);
      set({ isExecuting: false });
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Clone skill failed';
      set({ isExecuting: false, error: errorMessage });
      return {
        success: false,
        message: errorMessage,
        affectedFiles: [],
      };
    }
  },

  cloneAgent: async (agentId: number, newName: string): Promise<OperationResult> => {
    set({ isExecuting: true, error: null });

    try {
      const result = await api.fileOpsCloneAgent(agentId, newName);
      set({ isExecuting: false });
      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Clone agent failed';
      set({ isExecuting: false, error: errorMessage });
      return {
        success: false,
        message: errorMessage,
        affectedFiles: [],
      };
    }
  },

  // Utilities
  clearError: () => {
    set({ error: null });
  },
});

export const useFileOpsStore = create<FileOpsState>()(
  subscribeWithSelector(fileOpsStore)
);

// Selector hooks for common use cases
export const useSelectedFiles = () => useFileOpsStore((state) => state.selectedFiles);
export const usePendingOperations = () => useFileOpsStore((state) => state.pendingOperations);
export const useFileOpsExecuting = () => useFileOpsStore((state) => state.isExecuting);
export const useFileOpsError = () => useFileOpsStore((state) => state.error);
export const useMergePreview = () => useFileOpsStore((state) => state.mergePreview);
export const useMergeDialogOpen = () => useFileOpsStore((state) => state.showMergeDialog);
export const usePendingCount = () => useFileOpsStore((state) =>
  state.pendingOperations.filter((op) => op.status === 'pending').length
);
