import React, { useState, useEffect } from 'react';
import {
  GitBranch,
  Clock,
  Plus,
  History,
  ChevronRight,
  Loader2,
  CheckCircle,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

interface Checkpoint {
  id: string;
  name: string;
  createdAt: Date;
  description?: string;
  isActive: boolean;
}

interface CheckpointItemProps {
  checkpoint: Checkpoint;
  onRestore: (checkpoint: Checkpoint) => void;
}

const CheckpointItem: React.FC<CheckpointItemProps> = ({ checkpoint, onRestore }) => {
  const formatTimeAgo = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (days > 0) {return `${days}d ago`;}
    if (hours > 0) {return `${hours}h ago`;}
    if (minutes > 0) {return `${minutes}m ago`;}
    return 'Just now';
  };

  return (
    <div
      className={cn(
        "flex items-center justify-between p-2 rounded-md transition-colors",
        checkpoint.isActive
          ? "bg-green-500/10 border border-green-500/30"
          : "bg-muted/30 hover:bg-muted/50"
      )}
    >
      <div className="flex items-center gap-2 min-w-0">
        <div className="relative shrink-0">
          <GitBranch className="w-4 h-4 text-muted-foreground" />
          {checkpoint.isActive && (
            <CheckCircle className="absolute -top-1 -right-1 w-3 h-3 text-green-500 fill-background" />
          )}
        </div>
        <div className="min-w-0">
          <span className="text-sm font-medium truncate block">{checkpoint.name}</span>
          <span className="text-xs text-muted-foreground">
            {formatTimeAgo(checkpoint.createdAt)}
          </span>
        </div>
      </div>

      {!checkpoint.isActive && (
        <Button
          variant="ghost"
          size="sm"
          className="h-6 text-xs"
          onClick={() => onRestore(checkpoint)}
        >
          Restore
        </Button>
      )}
    </div>
  );
};

interface VersioningPanelProps {
  projectPath?: string;
}

export const VersioningPanel: React.FC<VersioningPanelProps> = ({ projectPath }) => {
  const [checkpoints, setCheckpoints] = useState<Checkpoint[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const currentVersion = '0.3.0';

  useEffect(() => {
    let isMounted = true;

    const fetchCheckpoints = async () => {
      setIsLoading(true);
      setError(null);

      try {
        // In real implementation, would fetch from API
        // For now, using mock data
        const mockCheckpoints: Checkpoint[] = [
          {
            id: '1',
            name: 'Current State',
            createdAt: new Date(),
            isActive: true,
          },
          {
            id: '2',
            name: 'Before refactor',
            createdAt: new Date(Date.now() - 3600000),
            isActive: false,
          },
          {
            id: '3',
            name: 'Initial setup',
            createdAt: new Date(Date.now() - 86400000),
            isActive: false,
          },
        ];

        if (isMounted) {
          setCheckpoints(mockCheckpoints);
        }
      } catch (err) {
        if (isMounted) {
          setError(err instanceof Error ? err.message : 'Failed to load checkpoints');
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    };

    fetchCheckpoints();

    return () => {
      isMounted = false;
    };
  }, [projectPath]);

  const handleCreateCheckpoint = async () => {
    setIsCreating(true);

    try {
      // In real implementation, would call API to create checkpoint
      const newCheckpoint: Checkpoint = {
        id: Date.now().toString(),
        name: `Checkpoint ${new Date().toLocaleTimeString()}`,
        createdAt: new Date(),
        isActive: true,
      };

      // Update existing checkpoints to not be active
      setCheckpoints((prev) => [
        newCheckpoint,
        ...prev.map((cp) => ({ ...cp, isActive: false })),
      ]);
    } catch (err) {
      console.error('Failed to create checkpoint:', err);
    } finally {
      setIsCreating(false);
    }
  };

  const handleRestoreCheckpoint = async (checkpoint: Checkpoint) => {
    // In real implementation, would call API to restore
    setCheckpoints((prev) =>
      prev.map((cp) => ({
        ...cp,
        isActive: cp.id === checkpoint.id,
      }))
    );
  };

  const lastCheckpointTime = checkpoints.length > 0
    ? checkpoints[0].createdAt
    : null;

  const formatLastCheckpoint = () => {
    if (!lastCheckpointTime) {return 'No checkpoints';}

    const now = new Date();
    const diff = now.getTime() - lastCheckpointTime.getTime();
    const minutes = Math.floor(diff / 60000);

    if (minutes === 0) {return 'Just now';}
    if (minutes < 60) {return `${minutes}m ago`;}
    return `${Math.floor(minutes / 60)}h ago`;
  };

  return (
    <Card className="shrink-0">
      <CardHeader className="pb-2 pt-4 px-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium flex items-center gap-2">
            <History className="w-4 h-4 text-purple-500" />
            Versioning
          </CardTitle>
          <Badge variant="secondary" className="text-xs">
            v{currentVersion}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="px-4 pb-4">
        {/* Version Info */}
        <div className="flex items-center justify-between p-3 rounded-lg bg-muted/30 mb-3">
          <div className="flex items-center gap-2">
            <Clock className="w-4 h-4 text-muted-foreground" />
            <div>
              <span className="text-sm font-medium">Last Checkpoint</span>
              <p className="text-xs text-muted-foreground">{formatLastCheckpoint()}</p>
            </div>
          </div>
        </div>

        {/* Create Checkpoint Button */}
        <Button
          variant="outline"
          size="sm"
          className="w-full mb-3"
          onClick={handleCreateCheckpoint}
          disabled={isCreating}
        >
          {isCreating ? (
            <Loader2 className="w-4 h-4 mr-2 animate-spin" />
          ) : (
            <Plus className="w-4 h-4 mr-2" />
          )}
          Create Checkpoint
        </Button>

        {/* Checkpoints List */}
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
          </div>
        ) : error ? (
          <div className="text-sm text-destructive text-center py-4">{error}</div>
        ) : checkpoints.length > 0 ? (
          <div className="space-y-1">
            {checkpoints.slice(0, 5).map((checkpoint) => (
              <CheckpointItem
                key={checkpoint.id}
                checkpoint={checkpoint}
                onRestore={handleRestoreCheckpoint}
              />
            ))}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground text-center py-4">
            No checkpoints yet
          </div>
        )}

        {checkpoints.length > 5 && (
          <button className="w-full mt-3 flex items-center justify-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors py-1">
            View All Checkpoints
            <ChevronRight className="w-3 h-3" />
          </button>
        )}
      </CardContent>
    </Card>
  );
};

export default VersioningPanel;
