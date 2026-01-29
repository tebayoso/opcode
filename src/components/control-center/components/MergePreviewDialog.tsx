import React, { useEffect, useState } from 'react';
import {
  AlertTriangle,
  Check,
  X,
  Loader2,
  FileText,
  ArrowRight,
  Merge,
} from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { useFileOpsStore, useMergePreview, useMergeDialogOpen } from '@/stores/fileOpsStore';
import type { MergeStrategy, MergeConflict } from '@/types/file-ops';

interface MergePreviewDialogProps {
  onMergeComplete?: () => void;
}

const ConflictItem: React.FC<{ conflict: MergeConflict }> = ({ conflict }) => (
    <div className="p-3 rounded-lg border border-yellow-500/30 bg-yellow-500/5 mb-2">
      <div className="flex items-start gap-2">
        <AlertTriangle className="w-4 h-4 text-yellow-500 shrink-0 mt-0.5" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm font-medium capitalize">{conflict.conflictType}</span>
            <Badge variant="outline" className="text-xs">
              Line {conflict.line}
            </Badge>
          </div>
          <div className="grid grid-cols-2 gap-2 mt-2">
            <div className="p-2 rounded bg-red-500/10 border border-red-500/20">
              <span className="text-xs text-red-400 block mb-1">Source</span>
              <pre className="text-xs text-muted-foreground whitespace-pre-wrap break-words">
                {conflict.sourceContent.slice(0, 100)}
                {conflict.sourceContent.length > 100 && '...'}
              </pre>
            </div>
            <div className="p-2 rounded bg-blue-500/10 border border-blue-500/20">
              <span className="text-xs text-blue-400 block mb-1">Target</span>
              <pre className="text-xs text-muted-foreground whitespace-pre-wrap break-words">
                {conflict.targetContent.slice(0, 100)}
                {conflict.targetContent.length > 100 && '...'}
              </pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  );

