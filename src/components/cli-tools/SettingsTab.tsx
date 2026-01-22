/**
 * Settings Tab
 * Structured settings editor with categorized controls
 */

import { useState } from 'react';
import {
  Settings,
  Save,
  RotateCcw,
  ChevronDown,
  ChevronRight,
  Info,
  Loader2,
} from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Badge } from '@/components/ui/badge';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useCLIToolConfigStore } from '@/stores/cliToolConfigStore';
import type {
  CLIToolType,
  ToolSettings,
  SettingDefinition,
  SettingsCategory,
} from '@/types/cli-tools';

interface SettingsTabProps {
  toolType: CLIToolType;
  settings: ToolSettings | null | undefined;
}

export function SettingsTab({ toolType, settings }: SettingsTabProps) {
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(
    new Set(['General'])
  );
  const [pendingChanges, setPendingChanges] = useState<
    Record<string, unknown>
  >({});
  const [savingKeys, setSavingKeys] = useState<Set<string>>(new Set());

  const { updateSetting, loadSettings } = useCLIToolConfigStore();

  const toggleCategory = (name: string) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  };

  const handleChange = (key: string, value: unknown) => {
    setPendingChanges((prev) => ({ ...prev, [key]: value }));
  };

  const handleSave = async (key: string) => {
    const value = pendingChanges[key];
    if (value === undefined) return;

    setSavingKeys((prev) => new Set(prev).add(key));
    try {
      await updateSetting(toolType, key, value);
      setPendingChanges((prev) => {
        const next = { ...prev };
        delete next[key];
        return next;
      });
    } finally {
      setSavingKeys((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
    }
  };

  const handleReset = (key: string, defaultValue: unknown) => {
    handleChange(key, defaultValue);
  };

  const handleRefresh = () => {
    loadSettings(toolType);
    setPendingChanges({});
  };

  if (!settings || !settings.categories || settings.categories.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
        <Settings className="h-8 w-8 mb-2 opacity-50" />
        <p className="text-sm">No settings available</p>
        <p className="text-xs mt-1">
          This tool may not have configurable settings or the config file doesn't
          exist yet.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <span className="text-xs text-muted-foreground">
          {settings.categories.reduce((acc, cat) => acc + cat.settings.length, 0)}{' '}
          settings in {settings.categories.length} categories
        </span>
        <Button variant="ghost" size="sm" onClick={handleRefresh}>
          <RotateCcw className="h-3 w-3" />
        </Button>
      </div>

      <div className="space-y-3">
        {settings.categories.map((category) => (
          <CategorySection
            key={category.name}
            category={category}
            isExpanded={expandedCategories.has(category.name)}
            onToggle={() => toggleCategory(category.name)}
            pendingChanges={pendingChanges}
            savingKeys={savingKeys}
            onChange={handleChange}
            onSave={handleSave}
            onReset={handleReset}
          />
        ))}
      </div>
    </div>
  );
}

interface CategorySectionProps {
  category: SettingsCategory;
  isExpanded: boolean;
  onToggle: () => void;
  pendingChanges: Record<string, unknown>;
  savingKeys: Set<string>;
  onChange: (key: string, value: unknown) => void;
  onSave: (key: string) => void;
  onReset: (key: string, defaultValue: unknown) => void;
}

function CategorySection({
  category,
  isExpanded,
  onToggle,
  pendingChanges,
  savingKeys,
  onChange,
  onSave,
  onReset,
}: CategorySectionProps) {
  return (
    <div className="border rounded-md overflow-hidden">
      <button
        onClick={onToggle}
        className="w-full flex items-center gap-2 p-3 hover:bg-muted/50 transition-colors text-left"
      >
        {isExpanded ? (
          <ChevronDown className="h-4 w-4 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-4 w-4 text-muted-foreground" />
        )}
        <span className="font-medium text-sm">{category.name}</span>
        <Badge variant="secondary" className="text-[10px]">
          {category.settings.length}
        </Badge>
        {category.description && (
          <span className="text-xs text-muted-foreground ml-auto truncate max-w-[200px]">
            {category.description}
          </span>
        )}
      </button>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: 'auto' }}
            exit={{ height: 0 }}
            transition={{ duration: 0.15 }}
            className="overflow-hidden"
          >
            <div className="border-t divide-y">
              {category.settings.map((setting) => (
                <SettingRow
                  key={setting.key}
                  setting={setting}
                  pendingValue={pendingChanges[setting.key]}
                  isSaving={savingKeys.has(setting.key)}
                  onChange={(value) => onChange(setting.key, value)}
                  onSave={() => onSave(setting.key)}
                  onReset={() => onReset(setting.key, setting.default)}
                />
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

interface SettingRowProps {
  setting: SettingDefinition;
  pendingValue: unknown;
  isSaving: boolean;
  onChange: (value: unknown) => void;
  onSave: () => void;
  onReset: () => void;
}

function SettingRow({
  setting,
  pendingValue,
  isSaving,
  onChange,
  onSave,
  onReset,
}: SettingRowProps) {
  const currentValue = pendingValue !== undefined ? pendingValue : setting.value;
  const hasChanges = pendingValue !== undefined;
  const hasDefault = setting.default !== null && setting.default !== undefined;

  const renderControl = () => {
    switch (setting.value_type) {
      case 'boolean':
        return (
          <Switch
            checked={currentValue as boolean}
            onCheckedChange={onChange}
            disabled={setting.readonly || isSaving}
          />
        );

      case 'string':
        if (setting.options && setting.options.length > 0) {
          return (
            <Select
              value={currentValue as string}
              onValueChange={onChange}
              disabled={setting.readonly || isSaving}
            >
              <SelectTrigger className="w-[180px] h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {setting.options.map((opt) => (
                  <SelectItem key={opt} value={opt} className="text-xs">
                    {opt}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          );
        }
        return (
          <Input
            value={currentValue as string}
            onChange={(e) => onChange(e.target.value)}
            disabled={setting.readonly || isSaving}
            className="w-[180px] h-8 text-xs"
          />
        );

      case 'number':
        return (
          <Input
            type="number"
            value={currentValue as number}
            onChange={(e) => onChange(Number(e.target.value))}
            disabled={setting.readonly || isSaving}
            className="w-[120px] h-8 text-xs"
          />
        );

      case 'array':
      case 'object':
        return (
          <code className="text-xs bg-muted px-2 py-1 rounded max-w-[200px] truncate block">
            {JSON.stringify(currentValue)}
          </code>
        );

      default:
        return (
          <span className="text-xs text-muted-foreground">
            {String(currentValue)}
          </span>
        );
    }
  };

  return (
    <div className="p-3 flex items-center gap-3">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <Label className="text-sm font-medium">{setting.key}</Label>
          {setting.readonly && (
            <Badge variant="outline" className="text-[10px]">
              Read-only
            </Badge>
          )}
          {setting.description && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger>
                  <Info className="h-3 w-3 text-muted-foreground" />
                </TooltipTrigger>
                <TooltipContent>
                  <p className="max-w-[300px] text-xs">{setting.description}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
        </div>
        <div className="flex items-center gap-2 mt-1">
          <Badge variant="secondary" className="text-[10px]">
            {setting.value_type}
          </Badge>
          {hasDefault && (
            <span className="text-[10px] text-muted-foreground">
              Default: {JSON.stringify(setting.default)}
            </span>
          )}
        </div>
      </div>

      <div className="flex items-center gap-2">
        {renderControl()}

        {hasChanges && !setting.readonly && (
          <>
            {hasDefault && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onReset}
                disabled={isSaving}
                className="h-8 px-2"
              >
                <RotateCcw className="h-3 w-3" />
              </Button>
            )}
            <Button
              size="sm"
              onClick={onSave}
              disabled={isSaving}
              className="h-8 px-2"
            >
              {isSaving ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <Save className="h-3 w-3" />
              )}
            </Button>
          </>
        )}
      </div>
    </div>
  );
}
