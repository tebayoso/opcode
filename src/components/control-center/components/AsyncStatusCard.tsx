import React from 'react';
import { Loader2, AlertCircle, CheckCircle, RefreshCw } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

export type AsyncStatus = 'idle' | 'loading' | 'success' | 'error';

interface AsyncStatusCardProps {
  title: string;
  icon?: React.ReactNode;
  status: AsyncStatus;
  error?: string | null;
  onRetry?: () => void;
  children?: React.ReactNode;
  className?: string;
}

export const AsyncStatusCard: React.FC<AsyncStatusCardProps> = ({
  title,
  icon,
  status,
  error,
  onRetry,
  children,
  className,
}) => {
  const renderStatusIndicator = () => {
    switch (status) {
      case 'loading':
        return (
          <div className="flex items-center gap-2">
            <Loader2 className="w-4 h-4 animate-spin text-primary" />
            <span className="text-xs text-muted-foreground">Loading...</span>
          </div>
        );
      case 'success':
        return (
          <CheckCircle className="w-4 h-4 text-green-500" />
        );
      case 'error':
        return (
          <div className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 text-destructive" />
            {onRetry && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={onRetry}
                title="Retry"
              >
                <RefreshCw className="w-3 h-3" />
              </Button>
            )}
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <Card className={cn("transition-all", className)}>
      <CardHeader className="pb-2 pt-4 px-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            {icon}
            {title}
          </CardTitle>
          {renderStatusIndicator()}
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4">
        {status === 'loading' ? (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
          </div>
        ) : status === 'error' && error ? (
          <div className="flex flex-col items-center justify-center py-6 text-center">
            <AlertCircle className="w-8 h-8 text-destructive/50 mb-2" />
            <p className="text-sm text-destructive">{error}</p>
            {onRetry && (
              <Button
                variant="outline"
                size="sm"
                className="mt-3"
                onClick={onRetry}
              >
                <RefreshCw className="w-3 h-3 mr-2" />
                Try Again
              </Button>
            )}
          </div>
        ) : (
          children
        )}
      </CardContent>
    </Card>
  );
};

export default AsyncStatusCard;
