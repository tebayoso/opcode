/**
 * File Operations Types
 * Types for file copy, move, merge, and bulk operations
 */

/** Operation type for file actions */
export type OperationType = 'copy' | 'move' | 'merge' | 'delete';

/** Strategy for merging markdown files */
export type MergeStrategy = 'append' | 'prepend' | 'section-based' | 'custom';

/** A single file operation definition */
export interface FileOperation {
  source: string;
  destination: string;
  operationType: OperationType;
  overwrite?: boolean;
}

/** Options for merge operations */
export interface MergeOptions {
  strategy: MergeStrategy;
  preserveHeaders: boolean;
  addSeparator: boolean;
  separator?: string;
  customPattern?: string;
}

/** Result of a file operation */
export interface OperationResult {
  success: boolean;
  message: string;
  affectedFiles: string[];
}

/** Conflict detected during merge preview */
export interface MergeConflict {
  line: number;
  sourceContent: string;
  targetContent: string;
  conflictType: 'header' | 'section' | 'content';
}

/** Preview of a merge operation before applying */
export interface MergePreview {
  sourceContent: string;
  targetContent: string;
  mergedContent: string;
  conflicts: MergeConflict[];
}

/** Status of an async file operation */
export type FileOperationStatus = 'idle' | 'pending' | 'running' | 'success' | 'error';

/** Tracked file operation with progress */
export interface TrackedFileOperation {
  id: string;
  operation: FileOperation;
  status: FileOperationStatus;
  result?: OperationResult;
  error?: string;
  startedAt?: Date;
  completedAt?: Date;
}

/** Batch operation configuration */
export interface BulkOperationConfig {
  operations: FileOperation[];
  stopOnError: boolean;
}

/** Result of a bulk operation */
export interface BulkOperationResult {
  results: OperationResult[];
  totalOperations: number;
  successCount: number;
  failureCount: number;
}

/** Clone operation for skills/agents */
export interface CloneOptions {
  sourceName: string;
  newName: string;
  includeMetadata?: boolean;
}
