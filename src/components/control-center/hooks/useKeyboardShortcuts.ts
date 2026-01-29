import { useEffect, useCallback } from 'react';
import { useFileOpsStore } from '@/stores/fileOpsStore';

interface KeyboardShortcutConfig {
  onOpenControlCenter?: () => void;
  onEscape?: () => void;
  enabled?: boolean;
}

/**
 * Hook to manage keyboard shortcuts for the Control Center
 *
 * Shortcuts:
 * - Ctrl+Shift+C (or Cmd+Shift+C on Mac): Open Control Center
 * - Escape: Clear file selection
 */
export const useKeyboardShortcuts = ({
  onOpenControlCenter,
  onEscape,
  enabled = true,
}: KeyboardShortcutConfig = {}) => {
  const { clearSelection, selectedFiles } = useFileOpsStore();

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (!enabled) return;

    // Check if we're in an input field
    const target = event.target as HTMLElement;
    const isInputField =
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable;

    // Ctrl+Shift+C (or Cmd+Shift+C on Mac): Open Control Center
    if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'C') {
      event.preventDefault();
      onOpenControlCenter?.();
      return;
    }

    // Escape: Clear selection (only if not in input field)
    if (event.key === 'Escape' && !isInputField) {
      if (selectedFiles.length > 0) {
        event.preventDefault();
        clearSelection();
        onEscape?.();
      }
      return;
    }
  }, [enabled, onOpenControlCenter, onEscape, clearSelection, selectedFiles.length]);

  useEffect(() => {
    if (!enabled) return;

    window.addEventListener('keydown', handleKeyDown);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [enabled, handleKeyDown]);

  return {
    clearSelection,
    hasSelection: selectedFiles.length > 0,
  };
};

export default useKeyboardShortcuts;
