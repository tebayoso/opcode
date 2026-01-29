import { useCallback } from 'react';
import { useToast } from './ToastProvider';

export function useApiError() {
  const { addToast } = useToast();

  const handleError = useCallback(
    (error: unknown, context?: string) => {
      let message = 'An unexpected error occurred';

      if (error instanceof Error) {
        message = error.message;
      } else if (typeof error === 'string') {
        message = error;
      }

      console.error(context ? `[${context}] ${message}` : message, error);

      addToast({
        title: context || 'Error',
        description: message,
        variant: 'destructive',
      });

      return message;
    },
    [addToast]
  );

  const handleSuccess = useCallback(
    (title: string, description?: string) => {
      addToast({
        title,
        description,
        variant: 'success',
      });
    },
    [addToast]
  );

  return { handleError, handleSuccess };
}
