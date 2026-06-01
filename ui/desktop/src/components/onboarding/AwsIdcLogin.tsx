import { useState } from 'react';
import { Cloud, ChevronRight, ArrowLeft } from 'lucide-react';
import { Button } from '../ui/button';
import {
  awsIdcStart,
  awsIdcAccounts,
  awsIdcRoles,
  awsIdcComplete,
} from '../../api';

// IDC account shape mirrors the goose backend response. We don't import from
// types.gen.ts because the openapi schema doesn't list IdcAccount.
type IdcAccount = {
  account_id: string;
  account_name: string;
  email_address?: string | null;
};

type Phase =
  | 'input'
  | 'authenticating'
  | 'pick-account'
  | 'pick-role'
  | 'completing'
  | 'error';

interface AwsIdcLoginProps {
  /// Called once Bedrock is configured and the user is ready to enter the app.
  onConfigured: (providerName: string) => void;
  /// Optional back handler — used when this component is shown as a sub-screen.
  onBack?: () => void;
}

const REGIONS = [
  'us-east-1',
  'us-east-2',
  'us-west-2',
  'eu-west-1',
  'eu-central-1',
  'ap-southeast-1',
  'ap-southeast-2',
  'ap-northeast-1',
];

export default function AwsIdcLogin({ onConfigured, onBack }: AwsIdcLoginProps) {
  const [phase, setPhase] = useState<Phase>('input');
  const [startUrl, setStartUrl] = useState('');
  const [region, setRegion] = useState('us-east-1');
  const [error, setError] = useState<string | null>(null);

  const [accounts, setAccounts] = useState<IdcAccount[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<IdcAccount | null>(null);
  const [roles, setRoles] = useState<string[]>([]);

  const reset = () => {
    setPhase('input');
    setError(null);
    setAccounts([]);
    setSelectedAccount(null);
    setRoles([]);
  };

  const handleSignIn = async () => {
    setError(null);
    if (!startUrl.trim()) {
      setError('Please enter your IDC Start URL.');
      return;
    }
    setPhase('authenticating');
    try {
      const resp = await awsIdcStart({
        body: { start_url: startUrl.trim(), region },
        throwOnError: true,
      });
      const data = resp.data;
      if (!data?.success) {
        setError(data?.message || 'Authentication failed.');
        setPhase('error');
        return;
      }
      // Fetch accounts and decide whether to skip the picker.
      const accountsResp = await awsIdcAccounts({ throwOnError: true });
      const list = (accountsResp.data as IdcAccount[]) || [];
      setAccounts(list);
      if (list.length === 0) {
        setError(
          'You are signed in, but have no AWS accounts assigned in this IDC instance.'
        );
        setPhase('error');
        return;
      }
      if (list.length === 1) {
        await loadRoles(list[0]);
      } else {
        setPhase('pick-account');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase('error');
    }
  };

  const loadRoles = async (account: IdcAccount) => {
    setSelectedAccount(account);
    try {
      const resp = await awsIdcRoles({
        query: { account_id: account.account_id },
        throwOnError: true,
      });
      const list = (resp.data as string[]) || [];
      setRoles(list);
      if (list.length === 0) {
        setError(`No roles assigned to you in account ${account.account_id}.`);
        setPhase('error');
        return;
      }
      if (list.length === 1) {
        await pickRole(account, list[0]);
      } else {
        setPhase('pick-role');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase('error');
    }
  };

  const pickRole = async (account: IdcAccount, roleName: string) => {
    setPhase('completing');
    try {
      const resp = await awsIdcComplete({
        body: {
          account_id: account.account_id,
          account_name: account.account_name,
          role_name: roleName,
        },
        throwOnError: true,
      });
      const data = resp.data;
      if (!data?.success) {
        setError(data?.message || 'Failed to finalize sign-in.');
        setPhase('error');
        return;
      }
      onConfigured('aws_bedrock');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase('error');
    }
  };

  return (
    <div className="flex flex-col gap-4 max-w-lg mx-auto">
      {onBack && phase === 'input' && (
        <button
          onClick={onBack}
          className="flex items-center gap-1 text-sm text-text-muted hover:text-text-default w-fit"
        >
          <ArrowLeft size={14} />
          Back
        </button>
      )}

      <div className="flex items-center gap-3">
        <Cloud className="w-8 h-8 text-blue-500" />
        <div>
          <h2 className="text-xl font-semibold">Sign in with AWS</h2>
          <p className="text-sm text-text-muted">
            Use your IAM Identity Center credentials to authenticate to Bedrock and your
            OpenSearch clusters.
          </p>
        </div>
      </div>

      {phase === 'input' && (
        <>
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium">IDC Start URL</label>
            <input
              type="url"
              value={startUrl}
              onChange={(e) => setStartUrl(e.target.value)}
              placeholder="https://d-xxxxxxxxxx.awsapps.com/start"
              className="px-3 py-2 border rounded-lg bg-transparent"
              autoFocus
            />
            <span className="text-xs text-text-muted">
              Ask your AWS admin or check your team's onboarding docs.
            </span>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium">AWS Region</label>
            <select
              value={region}
              onChange={(e) => setRegion(e.target.value)}
              className="px-3 py-2 border rounded-lg bg-transparent"
            >
              {REGIONS.map((r) => (
                <option key={r} value={r}>
                  {r}
                </option>
              ))}
            </select>
          </div>

          {error && (
            <div className="text-sm text-red-500 border border-red-500/30 rounded-lg p-3">
              {error}
            </div>
          )}

          <Button onClick={handleSignIn} size="lg" className="self-start">
            Sign in with AWS
          </Button>
        </>
      )}

      {phase === 'authenticating' && (
        <div className="flex flex-col gap-2 p-4 border rounded-lg">
          <p className="font-medium">Opening your browser…</p>
          <p className="text-sm text-text-muted">
            We've copied a verification code to your clipboard. Paste it in the IDC sign-in
            page and approve the sign-in. This page will continue once you complete it.
          </p>
        </div>
      )}

      {phase === 'pick-account' && (
        <div className="flex flex-col gap-2">
          <p className="font-medium">Select an AWS account</p>
          <div className="flex flex-col gap-2 max-h-96 overflow-y-auto">
            {accounts.map((acct) => (
              <button
                key={acct.account_id}
                onClick={() => loadRoles(acct)}
                className="w-full flex items-center justify-between gap-3 p-3 text-left border rounded-lg hover:border-blue-400"
              >
                <div className="flex flex-col">
                  <span className="font-medium">{acct.account_name}</span>
                  <span className="text-xs text-text-muted">
                    {acct.account_id}
                    {acct.email_address ? ` · ${acct.email_address}` : ''}
                  </span>
                </div>
                <ChevronRight size={16} />
              </button>
            ))}
          </div>
        </div>
      )}

      {phase === 'pick-role' && selectedAccount && (
        <div className="flex flex-col gap-2">
          <p className="font-medium">
            Select a role in {selectedAccount.account_name}
          </p>
          <div className="flex flex-col gap-2 max-h-96 overflow-y-auto">
            {roles.map((roleName) => (
              <button
                key={roleName}
                onClick={() => pickRole(selectedAccount, roleName)}
                className="w-full flex items-center justify-between gap-3 p-3 text-left border rounded-lg hover:border-blue-400"
              >
                <span>{roleName}</span>
                <ChevronRight size={16} />
              </button>
            ))}
          </div>
        </div>
      )}

      {phase === 'completing' && (
        <div className="flex flex-col gap-2 p-4 border rounded-lg">
          <p className="font-medium">Finalizing sign-in…</p>
          <p className="text-sm text-text-muted">Fetching temporary credentials.</p>
        </div>
      )}

      {phase === 'error' && (
        <div className="flex flex-col gap-3">
          <div className="text-sm text-red-500 border border-red-500/30 rounded-lg p-3">
            {error}
          </div>
          <Button onClick={reset} variant="outline">
            Try again
          </Button>
        </div>
      )}
    </div>
  );
}