export const MergePreviewDialog: React.FC<MergePreviewDialogProps> = ({
  onMergeComplete,
}) => {
  const isOpen = useMergeDialogOpen();
  const preview = useMergePreview();
  const { closeMergeDialog, previewMerge, executeMerge, setMergeOptions } = useFileOpsStore();

  const [strategy, setStrategy] = useState<MergeStrategy>('append');
  const [preserveHeaders, setPreserveHeaders] = useState(true);
  const [addSeparator, setAddSeparator] = useState(true);
  const [isApplying, setIsApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const sourcePath = useFileOpsStore((state) => state.mergeSource);
  const targetPath = useFileOpsStore((state) => state.mergeTarget);
  const isLoading = useFileOpsStore((state) => state.isPreviewLoading);

  // Track previous options to prevent unnecessary re-fetches
  const prevOptionsRef = React.useRef({ strategy, preserveHeaders, addSeparator });

  // Fetch preview when options change (initial preview is triggered by openMergeDialog)
  useEffect(() => {
    if (!isOpen || !sourcePath || !targetPath) {return;}

    // Check if options actually changed to prevent infinite loops
    const prevOptions = prevOptionsRef.current;
    if (
      prevOptions.strategy === strategy &&
      prevOptions.preserveHeaders === preserveHeaders &&
      prevOptions.addSeparator === addSeparator
    ) {
      return;
    }

    // Update ref with current options
    prevOptionsRef.current = { strategy, preserveHeaders, addSeparator };

    // Update options and re-fetch preview
    const mergeOptions = {
      strategy,
      preserveHeaders,
      addSeparator,
      separator: '\n---\n',
    };

    setMergeOptions(mergeOptions);
    previewMerge(sourcePath, targetPath, mergeOptions);
  }, [isOpen, sourcePath, targetPath, strategy, preserveHeaders, addSeparator, setMergeOptions, previewMerge]);

  const handleApplyMerge = async () => {
    if (!sourcePath || !targetPath) {return;}

    setIsApplying(true);
    setError(null);

    try {
      const result = await executeMerge();
      if (result && !result.success) {
        setError(result.message);
      } else {
        onMergeComplete?.();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to apply merge');
    } finally {
      setIsApplying(false);
    }
  };

  const handleClose = () => {
    closeMergeDialog();
    setError(null);
  };

  const getFileName = (path: string) => path.split('/').pop() || path;

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && handleClose()}>
      <DialogContent className="max-w-4xl max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Merge className="w-5 h-5 text-primary" />
            Merge Preview
          </DialogTitle>
          <DialogDescription>
            Review the merge result before applying changes
          </DialogDescription>
        </DialogHeader>

        {/* File Info */}
        {sourcePath && targetPath && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-muted/30 text-sm">
            <FileText className="w-4 h-4 text-muted-foreground" />
            <span className="font-medium">{getFileName(sourcePath)}</span>
            <ArrowRight className="w-4 h-4 text-muted-foreground" />
            <span className="font-medium">{getFileName(targetPath)}</span>
          </div>
        )}

        {/* Merge Options */}
        <div className="grid grid-cols-3 gap-4 p-3 rounded-lg border bg-card">
          <div className="space-y-2">
            <Label htmlFor="strategy" className="text-xs">Strategy</Label>
            <Select value={strategy} onValueChange={(v) => setStrategy(v as MergeStrategy)}>
              <SelectTrigger id="strategy" className="h-8">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="append">Append</SelectItem>
                <SelectItem value="prepend">Prepend</SelectItem>
                <SelectItem value="section-based">Section Based</SelectItem>
                <SelectItem value="custom">Custom</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="preserve-headers" className="text-xs">
              Preserve Headers
            </Label>
            <Switch
              id="preserve-headers"
              checked={preserveHeaders}
              onCheckedChange={setPreserveHeaders}
            />
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="add-separator" className="text-xs">
              Add Separator
            </Label>
            <Switch
              id="add-separator"
              checked={addSeparator}
              onCheckedChange={setAddSeparator}
            />
          </div>
        </div>

        {/* Content Preview */}
        <div className="flex-1 min-h-0 grid grid-cols-3 gap-4">
          {/* Source Content */}
          <div className="flex flex-col min-h-0">
            <div className="flex items-center gap-2 mb-2">
              <Badge variant="outline" className="text-xs bg-red-500/10 border-red-500/30">
                Source
              </Badge>
            </div>
            <ScrollArea className="flex-1 border rounded-lg p-3 bg-muted/20">
              {isLoading ? (
                <div className="flex items-center justify-center h-32">
                  <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
                </div>
              ) : (
                <pre className="text-xs text-muted-foreground whitespace-pre-wrap">
                  {preview?.sourceContent || 'No content'}
                </pre>
              )}
            </ScrollArea>
          </div>

          {/* Target Content */}
          <div className="flex flex-col min-h-0">
            <div className="flex items-center gap-2 mb-2">
              <Badge variant="outline" className="text-xs bg-blue-500/10 border-blue-500/30">
                Target
              </Badge>
            </div>
            <ScrollArea className="flex-1 border rounded-lg p-3 bg-muted/20">
              {isLoading ? (
                <div className="flex items-center justify-center h-32">
                  <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
                </div>
              ) : (
                <pre className="text-xs text-muted-foreground whitespace-pre-wrap">
                  {preview?.targetContent || 'No content'}
                </pre>
              )}
            </ScrollArea>
          </div>

          {/* Merged Result */}
          <div className="flex flex-col min-h-0">
            <div className="flex items-center gap-2 mb-2">
              <Badge variant="outline" className="text-xs bg-green-500/10 border-green-500/30">
                Result
              </Badge>
              {preview?.conflicts && preview.conflicts.length > 0 && (
                <Badge variant="destructive" className="text-xs">
                  {preview.conflicts.length} conflicts
                </Badge>
              )}
            </div>
            <ScrollArea className="flex-1 border rounded-lg p-3 bg-muted/20">
              {isLoading ? (
                <div className="flex items-center justify-center h-32">
                  <Loader2 className="w-5 h-5 animate-spin text-muted-foreground" />
                </div>
              ) : (
                <pre className="text-xs text-muted-foreground whitespace-pre-wrap">
                  {preview?.mergedContent || 'No content'}
                </pre>
              )}
            </ScrollArea>
          </div>
        </div>

        {/* Conflicts Section */}
        {preview?.conflicts && preview.conflicts.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-yellow-500" />
              Conflicts ({preview.conflicts.length})
            </h4>
            <ScrollArea className="max-h-32">
              {preview.conflicts.map((conflict, index) => (
                <ConflictItem key={index} conflict={conflict} />
              ))}
            </ScrollArea>
          </div>
        )}

        {/* Error Display */}
        {error && (
          <div className="p-3 rounded-lg border border-destructive/30 bg-destructive/5 text-sm text-destructive">
            {error}
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleClose} disabled={isApplying}>
            <X className="w-4 h-4 mr-2" />
            Cancel
          </Button>
          <Button
            onClick={handleApplyMerge}
            disabled={isLoading || isApplying || !preview}
          >
            {isApplying ? (
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <Check className="w-4 h-4 mr-2" />
            )}
            Apply Merge
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default MergePreviewDialog;
