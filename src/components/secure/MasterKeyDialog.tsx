import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Lock, Loader2, Shield } from 'lucide-react';

interface MasterKeyDialogProps {
  isOpen: boolean;
  onUnlock: () => void;
}

export function MasterKeyDialog({ isOpen, onUnlock }: MasterKeyDialogProps) {
  const [masterKey, setMasterKey] = useState('');
  const [confirmKey, setConfirmKey] = useState('');
  const [isFirstTime, setIsFirstTime] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showSetup, setShowSetup] = useState(false);

  useEffect(() => {
    checkIfFirstTime();
  }, []);

  const checkIfFirstTime = async () => {
    try {
      const result = await invoke<boolean>('check_master_key_exists');
      setIsFirstTime(!result);
      setShowSetup(!result);
    } catch (error) {
      console.error('Failed to check master key:', error);
      setIsFirstTime(true);
      setShowSetup(true);
    }
  };

  const handleSetup = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (masterKey.length < 8) {
      setError('Master key must be at least 8 characters long');
      return;
    }

    if (masterKey !== confirmKey) {
      setError('Master keys do not match');
      return;
    }

    setIsLoading(true);
    try {
      await invoke('initialize_master_key', { password: masterKey });
      onUnlock();
    } catch (error) {
      setError('Failed to initialize master key: ' + error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    setIsLoading(true);
    try {
      await invoke('unlock_with_master_key', { password: masterKey });
      onUnlock();
    } catch (error) {
      setError('Invalid master key');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Dialog open={isOpen} modal={true}>
      <DialogContent className="sm:max-w-md" onPointerDownOutside={(e) => e.preventDefault()}>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Shield className="w-6 h-6 text-primary" />
            {showSetup ? 'Set Up Master Key' : 'Enter Master Key'}
          </DialogTitle>
          <DialogDescription>
            {showSetup
              ? 'Create a master key to encrypt your sensitive data. This key will be required each time you start opcode.'
              : 'Enter your master key to decrypt your sensitive data.'}
          </DialogDescription>
        </DialogHeader>

        {error && (
          <Alert variant="destructive" className="mt-4">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {showSetup ? (
          <form onSubmit={handleSetup} className="space-y-4 mt-4">
            <div className="space-y-2">
              <Label htmlFor="master-key">Master Key</Label>
              <Input
                id="master-key"
                type="password"
                placeholder="Enter a strong master key"
                value={masterKey}
                onChange={(e) => setMasterKey(e.target.value)}
                required
                minLength={8}
              />
              <p className="text-xs text-muted-foreground">
                Must be at least 8 characters. Use a strong, memorable passphrase.
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="confirm-key">Confirm Master Key</Label>
              <Input
                id="confirm-key"
                type="password"
                placeholder="Confirm your master key"
                value={confirmKey}
                onChange={(e) => setConfirmKey(e.target.value)}
                required
              />
            </div>

            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading ? (
                <><Loader2 className="w-4 h-4 mr-2 animate-spin" />Setting up...</>
              ) : (
                <><Lock className="w-4 h-4 mr-2" />Set Master Key</>
              )}
            </Button>
          </form>
        ) : (
          <form onSubmit={handleUnlock} className="space-y-4 mt-4">
            <div className="space-y-2">
              <Label htmlFor="unlock-key">Master Key</Label>
              <Input
                id="unlock-key"
                type="password"
                placeholder="Enter your master key"
                value={masterKey}
                onChange={(e) => setMasterKey(e.target.value)}
                required
                autoFocus
              />
            </div>

            <Button type="submit" className="w-full" disabled={isLoading}>
              {isLoading ? (
                <><Loader2 className="w-4 h-4 mr-2 animate-spin" />Unlocking...</>
              ) : (
                <><Lock className="w-4 h-4 mr-2" />Unlock</>
              )}
            </Button>
          </form>
        )}
      </DialogContent>
    </Dialog>
  );
}
